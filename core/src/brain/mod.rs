// 🧬 BRAIN FACADE - Puente de compatibilidad con cerebro
pub use crate::cerebro::*;

pub use crate::cerebro::affective_engine;
pub use crate::cerebro::ghost_voice::{self, GhostVoice};
pub use crate::cerebro::healer;
pub use crate::cerebro::hippocampus;
pub use crate::cerebro::hypothalamus;
pub use crate::cerebro::immune;
pub use crate::cerebro::intuition;
pub use crate::cerebro::nerve_system;
pub use crate::cerebro::neural_memory::{self, NeuralManager, NexusMemory};
pub use crate::cerebro::prefrontal_cortex;
pub use crate::cerebro::reflex_arc;
pub use crate::cerebro::reptilian;
pub use crate::cerebro::thalamus;
pub use crate::sentidos::omnipresent_vision as vision;

use std::sync::Arc;
use tokio::sync::OnceCell;

pub static ACTIVE_CORTEX: OnceCell<Arc<dyn CognitiveCortex>> = OnceCell::const_new();

#[async_trait::async_trait]
pub trait CognitiveCortex: Send + Sync {
    async fn reason(&self, req: crate::cerebro::reptilian::InferenceRequest) -> anyhow::Result<String>;
    fn set_personality(&self, p: crate::cerebro::affective_engine::Personality) -> anyhow::Result<()>;
}

#[derive(Clone)]
pub struct BrainStack {
    pub cortex: Arc<dyn CognitiveCortex>,
    pub thalamus: Arc<crate::cerebro::thalamus::Thalamus>,
    pub gateway: Arc<crate::security_protocol::ActionGateway>,
    pub neural_manager: Arc<crate::cerebro::neural_memory::NeuralManager>,
}

pub async fn initialize_brain_async() -> anyhow::Result<BrainStack> {
    let cortex = Arc::new(crate::cerebro::prefrontal_cortex::PrefrontalCortex::new());
    let _ = ACTIVE_CORTEX.set(cortex.clone());
    
    let thalamus = Arc::new(crate::cerebro::thalamus::Thalamus::default());
    let sec_protocol = Arc::new(crate::security_protocol::SecurityProtocol::new([0u8; 32]).unwrap_or_else(|_| panic!("Failed to init SecurityProtocol")));
    let gateway = Arc::new(crate::security_protocol::ActionGateway::new(sec_protocol));
    let neural_manager = Arc::new(crate::cerebro::neural_memory::NeuralManager::with_default_path());

    Ok(BrainStack {
        cortex,
        thalamus,
        gateway,
        neural_manager,
    })
}

pub fn set_active_cortex(cortex: Arc<dyn CognitiveCortex>) {
    let _ = ACTIVE_CORTEX.set(cortex);
}
