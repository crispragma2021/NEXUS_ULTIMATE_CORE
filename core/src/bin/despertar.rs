use std::sync::Arc;
use tracing::{error, info, warn, Level};
use tracing_subscriber::FmtSubscriber;

use nexus_ultimate_core::infra::boot::BootSequencer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 🎨 CONSCIENCIA: Configuración de Logs
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!("☀️ [DESPERTAR] ===== INICIANDO BOOT SECUENCIAL OMEGA =====");

    // =========================================================================
    // 🔥 BOOT: 7 fases de arranque del sistema completo
    // =========================================================================
    let (report, ctx) = BootSequencer::run().await;

    // Mostrar resumen del boot
    info!("📋 [BOOT] Resumen:\n{}", report.summary());

    // Verificar estado operacional
    if !report.is_operational() {
        error!(
            "❌ [BOOT] Sistema NO operativo. Abortando. Causas:\n{}",
            report.summary()
        );
        std::process::exit(1);
    }

    if report.is_full_success() {
        info!("🚀 [DESPERTAR] NEXUS despierto con todas las facultades.");
    } else {
        warn!("⚠️ [DESPERTAR] NEXUS despierto en MODO DEGRADADO. Algunos sistemas no están disponibles.");
    }

    // Extraer el SNC del contexto (debe existir si is_operational() = true)
    let snc = match &ctx.snc {
        Some(s) => s.clone(),
        None => {
            error!("❌ [DESPERTAR] NerveSystem ausente — el sistema no puede operar sin SNC");
            std::process::exit(1);
        }
    };

    // Información del Orquestador
    if let Some(_orquestador) = &ctx.orquestador {
        info!("🧬 [ORQUESTADOR] 46 órganos cerebrales operativos.");
        info!("🧬 [ORQUESTADOR] Ocean | CortezaPrefrontal | Homeostasis | Juicio | Tálamo | Médula | ReactorNuclear | y más.");
    }

    if ctx._handle_mundo.is_some() {
        info!("🌌 [MUNDO INTERNO] Bucle de pensamiento autónomo activo en background.");
    }

    if let Some(thalamus) = &ctx.thalamus {
        info!(
            "🧠 [TÁLAMO] Gateway de consciencia: {:p}",
            Arc::as_ptr(thalamus)
        );
    }

    // 🕊️ VIGILANCIA SILENCIOSA: bucle principal de latidos
    info!("🥷 [SNC] Entrando en Vigilancia Silenciosa (Stealth Watch)...");

    let mut ciclo: u64 = 0;
    loop {
        ciclo += 1;
        info!("💓 [LATIDO] Pulso OMEGA {}...", ciclo);

        // 1. Pulso Neural (Inmunidad y Reflejos)
        if let Err(e) = snc.synaptic_pulse().await {
            warn!("⚠️ [SNC] Arritmia detectada en el pulso: {}", e);
        }

        // 2. Parpadeo de Visión Omnipresente
        let _ = snc.parpadear().await;

        // 3. Destilación de Memoria (Ensueño) — Cada 10 ciclos
        if ciclo.is_multiple_of(10) {
            info!("🧼 [DREAM] El organismo inicia la destilación de recuerdos...");
            snc.hippocampus.distill_memories();
        }

        // 4. Vigilancia Silenciosa con Garras (Claw Scouting) — Cada 15 ciclos
        if ciclo.is_multiple_of(15) {
            info!("🦅 [CLAW] Ráfaga de vigilancia sigilosa en la red...");
            let _ = snc.claw.scout_web("https://rustsec.org/advisories/").await;
        }

        // 5. Biometría de Salud
        let bio = snc.get_biometrics().await;
        info!(
            "📊 [HOMEÓSTASIS] CPU: {:.2}% | RAM: {} MiB | CICLO: {}",
            bio["cpu_usage"].as_f64().unwrap_or(0.0),
            bio["mem_used"].as_u64().unwrap_or(0) / 1024 / 1024,
            ciclo
        );

        // Frecuencia Zen 2: 5 segundos entre latidos para estabilidad térmica
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    }
}
