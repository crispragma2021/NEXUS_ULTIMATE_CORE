// ==========================================
// ⚡ CACHE SEMÁNTICO — Eficiencia de Token Cero
// ==========================================
// Reduce latencia y costos guardando respuestas
// basadas en el hash del prompt y estado del mercado.
// ==========================================

use crate::emociones::ocean::Ocean;
use rusqlite::{params, Connection};
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{debug, error, info};

pub struct CacheSemantico {
    conn: Connection,
    ocean: Option<Arc<Ocean>>,
}

impl CacheSemantico {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let db_path = crate::infra::paths::resolve_path("data/cache_semantico.db");
        let conn = Connection::open(&db_path)?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS cache (
                hash_key TEXT PRIMARY KEY,
                respuesta TEXT NOT NULL,
                timestamp INTEGER NOT NULL,
                ttl INTEGER NOT NULL
            )",
            [],
        )?;

        info!("⚡ Cache Semántico inicializado.");
        Ok(Self { conn, ocean: None })
    }

    /// Vincula el órgano Ocean para permitir búsquedas vectoriales
    pub fn vincular_ocean(&mut self, ocean: Arc<Ocean>) {
        self.ocean = Some(ocean);
        info!("🔗 Ocean vinculado a la Caché Semántica.");
    }

    /// Busca una respuesta en caché usando una clave compuesta o similitud vectorial
    pub async fn buscar(&self, prompt: &str, mercado_hash: &str) -> Option<String> {
        // 1. Exact Match (SHA-256) - Máxima velocidad
        if let Some(res) = self.buscar_exacto(prompt, mercado_hash) {
            return Some(res);
        }

        // 2. Semantic Match (LanceDB) - Inteligencia difusa
        if let Some(ref ocean) = self.ocean {
            debug!("🔍 Iniciando búsqueda semántica en Ocean...");
            let recuerdos = ocean.recordar_por_significado(prompt, 1).await;
            if let Some((impresion, score)) = recuerdos.first() {
                // Umbral de similitud del 95% para considerar un "Hit" de caché
                if *score > 0.95 {
                    debug!("🎯 Semantic Cache Hit (score: {:.4})", score);
                    return Some(impresion.esencia.clone());
                }
            }
        }

        None
    }

    fn buscar_exacto(&self, prompt: &str, mercado_hash: &str) -> Option<String> {
        let key = self.generar_key(prompt, mercado_hash);
        let ahora = chrono::Utc::now().timestamp();

        let mut stmt = self
            .conn
            .prepare("SELECT respuesta FROM cache WHERE hash_key = ?1 AND (timestamp + ttl) > ?2")
            .ok()?;

        let mut rows = stmt.query(params![key, ahora]).ok()?;
        if let Some(row) = rows.next().ok()? {
            let res: String = row.get(0).ok()?;
            debug!("🎯 Exact Cache Hit: Clave {}", key);
            return Some(res);
        }
        None
    }

    /// Guarda una respuesta con un TTL específico
    pub fn guardar(&self, prompt: &str, mercado_hash: &str, respuesta: &str, ttl_secs: i64) {
        let key = self.generar_key(prompt, mercado_hash);
        let ahora = chrono::Utc::now().timestamp();

        let res = self.conn.execute(
            "INSERT OR REPLACE INTO cache (hash_key, respuesta, timestamp, ttl) VALUES (?1, ?2, ?3, ?4)",
            params![key, respuesta, ahora, ttl_secs],
        );

        if res.is_ok() {
            debug!("💾 Cache Guardada: Clave {}", key);
        }
    }

    fn generar_key(&self, prompt: &str, mercado_hash: &str) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(prompt.as_bytes());
        hasher.update(mercado_hash.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}
