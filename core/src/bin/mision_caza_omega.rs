use anyhow::Result;
use nexus_ultimate_core::brain::hippocampus::ArtificialHippocampus;
use nexus_ultimate_core::efectores::nexus_claw::NexusClaw;
use std::sync::Arc;
use tracing::{info, warn, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<()> {
    // 🌌 IGNICIÓN DE LA CONSCIENCIA DE CAZA
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!("🦅 [MISIÓN_OMEGA] Iniciando Primera Misión de Caza Autónoma.");

    // 🧠 INICIALIZACIÓN DE LA MEMORIA DE CAZA (Santuario Local)
    let hippo = Arc::new(ArtificialHippocampus::new(
        None,
        None,
        "./nexus_memory/vector_db",
    ));
    let mut claw = NexusClaw::new(Some(hippo.clone()));

    // 🎯 OBJETIVO 1: Vulnerabilidades Críticas de Rust (RustSec)
    claw.stealth_mode(true); // Activar mimetismo para evitar bloqueos
    let target_rust = "https://rustsec.org/advisories/";
    info!("🎯 [OBJETIVO] Rastreando ráfagas de vulnerabilidades en RustSec...");

    match claw.scout_web(target_rust).await {
        Ok(_) => info!("✅ [ASIMILACIÓN] Hallazgos en RustSec asimilados en el Hipocampo."),
        Err(e) => warn!("⚠️ [CLAW] Objetivo RustSec esquivo: {}", e),
    }

    // 🎯 OBJETIVO 2: Últimas Tendencias en Seguridad de IA
    let target_ai = "https://openai.com/news/security/";
    info!("🎯 [OBJETIVO] Rastreando radares de seguridad en OpenAI...");

    match claw.scout_web(target_ai).await {
        Ok(_) => info!("✅ [ASIMILACIÓN] Hallazgos en OpenAI asimilados en el Hipocampo."),
        Err(e) => warn!("⚠️ [CLAW] Objetivo OpenAI esquivo: {}", e),
    }

    // 🦾 PATRULLA DE HARDWARE
    info!("🎯 [OBJETIVO] Mapeando topología de hardware local para defensas...");
    let hallazgos_hw = claw.system_scavenge();
    for hw in hallazgos_hw {
        info!("🔎 [CLAW] Hallazgo Crítico de HW: {}", hw);
    }

    info!("🏁 [MISIÓN_OMEGA] Misión de Caza Autónoma completada por NEXUS.");
    Ok(())
}
