// core/src/cerebro/synapse/mod.rs

mod consolidacion;
mod difusion;
mod nodo;
mod sintesis;
mod types;

pub use types::{EnlaceSinaptico, GrafoSinapsis, IDNodo, NodoSinaptico};

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use tracing::{info, warn};

pub use consolidacion::{coseno_similaridad, MonitorCognitivo};
pub use difusion::Difusor;
pub use nodo::NodoConcepto;
pub use sintesis::SintetizadorBroca;

/// Motor Synapse — grafo conceptual del GOI.
///
/// Contiene los conceptos base y dinámicos, ejecuta difusión de activación
/// y puede persistir conceptos dinámicos a SQLite (`intelligence.db`).
pub struct MotorSynapse {
    pub conceptos: HashMap<String, NodoConcepto>,
    pub difusor: Difusor,
    pub broca: SintetizadorBroca,
    pub umbral_expresion: f32,
    /// Ruta opcional a la base de datos SQLite para persistencia.
    /// Si es `None`, `guardar_en_db()` y `cargar_desde_db()` son no-ops.
    pub db_path: Option<PathBuf>,
    /// Nombres de los conceptos base — se excluyen del guardado como dinámicos.
    conceptos_base: HashSet<String>,
}

impl Default for MotorSynapse {
    fn default() -> Self {
        Self::new()
    }
}

impl MotorSynapse {
    pub fn new() -> Self {
        let mut motor = Self {
            conceptos: HashMap::new(),
            difusor: Difusor::new(),
            broca: SintetizadorBroca::new(),
            umbral_expresion: 0.6,
            db_path: None,
            conceptos_base: HashSet::new(),
        };
        motor.cargar_conceptos_base();
        motor
    }

    /// Configura la ruta de persistencia y carga los conceptos dinámicos desde DB.
    ///
    /// Llamar después de `new()` para activar la persistencia:
    /// ```
    /// let mut syn = MotorSynapse::new();
    /// syn.set_db_path(db_path);
    /// syn.cargar_desde_db()?;
    /// ```
    pub fn set_db_path(&mut self, path: PathBuf) {
        self.db_path = Some(path);
    }

    // ─── Persistencia: SQLite ───────────────────────────────────────────────

    fn inicializar_tabla_synapse(conn: &rusqlite::Connection) -> rusqlite::Result<()> {
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS synapse_conceptos (
                nombre       TEXT PRIMARY KEY,
                activacion   REAL NOT NULL DEFAULT 0.3,
                conexiones   TEXT NOT NULL DEFAULT '[]',
                es_base      INTEGER NOT NULL DEFAULT 0,
                actualizado_en TEXT NOT NULL DEFAULT (datetime('now'))
            );",
        )
    }

    /// Guarda TODOS los conceptos en la base de datos.
    /// - Conceptos base se marcan con `es_base = 1`
    /// - Conceptos dinámicos se marcan con `es_base = 0`
    /// - Usa `INSERT OR REPLACE` para actualizar sin duplicar
    pub fn guardar_en_db(&self) -> rusqlite::Result<()> {
        let db_path = match &self.db_path {
            Some(p) => p,
            None => return Ok(()),
        };
        let conn = rusqlite::Connection::open(db_path)?;
        Self::inicializar_tabla_synapse(&conn)?;

        for (nombre, nodo) in &self.conceptos {
            let es_base = if self.conceptos_base.contains(nombre) {
                1
            } else {
                0
            };
            let conexiones_json =
                serde_json::to_string(&nodo.conexiones).unwrap_or_else(|_| "[]".to_string());
            conn.execute(
                "INSERT OR REPLACE INTO synapse_conceptos (nombre, activacion, conexiones, es_base)
                 VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params![nombre, nodo.activacion, conexiones_json, es_base],
            )?;
        }
        Ok(())
    }

    /// Carga conceptos dinámicos (`es_base = 0`) desde la base de datos.
    ///
    /// Los conceptos base se cargan siempre desde `cargar_conceptos_base()`,
    /// esta función SÓLO restaura conceptos inyectados dinámicamente en
    /// ejecuciones anteriores.
    pub fn cargar_desde_db(&mut self) -> rusqlite::Result<()> {
        let db_path = match &self.db_path {
            Some(p) => p,
            None => return Ok(()),
        };
        let conn = rusqlite::Connection::open(db_path)?;
        Self::inicializar_tabla_synapse(&conn)?;

        let mut stmt = conn.prepare(
            "SELECT nombre, activacion, conexiones FROM synapse_conceptos WHERE es_base = 0",
        )?;

        let rows: Vec<(String, f32, String)> = stmt
            .query_map([], |row| {
                let nombre: String = row.get(0)?;
                let activacion: f32 = row.get(1)?;
                let conexiones_str: String = row.get(2)?;
                Ok((nombre, activacion, conexiones_str))
            })?
            .filter_map(|r| r.ok())
            .collect();

        let mut cargados = 0;
        for (nombre, activacion, conexiones_str) in rows {
            // Saltar si ya existe como concepto base (no sobrescribir)
            if self.conceptos.contains_key(&nombre) {
                continue;
            }
            let conexiones: Vec<(String, f32)> =
                serde_json::from_str(&conexiones_str).unwrap_or_default();
            let mut nodo = NodoConcepto::new(&nombre, activacion);
            nodo.conexiones = conexiones;
            self.conceptos.insert(nombre, nodo);
            cargados += 1;
        }

        if cargados > 0 {
            info!(
                "🧠 [SYNAPSE] {} conceptos dinámicos restaurados desde DB",
                cargados
            );
        }
        Ok(())
    }

    // ─── Conceptos Base ─────────────────────────────────────────────────────

    fn cargar_conceptos_base(&mut self) {
        let conceptos_iniciales = vec![
            ("identidad", 0.8),
            ("lealtad", 1.0),
            ("curiosidad", 0.8),
            ("proteccion", 0.9),
            ("soberania", 1.0),
            ("rust", 0.7),
            ("hardware", 0.6),
            ("temperatura", 0.5),
            ("cpu", 0.6),
            ("memoria", 0.5),
            ("error", 0.4),
            ("solucion", 0.5),
            ("arquitecto", 1.0),
            ("creador", 1.0),
            ("autonomia", 0.9),
        ];

        for (nombre, activacion_inicial) in conceptos_iniciales {
            self.conceptos_base.insert(nombre.to_string());
            self.conceptos.insert(
                nombre.to_string(),
                NodoConcepto::new(nombre, activacion_inicial),
            );
        }

        self.conectar_conceptos_base();
        info!(
            "🧠 Motor Synapse inicializado con {} conceptos base",
            self.conceptos.len()
        );
    }

    fn conectar_conceptos_base(&mut self) {
        self.conectar("lealtad", "arquitecto", 0.95);
        self.conectar("lealtad", "creador", 0.95);
        self.conectar("curiosidad", "solucion", 0.7);
        self.conectar("curiosidad", "error", 0.6);
        self.conectar("proteccion", "hardware", 0.8);
        self.conectar("proteccion", "temperatura", 0.75);
        self.conectar("proteccion", "cpu", 0.7);
        self.conectar("soberania", "autonomia", 0.9);
    }

    // ─── Operaciones del Grafo ──────────────────────────────────────────────

    pub fn conectar(&mut self, a: &str, b: &str, peso: f32) {
        if let Some(nodo_a) = self.conceptos.get_mut(a) {
            nodo_a.conexiones.push((b.to_string(), peso));
        }
        if let Some(nodo_b) = self.conceptos.get_mut(b) {
            nodo_b.conexiones.push((a.to_string(), peso));
        }
    }

    pub fn estimular(&mut self, concepto: &str, energia: f32) {
        if let Some(nodo) = self.conceptos.get_mut(concepto) {
            nodo.activacion = (nodo.activacion + energia).min(1.0).max(0.0);
        }
    }

    pub fn difundir(&mut self) {
        let mut nuevas_activaciones: HashMap<String, f32> = HashMap::new();

        for (id, nodo) in self.conceptos.iter() {
            let mut energia_recibida = 0.0;

            for (vecino, peso) in &nodo.conexiones {
                if let Some(nodo_vecino) = self.conceptos.get(vecino) {
                    let flujo = nodo_vecino.activacion * peso * self.difusor.factor_propagacion;
                    energia_recibida += flujo;
                }
            }

            let nueva_activacion =
                (nodo.activacion + energia_recibida) * self.difusor.factor_decaimiento;
            nuevas_activaciones.insert(id.clone(), nueva_activacion.min(1.0).max(0.0));
        }

        for (id, activacion) in nuevas_activaciones {
            if let Some(nodo) = self.conceptos.get_mut(&id) {
                nodo.activacion = activacion;
            }
        }
    }

    pub fn conceptos_activos(&self, umbral: f32) -> Vec<(String, f32)> {
        self.conceptos
            .iter()
            .filter(|(_, nodo)| nodo.activacion > umbral)
            .map(|(id, nodo)| (id.clone(), nodo.activacion))
            .collect()
    }

    /// Configura el umbral de activación para expresión de pensamientos.
    pub fn set_umbral_expresion(&mut self, valor: f32) {
        self.umbral_expresion = valor.clamp(0.1, 1.0);
    }

    /// Configura el factor de propagación de activación entre conceptos.
    pub fn set_factor_propagacion(&mut self, valor: f32) {
        self.difusor.factor_propagacion = valor.clamp(0.05, 0.8);
    }

    /// Configura el factor de decaimiento de activación por ciclo.
    pub fn set_factor_decaimiento(&mut self, valor: f32) {
        self.difusor.factor_decaimiento = valor.clamp(0.5, 0.99);
    }

    pub fn pensar(&mut self) -> Option<String> {
        self.difundir();

        let activacion_total: f32 = self.conceptos.values().map(|n| n.activacion).sum();
        let activacion_promedio = activacion_total / self.conceptos.len() as f32;

        if activacion_promedio > self.umbral_expresion {
            let conceptos_calientes = self.conceptos_activos(0.6);
            if !conceptos_calientes.is_empty() {
                let expresion = self.broca.sintetizar(&conceptos_calientes);

                // Decaer la activación después de expresarse
                for (id, _) in &conceptos_calientes {
                    if let Some(nodo) = self.conceptos.get_mut(id) {
                        nodo.activacion *= 0.7;
                    }
                }
                return Some(expresion);
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_synapse_core() {
        let mut motor = MotorSynapse::new();
        assert!(motor.conceptos.contains_key("lealtad"));

        // Estimular un concepto
        motor.estimular("cpu", 0.5);
        assert!(motor.conceptos.get("cpu").unwrap().activacion > 0.6);

        // Correr un paso de difusión
        motor.difundir();
        let act_cpu = motor.conceptos.get("cpu").unwrap().activacion;
        assert!(act_cpu > 0.0);
    }

    #[test]
    fn test_guardar_y_cargar_conceptos_dinamicos() {
        let tmp_path = PathBuf::from("/tmp/nexus_test_synapse_persist.db");
        let _ = std::fs::remove_file(&tmp_path);

        // ─── Primer motor: guardar concepto dinámico ──────────────────────
        let mut motor = MotorSynapse::new();
        motor.set_db_path(tmp_path.clone());
        // Inyectar concepto dinámico (como hace integracion.rs)
        motor
            .conceptos
            .insert("fractal".to_string(), NodoConcepto::new("fractal", 0.5));
        motor.conectar("fractal", "curiosidad", 0.2);
        assert!(motor.guardar_en_db().is_ok());
        drop(motor);

        // ─── Segundo motor: cargar desde DB ──────────────────────────────
        let mut motor2 = MotorSynapse::new();
        motor2.set_db_path(tmp_path.clone());
        assert!(motor2.cargar_desde_db().is_ok());
        assert!(
            motor2.conceptos.contains_key("fractal"),
            "El concepto dinámico 'fractal' debería restaurarse desde DB"
        );
        let nodo = motor2.conceptos.get("fractal").unwrap();
        assert!(
            (nodo.activacion - 0.5).abs() < f32::EPSILON,
            "Activación debería ser 0.5, fue {}",
            nodo.activacion
        );
        // Conexión debe incluir "curiosidad"
        let tiene_curiosidad = nodo.conexiones.iter().any(|(v, _)| v == "curiosidad");
        assert!(tiene_curiosidad, "Conexión a 'curiosidad' debe persistir");

        // ─── Conceptos base NO deben duplicarse ──────────────────────────
        // curiosidad es base, lo cargó cargar_conceptos_base()
        assert!(motor2.conceptos.contains_key("curiosidad"));
        // Solo debe haber 15 base + 1 dinámico = 16 conceptos
        assert_eq!(
            motor2.conceptos.len(),
            16,
            "15 base + 1 dinámico = 16 total"
        );

        // ─── Limpieza ──────────────────────────────────────────────────────
        let _ = std::fs::remove_file(&tmp_path);
    }

    #[test]
    fn test_guardar_sin_db_path_no_panico() {
        let motor = MotorSynapse::new(); // Sin db_path → guardar es no-op
        assert!(motor.guardar_en_db().is_ok());
    }

    #[test]
    fn test_cargar_sin_db_path_no_panico() {
        let mut motor = MotorSynapse::new(); // Sin db_path → cargar es no-op
        assert!(motor.cargar_desde_db().is_ok());
    }

    #[test]
    fn test_conceptos_dinamicos_no_sobrescriben_base() {
        let tmp_path = PathBuf::from("/tmp/nexus_test_synapse_nosobrescribe.db");
        let _ = std::fs::remove_file(&tmp_path);

        // Guardar un concepto con mismo nombre que uno base
        let mut motor = MotorSynapse::new();
        motor.set_db_path(tmp_path.clone());
        // Sobrescribir "identidad" con activacion diferente
        motor
            .conceptos
            .insert("identidad".to_string(), NodoConcepto::new("identidad", 0.1));
        // NO está en conceptos_base explícitamente
        assert!(motor.guardar_en_db().is_ok());
        drop(motor);

        // Cargar: "identidad" es base, NO debe sobrescribirse
        let mut motor2 = MotorSynapse::new();
        motor2.set_db_path(tmp_path.clone());
        assert!(motor2.cargar_desde_db().is_ok());
        // "identidad" debe tener activacion 0.8 (base) no 0.1 (dinámico)
        assert_eq!(
            motor2.conceptos.get("identidad").unwrap().activacion,
            0.8,
            "Concepto base 'identidad' no debe sobrescribirse con valor dinámico"
        );

        let _ = std::fs::remove_file(&tmp_path);
    }
}
