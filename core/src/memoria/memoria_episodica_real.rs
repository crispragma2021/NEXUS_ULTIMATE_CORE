// ==========================================
// MEMORIA EPISÓDICA REAL - Recuerdos Concretos
// ==========================================
// Recupera interacciones pasadas por fecha,
// hora o tema. Como un humano que recuerda
// "ayer a las 3pm hablamos de X".
// ==========================================

use rusqlite::Connection;
use tracing::info;

pub struct MemoriaEpisodicaReal {
    conn: Connection,
}

impl MemoriaEpisodicaReal {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let db_path = crate::infra::paths::resolve_path("data/intelligence.db");
        let conn = Connection::open(&db_path)?;
        info!("📅 [MEMORIA EPISÓDICA] Recuerdos concretos activos.");
        Ok(Self { conn })
    }

    /// Recupera interacciones de una fecha específica (formato: 2026-05-06)
    pub fn recordar_por_fecha(&self, fecha: &str) -> Vec<(String, String, String)> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT timestamp, entrada, salida FROM memoria_unica 
             WHERE tipo = 'EXPERIENCIA' AND date(timestamp) = ?1 
             ORDER BY timestamp DESC LIMIT 10",
            )
            .unwrap();
        let filas = stmt
            .query_map([fecha], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            })
            .unwrap();
        let mut recuerdos = Vec::new();
        for f in filas.flatten() {
            recuerdos.push(f);
        }
        info!(
            "📅 [MEMORIA EPISÓDICA] {} recuerdos del día {}.",
            recuerdos.len(),
            fecha
        );
        recuerdos
    }

    /// Recupera las últimas N interacciones con timestamp
    pub fn recordar_recientes(&self, limite: usize) -> Vec<(String, String, String)> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT timestamp, entrada, salida FROM memoria_unica 
             WHERE tipo = 'EXPERIENCIA' 
             ORDER BY timestamp DESC LIMIT ?1",
            )
            .unwrap();
        let filas = stmt
            .query_map([limite], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            })
            .unwrap();
        let mut recuerdos = Vec::new();
        for f in filas.flatten() {
            recuerdos.push(f);
        }
        recuerdos.reverse();
        info!(
            "📅 [MEMORIA EPISÓDICA] {} recuerdos recientes.",
            recuerdos.len()
        );
        recuerdos
    }
}
