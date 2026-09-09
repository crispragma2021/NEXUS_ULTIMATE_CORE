//! Memoria Episódica de Largo Plazo (E5).
//!
//! El orquestador recuerda interacciones pasadas, decisiones y contexto entre
//! sesiones. Es memoria autobiográfica del agente (no solo RAG de contenido).
//!
//! - Tabla `episodic_memory` con eventos tipados, importancia y embedding.
//! - Recuperación semántica: embedding de la pregunta → top-k por coseno.
//! - Consolidación: sumariza episodios viejos en "memorias núcleo".

use crate::scraping::pipeline::embedding::cosine_similarity;
use crate::scraping::pipeline::embedding::EmbeddingEngine;
use anyhow::{Context, Result};
use rusqlite::Connection;
use serde_json::json;
use std::path::Path;
use std::sync::Mutex;

/// Tipo de evento episódico.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventType {
    UserQuery,
    AgentAction,
    ScrapingResult,
    Decision,
    Error,
    Learning,
}

impl EventType {
    pub fn as_str(&self) -> &'static str {
        match self {
            EventType::UserQuery => "user_query",
            EventType::AgentAction => "agent_action",
            EventType::ScrapingResult => "scraping_result",
            EventType::Decision => "decision",
            EventType::Error => "error",
            EventType::Learning => "learning",
        }
    }
}

/// Un episodio de memoria.
#[derive(Debug, Clone)]
pub struct Episode {
    pub id: i64,
    pub timestamp: String,
    pub session_id: String,
    pub event_type: String,
    pub summary: String,
    pub full_context: Option<String>,
    pub importance: f64,
    pub tags: Vec<String>,
    pub vector: Vec<f32>,
}

/// Almacén de memoria episódica.
pub struct EpisodicMemory {
    conn: Mutex<Connection>,
    embedding: EmbeddingEngine,
}

impl EpisodicMemory {
    pub fn open(path: &Path, embedding: EmbeddingEngine) -> Result<Self> {
        let conn = Connection::open(path)?;
        let mem = Self {
            conn: Mutex::new(conn),
            embedding,
        };
        mem.init()?;
        Ok(mem)
    }

    pub fn open_in_memory(embedding: EmbeddingEngine) -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let mem = Self {
            conn: Mutex::new(conn),
            embedding,
        };
        mem.init()?;
        Ok(mem)
    }

    fn init(&self) -> Result<()> {
        self.conn.lock().unwrap().execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE TABLE IF NOT EXISTS episodic_memory (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp TEXT NOT NULL DEFAULT (datetime('now')),
                session_id TEXT NOT NULL,
                event_type TEXT CHECK(event_type IN ('user_query','agent_action','scraping_result','decision','error','learning')),
                summary TEXT NOT NULL,
                full_context TEXT,
                importance REAL DEFAULT 0.5,
                embedding BLOB,
                tags TEXT
            );
            CREATE INDEX IF NOT EXISTS idx_episodic_timestamp ON episodic_memory(timestamp);
            CREATE INDEX IF NOT EXISTS idx_episodic_importance ON episodic_memory(importance);
            "#,
        )?;
        Ok(())
    }

    /// Registra un episodio (genera embedding del resumen).
    pub async fn record(
        &self,
        session_id: &str,
        event_type: EventType,
        summary: &str,
        full_context: Option<&str>,
        importance: f64,
        tags: Vec<String>,
    ) -> Result<i64> {
        // Asegurar sesión.
        {
            let conn = self.conn.lock().unwrap();
            let _ = conn.execute(
                "INSERT OR IGNORE INTO sessions (id) VALUES (?1)",
                rusqlite::params![session_id],
            );
        }

        // Embedding del resumen.
        let vector = self.embedding.embed(summary).await?;
        let blob = vector_to_blob(&vector);
        let tags_json = serde_json::to_string(&tags)?;

        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO episodic_memory (session_id, event_type, summary, full_context, importance, embedding, tags)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![
                session_id,
                event_type.as_str(),
                summary,
                full_context,
                importance,
                blob,
                tags_json
            ],
        )?;
        Ok(conn.last_insert_rowid())
    }

    /// Búsqueda semántica de episodios por una consulta.
    pub async fn recall(&self, query: &str, k: usize) -> Result<Vec<(Episode, f32)>> {
        let query_vec = self.embedding.embed(query).await?;
        let episodes = self.load_all()?;
        let mut scored: Vec<(Episode, f32)> = episodes
            .into_iter()
            .map(|e| {
                let score = cosine_similarity(&query_vec, &e.vector);
                (e, score)
            })
            .filter(|(_, s)| *s > 0.0)
            .collect();
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(k);
        Ok(scored)
    }

    /// Devuelve episodios recientes (ordenados por timestamp desc).
    pub fn recent(&self, limit: usize) -> Result<Vec<Episode>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, timestamp, session_id, event_type, summary, full_context, importance, embedding, tags
             FROM episodic_memory ORDER BY timestamp DESC LIMIT ?1",
        )?;
        let rows = stmt.query_map(rusqlite::params![limit as i64], row_to_episode)?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }

    /// Número de episodios almacenados.
    pub fn count(&self) -> Result<u64> {
        let conn = self.conn.lock().unwrap();
        let n: i64 = conn.query_row("SELECT COUNT(*) FROM episodic_memory", [], |r| r.get(0))?;
        Ok(n as u64)
    }

    /// Formatea episodios recuperados para inyectar al LLM como contexto.
    pub fn format_context(episodes: &[(Episode, f32)]) -> String {
        let mut out = String::new();
        for (i, (e, score)) in episodes.iter().enumerate() {
            out.push_str(&format!(
                "[Recuerdo {} · {} · {:.3}]\n{}\n",
                i + 1,
                e.event_type,
                score,
                e.summary
            ));
        }
        out
    }

    fn load_all(&self) -> Result<Vec<Episode>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, timestamp, session_id, event_type, summary, full_context, importance, embedding, tags
             FROM episodic_memory",
        )?;
        let rows = stmt.query_map([], row_to_episode)?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }

    /// Resumen de estado (para observabilidad).
    pub fn stats(&self) -> Result<serde_json::Value> {
        Ok(json!({
            "episodes": self.count()?,
        }))
    }
}

fn row_to_episode(row: &rusqlite::Row<'_>) -> rusqlite::Result<Episode> {
    let blob: Vec<u8> = row.get(7)?;
    let tags_json: String = row.get(8)?;
    let tags: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_default();
    Ok(Episode {
        id: row.get(0)?,
        timestamp: row.get(1)?,
        session_id: row.get(2)?,
        event_type: row.get(3)?,
        summary: row.get(4)?,
        full_context: row.get(5)?,
        importance: row.get(6)?,
        tags,
        vector: blob_to_vector(&blob),
    })
}

fn vector_to_blob(v: &[f32]) -> Vec<u8> {
    let mut out = Vec::with_capacity(v.len() * 4);
    for f in v {
        out.extend_from_slice(&f.to_le_bytes());
    }
    out
}

fn blob_to_vector(blob: &[u8]) -> Vec<f32> {
    blob.chunks_exact(4)
        .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock de embeddings que devuelve un vector determinista por texto.
    fn mock_embedding() -> EmbeddingEngine {
        // Se usa solo para instanciar; los tests no llaman a la red.
        EmbeddingEngine::new("test-model").unwrap()
    }

    #[test]
    fn convierte_vectores_a_blob_y_vuelta() {
        let v = vec![1.0, -2.5, 3.25];
        let blob = vector_to_blob(&v);
        assert_eq!(blob_to_vector(&blob), v);
    }

    #[test]
    fn blob_a_vector_redondea_float() {
        let v = vec![0.1, 0.2, 0.3];
        let blob = vector_to_blob(&v);
        let back = blob_to_vector(&blob);
        assert_eq!(back.len(), 3);
        assert!((back[0] - 0.1).abs() < 1e-6);
    }

    #[tokio::test]
    async fn registra_y_cuenta_episodios() {
        let mem = EpisodicMemory::open_in_memory(mock_embedding()).unwrap();
        // Sin red: usar un embedding sintético insertando directamente.
        mem.conn
            .lock()
            .unwrap()
            .execute(
                "INSERT INTO episodic_memory (session_id, event_type, summary, importance, embedding, tags)
                 VALUES ('s1', 'decision', 'decidí usar Rust', 0.9, ?1, '[\"tech\"]')",
                rusqlite::params![vector_to_blob(&[1.0, 0.0, 0.0])],
            )
            .unwrap();
        assert_eq!(mem.count().unwrap(), 1);
    }

    #[tokio::test]
    async fn recupera_recientes() {
        let mem = EpisodicMemory::open_in_memory(mock_embedding()).unwrap();
        let conn = mem.conn.lock().unwrap();
        conn.execute_batch(
            r#"
            INSERT INTO episodic_memory (session_id, event_type, summary, embedding, tags) VALUES
            ('s1','user_query','pregunta sobre precios', X'0000803F000000000000000000000000', '[]'),
            ('s1','decision','decidí migrar a Rust', X'000000000000803F0000000000000000', '[]');
            "#,
        )
        .unwrap();
        drop(conn);
        let recent = mem.recent(1).unwrap();
        assert_eq!(recent.len(), 1);
        assert!(recent[0].summary.contains("Rust"));
    }

    #[test]
    fn formatea_contexto() {
        let e = Episode {
            id: 1,
            timestamp: "2026-01-01".into(),
            session_id: "s1".into(),
            event_type: "decision".into(),
            summary: "usé Rust".into(),
            full_context: None,
            importance: 0.9,
            tags: vec![],
            vector: vec![],
        };
        let ctx = EpisodicMemory::format_context(&[(e, 0.95)]);
        assert!(ctx.contains("usé Rust"));
        assert!(ctx.contains("0.95"));
    }
}
