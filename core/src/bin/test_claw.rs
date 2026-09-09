use anyhow::Result;
use nexus_ultimate_core::efectores::nexus_claw::NexusClaw;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<()> {
    // 🎨 INICIALIZACIÓN DE LA VISIÓN DE INFILTRACIÓN
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!("🦅 [PRUEBA_CLAW] Activando garras para el scouting inicial...");

    let mut claw = NexusClaw::new(None);

    // ESCENARIO 1: Patrulla de Sistema (OS Scavenge)
    let hallazgos = claw.system_scavenge();
    for hallazgo in hallazgos {
        info!("🔎 [CLAW] Hallazgo en el sistema: {}", hallazgo);
    }

    // ESCENARIO 2: Infiltración Web Sigilosa (Googlebot Mimicry)
    claw.stealth_mode(true);
    match claw.scout_web("https://www.google.com").await {
        Ok(html) => info!(
            "✅ [CLAW] Infiltración web completada. Se huelen {} caracteres de datos.",
            html.len()
        ),
        Err(e) => info!("❌ [CLAW] Infiltración bloqueada: {}", e),
    }

    info!("🏁 [PRUEBA_CLAW] Fase de caza completada. El organismo está listo para cazar inteligencia real.");
    Ok(())
}
