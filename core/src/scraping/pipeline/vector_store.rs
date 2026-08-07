//! Base vectorial local del Cerebro Auto-Creciente (E2).
//!
//! Almacena vectores de embedding en SQLite (tabla `knowledge_chunks`) y
//! realiza búsqueda por similitud coseno en memoria. Suficiente para decenas
//! de miles de chunks (típico de un RAG personal). Si se supera esa escala,
//! la interfaz permite migrar a Qdrant sin cambiar el resto.

use crate::scraping::pipeline::embedding::cosine_similarity;
use anyhow::{Context, Result};
use rusqlite::Connection;
use std::path::Path;
use std::sync::Mutex;

/// Chunk de conocimiento indexado.
#[derive(Debug, Clone)]
pub struct KnowledgeChunk {
    pub id: i64,
    pub task_id: String,
    pub project_id: String,
    pub source_url: String,
    pub text: String,
    /// Vector normalizado.
    pub vector: Vec<f32>,
    pub created_at: String,
}

/// Resultado de una búsqueda de similitud.
#[derive(Debug, Clone)]
pub struct SearchHit {
    pub chunk: KnowledgeChunk,
    pub score: f32,
}

/// Almacén vectorial local.
pub struct VectorStore {
    conn: Mutex<Connection>,
}

impl VectorStore {
    pub fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)?;
        let store = Self {
            conn: Mutex::new(conn),
        };
        store.init()?;
        Ok(store)
    }

    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let store = Self {
            conn: Mutex::new(conn),
        };
        store.init()?;
        Ok(store)
    }

    fn init(&self) -> Result<()> {
        {
            let conn = self.conn.lock().unwrap();
            conn.execute_batch(
                r#"
                CREATE TABLE IF NOT EXISTS knowledge_chunks (
                    id          INTEGER PRIMARY KEY AUTOINCREMENT,
                    task_id     TEXT NOT NULL,
                    project_id  TEXT NOT NULL DEFAULT '',
                    source_url  TEXT NOT NULL DEFAULT '',
                    text        TEXT NOT NULL,
                    vector      BLOB NOT NULL,
                    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
                );
                CREATE INDEX IF NOT EXISTS idx_knowledge_task ON knowledge_chunks(task_id);
                CREATE INDEX IF NOT EXISTS idx_knowledge_created ON knowledge_chunks(created_at);
                CREATE INDEX IF NOT EXISTS idx_knowledge_project ON knowledge_chunks(project_id);
                "#,
            )?;
        }

        // Migración no destructiva para DBs existentes: añadir project_id si falta.
        let conn = self.conn.lock().unwrap();
        let has_col: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('knowledge_chunks') WHERE name='project_id'",
                [],
                |r| r.get(0),
            )
            .unwrap_or(0);
        if has_col == 0 {
            conn.execute_batch(
                "ALTER TABLE knowledge_chunks ADD COLUMN project_id TEXT NOT NULL DEFAULT '';",
            )?;
        }
        Ok(())
    }

    /// Serializa un vector f32 a BLOB (bytes little-endian).
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

    /// Inserta un chunk de conocimiento (API original, sin proyecto).
    pub fn insert(
        &self,
        task_id: &str,
        source_url: &str,
        text: &str,
        vector: &[f32],
    ) -> Result<i64> {
        self.insert_in_project(task_id, "", source_url, text, vector)
    }

    /// Inserta un chunk de conocimiento asociado a un proyecto (Scope isolation).
    pub fn insert_in_project(
        &self,
        task_id: &str,
        project_id: &str,
        source_url: &str,
        text: &str,
        vector: &[f32],
    ) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO knowledge_chunks (task_id, project_id, source_url, text, vector)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![
                task_id,
                project_id,
                source_url,
                text,
                Self::vector_to_blob(vector)
            ],
        )?;
        Ok(conn.last_insert_rowid())
    }

    /// Carga todos los chunks (para búsqueda por fuerza bruta).
    fn load_all(&self) -> Result<Vec<KnowledgeChunk>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, task_id, project_id, source_url, text, vector, created_at
             FROM knowledge_chunks",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, String>(4)?,
                r.get::<_, Vec<u8>>(5)?,
                r.get::<_, String>(6)?,
            ))
        })?;
        let mut out = Vec::new();
        for row in rows {
            let (id, task_id, project_id, source_url, text, blob, created_at) = row?;
            out.push(KnowledgeChunk {
                id,
                task_id,
                project_id,
                source_url,
                text,
                vector: Self::blob_to_vector(&blob),
                created_at,
            });
        }
        Ok(out)
    }

    /// Búsqueda top-k por similitud coseno (fuerza bruta, suficiente para
    /// decenas de miles de chunks en RAM).
    pub fn search(&self, query_vector: &[f32], k: usize) -> Result<Vec<SearchHit>> {
        let chunks = self.load_all()?;
        let mut hits: Vec<SearchHit> = chunks
            .iter()
            .map(|c| SearchHit {
                chunk: c.clone(),
                score: cosine_similarity(query_vector, &c.vector),
            })
            .filter(|h| h.score > 0.0)
            .collect();
        hits.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        hits.truncate(k);
        Ok(hits)
    }

    /// Búsqueda top-k limitada a UN proyecto (Scope isolation).
    ///
    /// Esta es la pieza que conecta el ScopeMapper con el RAG: solo se
    /// consideran los chunks del proyecto indicado, nunca de otros.
    pub fn search_in_project(
        &self,
        query_vector: &[f32],
        project_id: &str,
        k: usize,
    ) -> Result<Vec<SearchHit>> {
        let chunks = self.load_all()?;
        let mut hits: Vec<SearchHit> = chunks
            .iter()
            .filter(|c| c.project_id == project_id)
            .map(|c| SearchHit {
                chunk: c.clone(),
                score: cosine_similarity(query_vector, &c.vector),
            })
            .filter(|h| h.score > 0.0)
            .collect();
        hits.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        hits.truncate(k);
        Ok(hits)
    }

    /// Número de chunks almacenados.
    pub fn count(&self) -> Result<u64> {
        let conn = self.conn.lock().unwrap();
        let n: i64 = conn.query_row("SELECT COUNT(*) FROM knowledge_chunks", [], |r| r.get(0))?;
        Ok(n as u64)
    }

    /// Elimina chunks de una tarea específica.
    pub fn delete_by_task(&self, task_id: &str) -> Result<u64> {
        let conn = self.conn.lock().unwrap();
        let n = conn.execute(
            "DELETE FROM knowledge_chunks WHERE task_id = ?1",
            rusqlite::params![task_id],
        )?;
        Ok(n as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_store() -> VectorStore {
        let store = VectorStore::open_in_memory().unwrap();
        // Vectores de 4 dims normalizados.
        store
            .insert("t1", "https://a.com", "gatos", &[1.0, 0.0, 0.0, 0.0])
            .unwrap();
        store
            .insert("t2", "https://b.com", "perros", &[0.0, 1.0, 0.0, 0.0])
            .unwrap();
        store
            .insert("t3", "https://c.com", "felinos", &[0.9, 0.1, 0.0, 0.0])
            .unwrap();
        store
    }

    #[test]
    fn inserta_y_cuenta_chunks() {
        let store = VectorStore::open_in_memory().unwrap();
        store.insert("t1", "https://x", "texto", &[0.0, 1.0]).unwrap();
        assert_eq!(store.count().unwrap(), 1);
    }

    #[test]
    fn busca_por_similitud_top_k() {
        let store = sample_store();
        // Consulta = "gatos" → el chunk más cercano es "gatos" (t1).
        let query = [1.0, 0.0, 0.0, 0.0];
        let hits = store.search(&query, 2).unwrap();
        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0].chunk.task_id, "t1");
        assert!(hits[0].score > hits[1].score);
    }

    #[test]
    fn borra_por_tarea() {
        let store = sample_store();
        let n = store.delete_by_task("t1").unwrap();
        assert_eq!(n, 1);
        assert_eq!(store.count().unwrap(), 2);
    }

    // ── Scope isolation por proyecto (conexión ScopeMapper ↔ RAG) ──

    #[test]
    fn search_in_project_solo_devuelve_ese_proyecto() {
        let store = VectorStore::open_in_memory().unwrap();
        // Proyecto "trader" y "telegram" con vectores distintos.
        store
            .insert_in_project("t1", "trader", "https://trader.com", "precios de acciones", &[1.0, 0.0, 0.0, 0.0])
            .unwrap();
        store
            .insert_in_project("t2", "trader", "https://trader.com/2", "margenes de trading", &[0.8, 0.2, 0.0, 0.0])
            .unwrap();
        store
            .insert_in_project("t3", "telegram", "https://tg.com", "notificaciones de bots", &[0.0, 0.0, 1.0, 0.0])
            .unwrap();

        // Búsqueda sobre "trader" → NO debe devolver el chunk de telegram.
        let hits = store.search_in_project(&[1.0, 0.0, 0.0, 0.0], "trader", 10).unwrap();
        assert_eq!(hits.len(), 2);
        assert!(hits.iter().all(|h| h.chunk.project_id == "trader"));

        // Búsqueda sobre "telegram" → solo el chunk de telegram.
        let hits_tg = store.search_in_project(&[0.0, 0.0, 1.0, 0.0], "telegram", 10).unwrap();
        assert_eq!(hits_tg.len(), 1);
        assert_eq!(hits_tg[0].chunk.task_id, "t3");
    }

    #[test]
    fn insert_original_sigue_sin_proyecto() {
        let store = VectorStore::open_in_memory().unwrap();
        // La API original `insert` usa project_id vacío (retrocompatibilidad).
        store.insert("t1", "https://x", "texto", &[0.0, 1.0]).unwrap();
        let all = store.load_all().unwrap();
        assert_eq!(all[0].project_id, "");
    }
}
