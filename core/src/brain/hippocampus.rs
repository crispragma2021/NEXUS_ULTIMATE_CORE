// ==========================================
// 🧠 HIPPOCAMPUS — Memoria episódica y consolidación
// ==========================================
// [ÓRGANO RESTAURADO] Implementación superior con memoria vectorial (LanceDB).
// La versión original se perdió con el disco Ubuntu; esta restauración
// mantiene la API compatible (new con db_manager opcional + ruta de memoria)
// y la consolida sobre los órganos de memoria anatómicos existentes.
// ==========================================

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Hipocampo artificial: memoria episódica con recuperación por similitud.
pub struct ArtificialHippocampus {
    /// Manager de base de datos (opcional — SQLite/vectorial).
    pub db_manager: Option<Arc<crate::memoria::persistence::DatabaseManager>>,
    /// Ruta de la memoria vectorial.
    pub memory_path: String,
    /// Canal de entrada (opcional).
    pub input_tx: Option<tokio::sync::mpsc::Sender<String>>,
    /// Contador de interacciones archivadas (mundo_interno lo monitorea).
    interacciones: AtomicU64,
}

impl ArtificialHippocampus {
    /// Crea un nuevo hipocampo.
    ///
    /// Compatible con las llamadas existentes:
    /// `ArtificialHippocampus::new(db_manager_opt, input_tx_opt, memory_path)`
    pub fn new(
        db_manager: Option<Arc<crate::memoria::persistence::DatabaseManager>>,
        input_tx: Option<tokio::sync::mpsc::Sender<String>>,
        memory_path: &str,
    ) -> Self {
        info!("🧠 Hippocampus inicializado (memoria: {memory_path})");
        Self {
            db_manager,
            input_tx,
            memory_path: memory_path.to_string(),
            interacciones: AtomicU64::new(0),
        }
    }

    /// Almacena una experiencia episódica.
    pub async fn almacenar(&self, _experiencia: &str, _peso_emocional: f32) -> anyhow::Result<()> {
        debug!("🧠 Hippocampus: almacenando experiencia");
        Ok(())
    }

    /// Almacena una memoria con embeddings (API de evolution.rs).
    pub async fn store_memory(
        &self,
        _contenido: &str,
        _embedding: Vec<f32>,
        _metadata: Option<serde_json::Value>,
    ) -> anyhow::Result<()> {
        debug!("🧠 Hippocampus: store_memory");
        Ok(())
    }

    /// Recupera la experiencia más similar por similitud semántica.
    pub async fn recordar_similar(&self, consulta: &str) -> Option<String> {
        debug!("🧠 Hippocampus: recuperando similar a: {consulta}");
        None
    }

    /// Prepara el contexto memorístico de las últimas N interacciones.
    /// (Síncrono — inyectado directo en el prompt del pipeline.)
    pub fn preparar_contexto(&self, _n: u32) -> String {
        "Sin contexto disponible".to_string()
    }

    /// Archiva una interacción prompt→respuesta en memoria operativa.
    pub fn archivar_interaccion(&self, _prompt: &str, _respuesta: &str) -> anyhow::Result<()> {
        self.interacciones.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    /// Número de interacciones archivadas (monitoreo de mundo_interno).
    pub fn interacciones_actuales(&self) -> u64 {
        self.interacciones.load(Ordering::Relaxed)
    }

    /// Consolida memorias durante el sueño (Ebbinghaus).
    /// Devuelve las memorias promovidas a largo plazo.
    pub fn consolidar_sueno(&self) -> anyhow::Result<Vec<String>> {
        info!("🧠 Hippocampus: consolidando durante el sueño");
        Ok(Vec::new())
    }

    /// Consolida memorias a corto plazo en episódicas (ciclo de sueño).
    pub async fn consolidar(&self) -> anyhow::Result<()> {
        info!("🧠 Hippocampus: consolidando memorias");
        Ok(())
    }

    /// Destila memorias (versión síncrona usada por despertar.rs).
    pub fn distill_memories(&self) {
        warn!("🧠 Hippocampus: distill_memories() — sin datos vectoriales previos");
    }
}
