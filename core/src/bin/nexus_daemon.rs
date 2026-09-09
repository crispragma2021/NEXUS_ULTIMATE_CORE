use anyhow::Result;
use dotenv::dotenv;
use nexus_ultimate_core::autonomia::NexusKernel;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    // Configurar logging con estilo NEXUS
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!("🔱 [NEXUS DAEMON] Iniciando Órgano de Autonomía...");

    // Crear el Kernel con un pulso de 60 segundos
    let kernel = NexusKernel::new(60);

    // Iniciar el pulso eterno
    kernel.iniciar().await;
}
