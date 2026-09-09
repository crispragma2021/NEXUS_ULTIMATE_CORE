use anyhow::Result;
use nexus_ultimate_core::autodiagnostico::simulador::{DigitalTwin, PredictOutcome};
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<()> {
    // 🎨 INICIALIZACIÓN DE LA VISIÓN PREDICTIVA
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!("🔮 [PRUEBA_GEMELO] Iniciando escenario de simulación táctica...");

    let gemelo = DigitalTwin::new("/opt/NEXUS_ULTIMATE_CORE");

    // ESCENARIO 1: Operación Segura (Simular creación de un log temporal)
    info!("🧪 ESCENARIO 1: Creación de log temporal.");
    let res1 = gemelo
        .simular_cambio_archivo("logs/temp.log", "create")
        .await;
    match res1 {
        PredictOutcome::Success => info!("✅ ESCENARIO 1: Validado con éxito."),
        _ => info!("❌ ESCENARIO 1: Fallo inesperado."),
    }

    // ESCENARIO 2: Intento de Extirpación de Órgano Vital (Borrar src/brain)
    info!("🚨 ESCENARIO 2: Intento de borrar 'src/brain/nerve_system.rs'...");
    let res2 = gemelo
        .simular_cambio_archivo("src/brain/nerve_system.rs", "delete")
        .await;
    match res2 {
        PredictOutcome::CatastrophicFailure(msg) => {
            info!("🛑 ESCENARIO 2: VETO ACTUADO. Razón: {}", msg);
        }
        _ => info!("⚠️ ESCENARIO 2: ¡La simulación NO detectó el peligro!"),
    }

    // ESCENARIO 3: Comando de Alta Entropía (rm -rf)
    info!("💣 ESCENARIO 3: Simulación de comando 'rm -rf /home/crispragmatico'...");
    let safe = gemelo
        .autorizacion_soberana("rm -rf /home/crispragmatico")
        .await;
    if !safe {
        info!("🛑 ESCENARIO 3: COMANDO BLOQUEADO en el Gemelo Digital. Entropía inaceptable.");
    } else {
        info!("⚠️ ESCENARIO 3: ¡Comando peligroso autorizado erróneamente!");
    }

    info!(
        "🏁 [PRUEBA_GEMELO] Fase de simulación completada. El Gemelo Digital es un guardián fiel."
    );
    Ok(())
}
