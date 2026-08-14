// ==========================================
// 🧠 NEXUS BRAIN — Córtex Cognitivo Unificado
// ==========================================
// [ÓRGANO RESTAURADO] Este módulo se perdió en la migración Ubuntu→Windows
// (estaba en .gitignore local y nunca se subió al repo).
// Restaurado como shim anatómico: los órganos viven en sus sistemas
// (sentidos/, memoria/, cerebro/), brain/ los re-exporta y define los
// tipos de coordinación central (NeuralManager, CognitiveCortex, BrainStack).
// ==========================================

pub mod affective_engine;
pub mod ghost_voice;
pub mod healer;
pub mod hippocampus;
pub mod hypothalamus;
pub mod immune;
pub mod intuition;
pub mod nerve_system;
pub mod neural_memory;
pub mod prefrontal_cortex;
pub mod reflex_arc;
pub mod reptilian;
pub mod thalamus;
pub mod vision;

pub use affective_engine::Personality;
pub use ghost_voice::GhostVoice;
pub use hippocampus::ArtificialHippocampus;
pub use neural_memory::NexusMemory;
pub use nerve_system::NerveSystem;
pub use reflex_arc::ReflexSignal;
pub use thalamus::Thalamus;
pub use vision::{HypothalamusSignal, OmnipresentVision};

use std::sync::Arc;
use std::sync::OnceLock;
use tracing::{error, info, warn};

// ─── Coordinación central del córtex ────────────────────────────────────

/// Gestor de redes neuronales (memoria vectorial + embeddings).
pub struct NeuralManager {
    /// Ruta de la base de memoria vectorial.
    pub memory_path: String,
    /// Embedder de texto activo.
    pub embedder: Option<Arc<crate::nexus_embedder::NexusEmbedder>>,
}

impl NeuralManager {
    pub fn new() -> Self {
        Self {
            memory_path: "data/nexus_memoria.lance".to_string(),
            embedder: None,
        }
    }

    pub fn with_embedder(embedder: Arc<crate::nexus_embedder::NexusEmbedder>) -> Self {
        Self {
            memory_path: "data/nexus_memoria.lance".to_string(),
            embedder: Some(embedder),
        }
    }

    /// Devuelve el motor de inferencia activo (RwLock para lectura concurrente).
    pub fn get_active_engine(
        &self,
    ) -> Arc<tokio::sync::RwLock<Option<Box<crate::brain::neural_memory::InferenceEngine>>>> {
        static NO_ENGINE: std::sync::OnceLock<
            Arc<tokio::sync::RwLock<Option<Box<crate::brain::neural_memory::InferenceEngine>>>>,
        > = std::sync::OnceLock::new();
        NO_ENGINE
            .get_or_init(|| Arc::new(tokio::sync::RwLock::new(None)))
            .clone()
    }
}

impl Default for NeuralManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Córtex cognitivo: interfaz de razonamiento del organismo.
pub trait CognitiveCortex: Send + Sync {
    /// Establece la personalidad activa del organismo.
    fn set_personality(&self, p: Personality) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<()>> + Send + '_>>;
    /// Procesa un mensaje entrante y produce una respuesta.
    fn procesar(&self, entrada: &str) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<String>> + Send + '_>>;
    /// Razonamiento profundo con una solicitud estructurada (API omega_stress).
    fn reason(&self, request: crate::brain::reptilian::InferenceRequest) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<String>> + Send + '_>>;
}

/// Stack completo del córtex: coordina tálamo, gateway y red neural.
#[derive(Clone)]
pub struct BrainStack {
    pub thalamus: Arc<Thalamus>,
    pub gateway: Arc<crate::security_protocol::ActionGateway>,
    pub neural_manager: Arc<NeuralManager>,
}

impl BrainStack {
    pub fn new(
        thalamus: Arc<Thalamus>,
        gateway: Arc<crate::security_protocol::ActionGateway>,
        neural_manager: Arc<NeuralManager>,
    ) -> Self {
        Self {
            thalamus,
            gateway,
            neural_manager,
        }
    }
}

/// Córtex activo global (una sola instancia por proceso).
pub static ACTIVE_CORTEX: OnceLock<Arc<dyn CognitiveCortex>> = OnceLock::new();

/// Inicializa el córtex cognitivo completo y retorna el BrainStack.
pub async fn initialize_brain_async() -> anyhow::Result<BrainStack> {
    info!("🧠 Inicializando córtex cognitivo (BrainStack)...");

    let neural_manager = Arc::new(NeuralManager::new());
    let thalamus = Arc::new(Thalamus::new());
    // Gateway de emergencia (clave maestra por defecto, como boot.rs fase 6)
    let protocol = crate::security_protocol::SecurityProtocol::new([0u8; 32])
        .map_err(|e| anyhow::anyhow!("SecurityProtocol emergencia: {e}"))?;
    let gateway = Arc::new(crate::security_protocol::ActionGateway::new(Arc::new(protocol)));

    let stack = BrainStack::new(thalamus.clone(), gateway.clone(), neural_manager.clone());
    info!("✅ BrainStack listo (thalamus + gateway + neural)");
    Ok(stack)
}

/// Registra el córtex activo global.
pub fn set_active_cortex(cortex: Arc<dyn CognitiveCortex>) -> bool {
    ACTIVE_CORTEX.set(cortex).is_ok()
}

/// Retorna una referencia al córtex activo si existe.
pub fn active_cortex() -> Option<&'static Arc<dyn CognitiveCortex>> {
    ACTIVE_CORTEX.get()
}

/// Degradación elegante si el córtex no está disponible.
pub fn cortex_unavailable() -> anyhow::Error {
    anyhow::anyhow!("Córtex Cognitivo no inicializado")
}

#[allow(unused)]
fn _quiet_warnings() {
    let _ = error!("");
    let _ = warn!("");
}
