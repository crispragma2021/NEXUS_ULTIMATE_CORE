// ============================================================================
// 💾 SESSION STORE — Persistencia SQLite del pipeline V0
// ============================================================================
// Almacena SessionState (plan, código, diff history, design tokens, métricas)
// por sesión de generación de UI. Permite continuidad conversacional:
// "cambia el header" modifica la app existente.
//
// Reutiliza el patrón de `core/src/browser/session_manager.rs`:
// SQLite + Mutex<Connection> + UUID v4.
// ============================================================================

use crate::cerebro::v0::contracts::{GeneracionUI, PlanComponentes, SessionState};
use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use std::path::PathBuf;
use std::sync::Mutex;
use uuid::Uuid;

/// Error tipado del session store.
#[derive(Debug, thiserror::Error)]
pub enum SessionStoreError {
    #[error("error de sqlite: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("serialización: {0}")]
    Json(#[from] serde_json::Error),
    #[error("sesión no encontrada: {0}")]
    NoEncontrada(String),
}

type Result<T> = std::result::Result<T, SessionStoreError>;

/// Gestor de persistencia de sesiones V0.
pub struct SessionStore {
    conn: Mutex<Connection>,
    db_path: PathBuf,
}

/// Convierte un error de lock (poison) en un `rusqlite::Error` boxeado,
/// que es lo que espera la variante `ToSqlConversionFailure`.
fn error_lock(e: &str) -> SessionStoreError {
    SessionStoreError::Sqlite(rusqlite::Error::ToSqlConversionFailure(Box::new(
        std::io::Error::other(e.to_string()),
    )))
}

impl SessionStore {
    /// Abre (o crea) la base de datos SQLite de sesiones V0.
    ///
    /// Si `db_path` es `None`, usa `NEXUS_ROOT/data/nexus_v0_sessions.db`
    /// con fallback a `/tmp`.
    pub fn new(db_path: Option<PathBuf>) -> Result<Self> {
        let path = db_path.unwrap_or_else(|| {
            let root = std::env::var("NEXUS_ROOT").unwrap_or_else(|_| "/tmp".into());
            let mut p = PathBuf::from(root);
            p.push("data");
            p.push("nexus_v0_sessions.db");
            p
        });

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                SessionStoreError::Sqlite(rusqlite::Error::ToSqlConversionFailure(e.into()))
            })?;
        }

        let conn = Connection::open(&path)?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS v0_sessions (
                session_id TEXT PRIMARY KEY,
                state_json TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );",
        )?;

        Ok(Self {
            conn: Mutex::new(conn),
            db_path: path,
        })
    }

    /// Ruta de la base de datos.
    pub fn db_path(&self) -> &PathBuf {
        &self.db_path
    }

    /// Crea una sesión nueva con `session_id` generado (UUID v4).
    pub fn crear_sesion(&self) -> Result<SessionState> {
        let now = Utc::now().to_rfc3339();
        let state = SessionState {
            schema: crate::cerebro::v0::contracts::V0_SCHEMA_SESSION.to_string(),
            session_id: Uuid::new_v4().to_string(),
            created_at: now.clone(),
            updated_at: now,
            current_plan: None,
            current_code: None,
            diff_history: Vec::new(),
            design_tokens: Default::default(),
            metrics: Default::default(),
        };
        self.guardar(&state)?;
        Ok(state)
    }

    /// Carga una sesión por su id.
    pub fn cargar(&self, session_id: &str) -> Result<SessionState> {
        let conn = self.conn.lock().map_err(|_| error_lock("lock cargar"))?;
        let row = conn
            .query_row(
                "SELECT state_json FROM v0_sessions WHERE session_id = ?1",
                params![session_id],
                |r| r.get::<_, String>(0),
            )
            .optional()?;

        match row {
            Some(json) => Ok(serde_json::from_str(&json)?),
            None => Err(SessionStoreError::NoEncontrada(session_id.to_string())),
        }
    }

    /// Guarda (inserta o sobrescribe) una sesión.
    pub fn guardar(&self, state: &SessionState) -> Result<()> {
        let json = serde_json::to_string(state)?;
        let conn = self.conn.lock().map_err(|_| error_lock("lock guardar"))?;
        conn.execute(
            "INSERT OR REPLACE INTO v0_sessions (session_id, state_json, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4)",
            params![state.session_id, json, state.created_at, state.updated_at],
        )?;
        Ok(())
    }

    /// Elimina una sesión.
    pub fn eliminar(&self, session_id: &str) -> Result<()> {
        let conn = self.conn.lock().map_err(|_| error_lock("lock eliminar"))?;
        let afectadas = conn.execute(
            "DELETE FROM v0_sessions WHERE session_id = ?1",
            params![session_id],
        )?;
        if afectadas == 0 {
            return Err(SessionStoreError::NoEncontrada(session_id.to_string()));
        }
        Ok(())
    }

    /// Lista los IDs de todas las sesiones (ordenadas por actualización desc).
    pub fn listar_sesiones(&self) -> Result<Vec<String>> {
        let conn = self.conn.lock().map_err(|_| error_lock("lock listar"))?;
        let mut stmt =
            conn.prepare("SELECT session_id FROM v0_sessions ORDER BY updated_at DESC")?;
        let ids = stmt
            .query_map([], |r| r.get::<_, String>(0))?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(ids)
    }

    /// Actualiza el plan actual de una sesión (toca `updated_at`).
    pub fn actualizar_plan(&self, session_id: &str, plan: PlanComponentes) -> Result<()> {
        let mut state = self.cargar(session_id)?;
        state.current_plan = Some(plan);
        state.updated_at = Utc::now().to_rfc3339();
        self.guardar(&state)
    }

    /// Actualiza el código actual de una sesión.
    pub fn actualizar_codigo(&self, session_id: &str, codigo: GeneracionUI) -> Result<()> {
        let mut state = self.cargar(session_id)?;
        state.current_code = Some(codigo);
        state.updated_at = Utc::now().to_rfc3339();
        self.guardar(&state)
    }

    /// Incrementa el contador de turnos y actualiza métricas.
    pub fn registrar_turno(&self, session_id: &str, latencia_ms: u64) -> Result<()> {
        let mut state = self.cargar(session_id)?;
        let m = &mut state.metrics;
        m.total_turns += 1;
        // promedio móvil
        let anterior = m.avg_latency_ms as u128 * (m.total_turns - 1) as u128;
        m.avg_latency_ms = ((anterior + latencia_ms as u128) / m.total_turns as u128) as u64;
        state.updated_at = Utc::now().to_rfc3339();
        self.guardar(&state)
    }

    /// Registra una falla de gate.
    pub fn registrar_gate_failure(&self, session_id: &str) -> Result<()> {
        let mut state = self.cargar(session_id)?;
        state.metrics.total_gate_failures += 1;
        state.updated_at = Utc::now().to_rfc3339();
        self.guardar(&state)
    }

    /// Registra una invocación del debugger.
    pub fn registrar_debugger(&self, session_id: &str) -> Result<()> {
        let mut state = self.cargar(session_id)?;
        state.metrics.total_debugger_invocations += 1;
        state.updated_at = Utc::now().to_rfc3339();
        self.guardar(&state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cerebro::v0::contracts::{ArchivoGenerado, GateKind, GateResult, V0_SCHEMA_GATE};

    fn store_tmp() -> SessionStore {
        let dir = std::env::temp_dir().join(format!("nexus_v0_test_{}", Uuid::new_v4()));
        SessionStore::new(Some(dir.join("sessions.db"))).unwrap()
    }

    fn plan_ejemplo() -> PlanComponentes {
        serde_json::from_value(serde_json::json!({
            "$schema": "nexus-v0-plan-v1",
            "app": {"name":"dashboard","description":"panel","framework":"react","styling":"tailwind","component_library":"shadcn/ui","theme":"light"},
            "page_tree": [{"path":"/","component":"DashboardPage","layout":"default"}],
            "component_tree": {"name":"App","source":"local","props":{},"children":[]},
            "dependencies": {"runtime":["react"],"ui":[],"styling":[],"utils":[]},
            "state_shape": {"useState":[],"useReducer":[],"context":[]}
        }))
        .unwrap()
    }

    fn codigo_ejemplo() -> GeneracionUI {
        GeneracionUI {
            schema: "nexus-v0-generate-v1".into(),
            plan_id: "plan-1".into(),
            files: vec![ArchivoGenerado {
                path: "src/App.tsx".into(),
                content: "export default function App(){return null;}".into(),
                language: "tsx".into(),
            }],
            package_json: Default::default(),
            entry_point: "src/App.tsx".into(),
        }
    }

    #[test]
    fn test_crear_sesion_genera_uuid() {
        let store = store_tmp();
        let s1 = store.crear_sesion().unwrap();
        let s2 = store.crear_sesion().unwrap();
        assert_ne!(s1.session_id, s2.session_id);
        assert!(s1.current_plan.is_none());
        assert_eq!(s1.metrics.total_turns, 0);
    }

    #[test]
    fn test_guardar_y_cargar_roundtrip() {
        let store = store_tmp();
        let s = store.crear_sesion().unwrap();
        let cargada = store.cargar(&s.session_id).unwrap();
        assert_eq!(s, cargada);
    }

    #[test]
    fn test_cargar_inexistente_error() {
        let store = store_tmp();
        match store.cargar("no-existe") {
            Err(SessionStoreError::NoEncontrada(id)) => assert_eq!(id, "no-existe"),
            other => panic!("esperaba NoEncontrada, obtuve {other:?}"),
        }
    }

    #[test]
    fn test_actualizar_plan_y_codigo() {
        let store = store_tmp();
        let s = store.crear_sesion().unwrap();
        store
            .actualizar_plan(&s.session_id, plan_ejemplo())
            .unwrap();
        store
            .actualizar_codigo(&s.session_id, codigo_ejemplo())
            .unwrap();
        let cargada = store.cargar(&s.session_id).unwrap();
        assert!(cargada.current_plan.is_some());
        assert!(cargada.current_code.is_some());
        assert_eq!(cargada.current_code.unwrap().entry_point, "src/App.tsx");
    }

    #[test]
    fn test_registrar_turno_acumula_metricas() {
        let store = store_tmp();
        let s = store.crear_sesion().unwrap();
        store.registrar_turno(&s.session_id, 3000).unwrap();
        store.registrar_turno(&s.session_id, 5000).unwrap();
        let cargada = store.cargar(&s.session_id).unwrap();
        assert_eq!(cargada.metrics.total_turns, 2);
        assert_eq!(cargada.metrics.avg_latency_ms, 4000);
    }

    #[test]
    fn test_registrar_gate_failure_y_debugger() {
        let store = store_tmp();
        let s = store.crear_sesion().unwrap();
        store.registrar_gate_failure(&s.session_id).unwrap();
        store.registrar_gate_failure(&s.session_id).unwrap();
        store.registrar_debugger(&s.session_id).unwrap();
        let cargada = store.cargar(&s.session_id).unwrap();
        assert_eq!(cargada.metrics.total_gate_failures, 2);
        assert_eq!(cargada.metrics.total_debugger_invocations, 1);
    }

    #[test]
    fn test_eliminar_sesion() {
        let store = store_tmp();
        let s = store.crear_sesion().unwrap();
        store.eliminar(&s.session_id).unwrap();
        assert!(matches!(
            store.cargar(&s.session_id),
            Err(SessionStoreError::NoEncontrada(_))
        ));
        // eliminar de nuevo falla
        assert!(matches!(
            store.eliminar(&s.session_id),
            Err(SessionStoreError::NoEncontrada(_))
        ));
    }

    #[test]
    fn test_listar_sesiones() {
        let store = store_tmp();
        let a = store.crear_sesion().unwrap();
        let b = store.crear_sesion().unwrap();
        let ids = store.listar_sesiones().unwrap();
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&a.session_id));
        assert!(ids.contains(&b.session_id));
    }

    #[test]
    fn test_persistencia_entre_instancias() {
        let dir = std::env::temp_dir().join(format!("nexus_v0_persist_{}", Uuid::new_v4()));
        let db = dir.join("sessions.db");
        {
            let store = SessionStore::new(Some(db.clone())).unwrap();
            let s = store.crear_sesion().unwrap();
            store
                .actualizar_plan(&s.session_id, plan_ejemplo())
                .unwrap();
        }
        // nueva instancia sobre la misma DB
        let store2 = SessionStore::new(Some(db)).unwrap();
        let ids = store2.listar_sesiones().unwrap();
        assert_eq!(ids.len(), 1);
        let cargada = store2.cargar(&ids[0]).unwrap();
        assert!(cargada.current_plan.is_some());
    }

    #[test]
    fn test_gate_result_guardar_en_plan_demo() {
        // Verifica que GateResult sea serializable dentro de un contrato
        let gate = GateResult {
            schema: V0_SCHEMA_GATE.into(),
            gate: GateKind::Render,
            passed: true,
            errors: vec![],
            runtime_errors: vec![],
            visual_issues: vec![],
            duration_ms: 88,
        };
        let json = serde_json::to_string(&gate).unwrap();
        assert!(json.contains("\"render\""));
        let gate2: GateResult = serde_json::from_str(&json).unwrap();
        assert_eq!(gate, gate2);
    }
}
