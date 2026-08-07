#![recursion_limit = "2048"]
pub mod autodiagnostico;
pub mod autonomia;
pub mod brain;
pub mod browser; // 🧬 Navegación, sesiones y perfiles (Fase 3)
pub mod cache; // ⚡ Cache Semántico y Eficiencia de Tokens
pub mod captcha; // 🧬 Detección, resolución y orquestación CAPTCHA (Fase 3)
pub mod cerebro;
pub mod colmena;
pub mod comms;
pub mod conocimiento;
pub mod defensa;
pub mod efectores;
pub mod emociones;
pub mod energia;
pub mod figma; // Añadido para el cliente Figma
pub mod identities;
pub mod infra;
pub mod memoria;
pub mod neuroquimica;
pub mod organismo; // 🫀 Interocepción: sensaciones corporales funcionales
pub mod nexus_embedder;
pub mod nexus_telegram;
pub mod phantom; // 👻 Módulo fantasma de compatibilidad
pub use memoria::evolution;
pub mod brain_metabolism;
pub mod neural_ingest;
pub mod orden;
pub mod orquestador; // 🔱 Orquestador local determinista de operadores (5 pilares)
pub mod prediccion;
pub mod procesos;
pub mod reflejos;
pub mod scraping; // 🕸️ Pipeline de scraping: captura, limpieza y enrutado (F0/F1)
pub mod security_protocol;
pub mod sentidos;
pub mod spatial;
pub mod valores;

pub use brain_metabolism::{aplicar_metabolismo, obtener_latencia_disco, METABOLISMO_ACTUAL};

// 👻 Re-export de módulos fantasma para compatibilidad
pub use phantom::capa_invisibilidad;
pub use phantom::homeostasis_utils;
pub use phantom::medico;
pub use phantom::thinking_strategy;

// Capa de Compatibilidad y Puentes de ADN
pub use autodiagnostico::nexus_repair;
pub use autodiagnostico::nexus_repair::{DivineOptimizer, ServiceManager};
pub use brain::affective_engine::Personality;
pub use brain::{initialize_brain_async, set_active_cortex, CognitiveCortex, NeuralManager};
pub use comms::actions;
pub use defensa::mediador_accion::MediadorAccion;
pub use efectores::agente_ejecutor::AgenteEjecutor;
pub use efectores::nexus_claw_pro::NexusClawPro;
pub use infra::arsenal;
// 🧬 Módulos migrados de legacy — exportados para uso público
pub use colmena::{ColmenaHijo, ColmenaMadre};
pub use defensa::biometric_bridge;
pub use defensa::sistema_digestivo::SistemaDigestivo;
pub use efectores::oido_empatico::{OidoEmpatico, Tono};
pub use efectores::osint::{DorkEngine, ShadowCrawlClient, ShadowSearchResult, UsernameScanner};
pub use infra::ghost_vm::GhostVmController;
pub use infra::herramientas_nativas::HerramientasNativas;
pub use infra::kernel::KernelSovereign;
pub use infra::network;
pub use infra::paths::{resolve_path, NEXUS_ROOT};
pub use memoria::persistence;
pub use memoria::memory_loader;
pub use memoria::intention_encoder;
pub use memoria::prompt_assembler;
pub use memoria::sistema_limbico;
pub use procesos::telemetry;

pub fn set_personality(p: Personality) -> anyhow::Result<()> {
    if let Some(cortex) = brain::ACTIVE_CORTEX.get() {
        let _ = pollster::block_on(cortex.set_personality(p));
        Ok(())
    } else {
        Err(anyhow::anyhow!("Córtex Cognitivo no inicializado"))
    }
}
