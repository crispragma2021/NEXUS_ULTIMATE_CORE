// ==========================================
// CHECKPOINT — Ejecución Durable (Patrón LangGraph absorbido)
// ==========================================
// Permite pausar/reanudar pipelines de larga duración.
// Si el daemon muere, la ejecución reanuda desde la última etapa
// completada — no desde cero. Asimilado de LangGraph (checkpointing)
// el 2026-08-01 por orden del Arquitecto: solo lo bueno, lo que sirve.
//
// Ciclo de vida:
//   iniciar()  →  guardar_etapa(1..N)  →  completar() | marcar_fallido()
//   Muerte del proceso en cualquier punto → resumir() devuelve el estado
//   exacto desde la última etapa guardada.
// ==========================================

use anyhow::{Context, Result};
use chrono::Utc;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Ruta a la base de datos unificada (misma convención que memoria_operativa)
const NEXUS_MEMORIA_DB: &str = "data/nexus_memoria.db";

/// Nombre de la tabla de checkpoints
const CHECKPOINTS_TABLA: &str = "checkpoints";

// ---------------------------------------------------------------------------
// Tipos
// ---------------------------------------------------------------------------

/// Estado del ciclo de vida de una ejecución durable
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EstadoEjecucion {
    EnProgreso,
    Completado,
    Fallido,
}

impl EstadoEjecucion {
    fn a_texto(&self) -> &'static str {
        match self {
            EstadoEjecucion::EnProgreso => "en_progreso",
            EstadoEjecucion::Completado => "completado",
            EstadoEjecucion::Fallido => "fallido",
        }
    }

    fn desde_texto(s: &str) -> EstadoEjecucion {
        match s {
            "completado" => EstadoEjecucion::Completado,
            "fallido" => EstadoEjecucion::Fallido,
            _ => EstadoEjecucion::EnProgreso,
        }
    }
}

/// Un checkpoint: fotografía del estado de un pipeline en una etapa dada
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    pub id: String,
    pub pipeline: String,
    /// Última etapa completada (0 = recién iniciado)
    pub etapa: u32,
    pub total_etapas: u32,
    pub estado: EstadoEjecucion,
    /// Estado arbitrario del pipeline (cualquier JSON serializable)
    pub payload: serde_json::Value,
    pub creado_en: i64,
    pub actualizado_en: i64,
}

// ---------------------------------------------------------------------------
// Conexión
// ---------------------------------------------------------------------------

/// Abre la conexión a la base unificada (WAL + busy_timeout, misma
/// configuración que el resto de órganos de memoria)
fn abrir_conexion_en(ruta: &str) -> Result<Connection> {
    let conn = Connection::open(ruta)
        .with_context(|| format!("No se pudo abrir la base de datos en {}", ruta))?;
    conn.busy_timeout(std::time::Duration::from_millis(5000))?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.execute_batch(&format!(
        "CREATE TABLE IF NOT EXISTS {CHECKPOINTS_TABLA} (
            id TEXT PRIMARY KEY,
            pipeline TEXT NOT NULL,
            etapa INTEGER NOT NULL DEFAULT 0,
            total_etapas INTEGER NOT NULL DEFAULT 0,
            estado TEXT NOT NULL DEFAULT 'en_progreso',
            payload TEXT NOT NULL DEFAULT '{{}}',
            creado_en INTEGER NOT NULL,
            actualizado_en INTEGER NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_checkpoints_pipeline
            ON {CHECKPOINTS_TABLA}(pipeline, actualizado_en DESC);"
    ))?;
    Ok(conn)
}

fn abrir_conexion() -> Result<Connection> {
    abrir_conexion_en(NEXUS_MEMORIA_DB)
}

// ---------------------------------------------------------------------------
// API pública
// ---------------------------------------------------------------------------

impl Checkpoint {
    /// Inicia una ejecución durable nueva. Si existe una ejecución previa
    /// en progreso del mismo pipeline, la abandona (se empieza limpio).
    pub fn iniciar(pipeline: &str, total_etapas: u32, payload: serde_json::Value) -> Result<Self> {
        let conn = abrir_conexion()?;
        Self::iniciar_en(&conn, pipeline, total_etapas, payload)
    }

    fn iniciar_en(
        conn: &Connection,
        pipeline: &str,
        total_etapas: u32,
        payload: serde_json::Value,
    ) -> Result<Self> {
        // Limpiar ejecuciones huérfanas del mismo pipeline (muertes previas)
        conn.execute(
            &format!(
                "DELETE FROM {CHECKPOINTS_TABLA} WHERE pipeline = ?1 AND estado = 'en_progreso'"
            ),
            params![pipeline],
        )?;

        let ahora = Utc::now().timestamp();
        let cp = Checkpoint {
            id: Uuid::new_v4().to_string(),
            pipeline: pipeline.to_string(),
            etapa: 0,
            total_etapas,
            estado: EstadoEjecucion::EnProgreso,
            payload,
            creado_en: ahora,
            actualizado_en: ahora,
        };
        registrar(conn, &cp)?;
        info!(pipeline = %cp.pipeline, id = %cp.id, "⚡ Checkpoint iniciado");
        Ok(cp)
    }

    /// Guarda el progreso tras completar la etapa dada
    pub fn guardar_etapa(&self, etapa: u32, payload: serde_json::Value) -> Result<()> {
        let conn = abrir_conexion()?;
        conn.execute(
            &format!(
                "UPDATE {CHECKPOINTS_TABLA} SET etapa = ?1, payload = ?2, actualizado_en = ?3 WHERE id = ?4"
            ),
            params![etapa, payload.to_string(), Utc::now().timestamp(), self.id],
        )?;
        debug!(pipeline = %self.pipeline, etapa, "💾 Checkpoint guardado");
        Ok(())
    }

    /// Marca la ejecución como completada
    pub fn completar(&self) -> Result<()> {
        let conn = abrir_conexion()?;
        conn.execute(
            &format!(
                "UPDATE {CHECKPOINTS_TABLA} SET estado = 'completado', actualizado_en = ?1 WHERE id = ?2"
            ),
            params![Utc::now().timestamp(), self.id],
        )?;
        info!(pipeline = %self.pipeline, "✅ Ejecución durable completada");
        Ok(())
    }

    /// Marca la ejecución como fallida (para diagnóstico posterior)
    pub fn marcar_fallido(&self) -> Result<()> {
        let conn = abrir_conexion()?;
        conn.execute(
            &format!(
                "UPDATE {CHECKPOINTS_TABLA} SET estado = 'fallido', actualizado_en = ?1 WHERE id = ?2"
            ),
            params![Utc::now().timestamp(), self.id],
        )?;
        warn!(pipeline = %self.pipeline, "⚠️ Ejecución durable marcada como fallida");
        Ok(())
    }

    /// Recupera la ejecución en progreso más reciente de un pipeline.
    /// Devuelve None si no hay nada que reanudar (ya completada o inexistente).
    pub fn resumir(pipeline: &str) -> Result<Option<Self>> {
        let conn = abrir_conexion()?;
        Self::resumir_en(&conn, pipeline)
    }

    fn resumir_en(conn: &Connection, pipeline: &str) -> Result<Option<Self>> {
        let mut stmt = conn.prepare(&format!(
            "SELECT id, pipeline, etapa, total_etapas, estado, payload, creado_en, actualizado_en
             FROM {CHECKPOINTS_TABLA}
             WHERE pipeline = ?1 AND estado = 'en_progreso'
             ORDER BY actualizado_en DESC LIMIT 1"
        ))?;
        let mut filas = stmt.query_map(params![pipeline], |f| {
            Ok(Checkpoint {
                id: f.get(0)?,
                pipeline: f.get(1)?,
                etapa: f.get(2)?,
                total_etapas: f.get(3)?,
                estado: EstadoEjecucion::desde_texto(&f.get::<_, String>(4)?),
                payload: serde_json::from_str(&f.get::<_, String>(5)?)
                    .unwrap_or(serde_json::Value::Null),
                creado_en: f.get(6)?,
                actualizado_en: f.get(7)?,
            })
        })?;

        match filas.next() {
            Some(Ok(cp)) => {
                info!(pipeline = %cp.pipeline, etapa = cp.etapa, "♻️ Checkpoint resumido desde la etapa {}", cp.etapa);
                Ok(Some(cp))
            }
            Some(Err(e)) => Err(e.into()),
            None => Ok(None),
        }
    }

    /// Censo de todas las ejecuciones registradas (para telemetría)
    pub fn censo() -> Result<Vec<Self>> {
        let conn = abrir_conexion()?;
        let mut stmt = conn.prepare(&format!(
            "SELECT id, pipeline, etapa, total_etapas, estado, payload, creado_en, actualizado_en
             FROM {CHECKPOINTS_TABLA} ORDER BY actualizado_en DESC"
        ))?;
        let filas = stmt.query_map([], |f| {
            Ok(Checkpoint {
                id: f.get(0)?,
                pipeline: f.get(1)?,
                etapa: f.get(2)?,
                total_etapas: f.get(3)?,
                estado: EstadoEjecucion::desde_texto(&f.get::<_, String>(4)?),
                payload: serde_json::from_str(&f.get::<_, String>(5)?)
                    .unwrap_or(serde_json::Value::Null),
                creado_en: f.get(6)?,
                actualizado_en: f.get(7)?,
            })
        })?;
        Ok(filas.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    /// Elimina todas las ejecuciones en progreso de un pipeline (limpieza)
    pub fn abandonar(pipeline: &str) -> Result<usize> {
        let conn = abrir_conexion()?;
        let n = conn.execute(
            &format!("DELETE FROM {CHECKPOINTS_TABLA} WHERE pipeline = ?1"),
            params![pipeline],
        )?;
        info!(
            pipeline,
            "🧹 Checkpoints de '{}' abandonados ({})", pipeline, n
        );
        Ok(n)
    }
}

fn registrar(conn: &Connection, cp: &Checkpoint) -> Result<()> {
    conn.execute(
        &format!(
            "INSERT OR REPLACE INTO {CHECKPOINTS_TABLA}
             (id, pipeline, etapa, total_etapas, estado, payload, creado_en, actualizado_en)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)"
        ),
        params![
            cp.id,
            cp.pipeline,
            cp.etapa,
            cp.total_etapas,
            cp.estado.a_texto(),
            cp.payload.to_string(),
            cp.creado_en,
            cp.actualizado_en,
        ],
    )?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Pruebas — incluyen simulación de muerte del proceso
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn conexion_temporal() -> Result<Connection> {
        let ruta = std::env::temp_dir().join(format!("checkpoint_test_{}.db", Uuid::new_v4()));
        let conn = abrir_conexion_en(ruta.to_str().unwrap())?;
        Ok(conn)
    }

    /// Abre una conexión sobre UNA RUTA FIJA: permite simular la muerte
    /// (drop de conexión) y reabrir la MISMA base en el "proceso 2"
    fn conexion_en_ruta_fija(ruta: &str) -> Result<Connection> {
        abrir_conexion_en(ruta)
    }

    /// Ciclo completo: iniciar → avanzar → "morir" (dropear conexión) →
    /// resumir en proceso nuevo → completar → ya no hay nada que reanudar
    #[test]
    fn ciclo_vida_con_muerte_simulada() -> Result<()> {
        // Misma base para ambos "procesos" (simula persistencia en disco)
        let ruta = std::env::temp_dir().join(format!("checkpoint_muerte_{}.db", Uuid::new_v4()));
        let ruta = ruta.to_str().unwrap().to_string();

        // Proceso 1: inicia y llega a la etapa 6
        let conn = conexion_en_ruta_fija(&ruta)?;
        let cp = Checkpoint::iniciar_en(
            &conn,
            "goi_14_etapas",
            14,
            serde_json::json!({"prompt": "analiza"}),
        )?;
        for etapa in 1..=6 {
            cp.guardar_etapa_en(&conn, etapa, serde_json::json!({"etapa": etapa}))?;
        }
        drop(conn); // 💀 Muerte simulada: conexión cerrada, proceso "muere"

        // Proceso 2: reabre LA MISMA base y reanuda — debe continuar desde la etapa 6
        let conn = conexion_en_ruta_fija(&ruta)?;
        let reanudado = Checkpoint::resumir_en(&conn, "goi_14_etapas")?
            .expect("Debe existir un checkpoint en progreso");
        assert_eq!(reanudado.etapa, 6, "Debe reanudar desde la etapa 6");
        assert_eq!(reanudado.pipeline, "goi_14_etapas");
        assert_eq!(reanudado.payload["etapa"], 6, "El payload debe sobrevivir");

        // Termina el trabajo pendiente (7..=14)
        for etapa in 7..=14 {
            reanudado.guardar_etapa_en(&conn, etapa, serde_json::json!({"etapa": etapa}))?;
        }
        reanudado.completar_en(&conn)?;

        // Verificación: ya no hay nada que reanudar
        let resto = Checkpoint::resumir_en(&conn, "goi_14_etapas")?;
        assert!(
            resto.is_none(),
            "No debe haber ejecuciones en progreso tras completar"
        );

        // El censo conserva el historial
        let censo = Checkpoint::censo_en(&conn)?;
        assert_eq!(censo.len(), 1);
        assert_eq!(censo[0].estado, EstadoEjecucion::Completado);
        Ok(())
    }

    /// Dos ejecuciones huérfanas + una nueva: iniciar() debe limpiar lo viejo
    #[test]
    fn iniciar_abandona_ejecuciones_huérfanas() -> Result<()> {
        let conn = conexion_temporal()?;
        let a = Checkpoint::iniciar_en(&conn, "p", 5, serde_json::json!({}))?;
        a.guardar_etapa_en(&conn, 2, serde_json::json!({}))?;
        let b = Checkpoint::iniciar_en(&conn, "p", 5, serde_json::json!({}))?;
        b.guardar_etapa_en(&conn, 3, serde_json::json!({}))?;

        // Tercera ejecución: debe haber limpiado las dos huérfanas
        let c = Checkpoint::iniciar_en(&conn, "p", 5, serde_json::json!({}))?;
        c.guardar_etapa_en(&conn, 1, serde_json::json!({}))?;
        let reanudado = Checkpoint::resumir_en(&conn, "p")?;
        assert!(reanudado.is_some());
        assert_eq!(
            reanudado.unwrap().etapa,
            1,
            "Solo debe quedar la ejecución más nueva"
        );
        Ok(())
    }

    // Helpers de test: variantes que reciben la conexión (evitan reabrir el archivo temporal)
    impl Checkpoint {
        fn guardar_etapa_en(
            &self,
            conn: &Connection,
            etapa: u32,
            payload: serde_json::Value,
        ) -> Result<()> {
            conn.execute(
                &format!(
                    "UPDATE {CHECKPOINTS_TABLA} SET etapa = ?1, payload = ?2, actualizado_en = ?3 WHERE id = ?4"
                ),
                params![etapa, payload.to_string(), Utc::now().timestamp(), self.id],
            )?;
            Ok(())
        }

        fn completar_en(&self, conn: &Connection) -> Result<()> {
            conn.execute(
                &format!(
                    "UPDATE {CHECKPOINTS_TABLA} SET estado = 'completado', actualizado_en = ?1 WHERE id = ?2"
                ),
                params![Utc::now().timestamp(), self.id],
            )?;
            Ok(())
        }

        fn censo_en(conn: &Connection) -> Result<Vec<Self>> {
            let mut stmt = conn.prepare(&format!(
                "SELECT id, pipeline, etapa, total_etapas, estado, payload, creado_en, actualizado_en
                 FROM {CHECKPOINTS_TABLA} ORDER BY actualizado_en DESC"
            ))?;
            let filas = stmt.query_map([], |f| {
                Ok(Checkpoint {
                    id: f.get(0)?,
                    pipeline: f.get(1)?,
                    etapa: f.get(2)?,
                    total_etapas: f.get(3)?,
                    estado: EstadoEjecucion::desde_texto(&f.get::<_, String>(4)?),
                    payload: serde_json::from_str(&f.get::<_, String>(5)?)
                        .unwrap_or(serde_json::Value::Null),
                    creado_en: f.get(6)?,
                    actualizado_en: f.get(7)?,
                })
            })?;
            Ok(filas.collect::<rusqlite::Result<Vec<_>>>()?)
        }
    }
}
