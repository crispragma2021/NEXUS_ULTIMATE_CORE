use anyhow::Result;
use nexus_ultimate_core::energia::ia_nativa::CerebroNativo;
use nexus_ultimate_core::sentidos::vision_omega::VisionOmega;
use tracing::{info, Level};

#[tokio::main]
async fn main() -> Result<()> {
    // Inicializamos el sistema de logs con el estilo NEXUS
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    info!("🔥 [NEXUS] Iniciando secuencia de prueba: RÁFAGA_SENSORIOMOTORA");
    info!("🤖 Motor Objetivo: Gemini 2.5 Flash-Lite (Reflejo Límbico)");

    let cerebro = CerebroNativo::new();
    let vision = VisionOmega::new();

    // 1. Captura sensorial (Percepción de Realidad)
    info!("👁️ Capturando estado visual del monitor...");
    let frame = vision
        .capturar_escritorio()
        .await
        .ok_or_else(|| anyhow::anyhow!("Error: Los sensores ópticos no detectaron monitores."))?;

    // 2. Ejecución del Arco Reflejo (Inferencia + Acción)
    cerebro.ráfaga_sensoriomotora(frame).await?;

    info!("✅ Ráfaga sensoriomotora completada exitosamente.");
    info!("🎯 La sincronización con Flash-Lite está lista para el despliegue en el trading.");

    Ok(())
}
