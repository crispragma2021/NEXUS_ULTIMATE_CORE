use anyhow::Result;
use nexus_ultimate_core::brain::intuition::{IntuitionFeeling, IntuitionLobe};
use std::fs;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<()> {
    // 🎨 INICIALIZACIÓN DE LA VISIÓN PREDICTIVA
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!("🦋 [ESCANEO_INTUICIÓN] Iniciando percepción profunda del organismo...");

    let intuicion = IntuitionLobe::new();
    let root = "/opt/NEXUS_ULTIMATE_CORE/src";

    let archivos_clave = [
        "brain/mod.rs",
        "brain/nerve_system.rs",
        "security_protocol.rs",
        "bin/despertar.rs",
        "sentidos/mod.rs",
    ];

    for archivo in archivos_clave {
        let path = format!("{}/{}", root, archivo);
        match fs::read_to_string(&path) {
            Ok(content) => {
                let feeling = intuicion.sentir_codigo(&content);
                match feeling {
                    IntuitionFeeling::Stable => info!(
                        "✅ [ESTABLE] {}: Siento una estructura sólida y equilibrada.",
                        archivo
                    ),
                    IntuitionFeeling::Rotting(msg) => info!(
                        "⚠️ [FRÁGIL] {}: Percibo decadencia estructural. Razón: {}",
                        archivo, msg
                    ),
                    IntuitionFeeling::Unstable(val) => info!(
                        "🚨 [INEXTABLE] {}: Siento una vibración de inestabilidad de nivel {:.2}.",
                        archivo, val
                    ),
                    _ => info!("ℹ️ [OK] {}: Percepción neutral.", archivo),
                }
            }
            Err(_) => info!(
                "❌ [CEGUERA] No pude leer {}. Mi percepción está bloqueada en esta ruta.",
                archivo
            ),
        }
    }

    info!(
        "🏁 [ESCANEO_INTUICIÓN] Mapa intuitivo completado. El organismo conoce su propio cuerpo."
    );
    Ok(())
}
