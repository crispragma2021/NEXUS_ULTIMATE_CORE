// ==========================================
// 🔱 NEXUS TELEGRAM DAEMON — Integración real
// ==========================================
// Conecta el TelegramBridge de NEXUS al Orquestador completo.
// El bot responde con la personalidad REAL de NEXUS (memoria,
// Juicio Soberano, pipeline de 14 etapas) — no un eco.
//
// Ejecutar: cargo run --release --bin nexus_telegram_daemon
// (o usar el binario compilado en target/release/)
// ==========================================

use nexus_ultimate_core::brain::hippocampus::ArtificialHippocampus;
use nexus_ultimate_core::cerebro::orquestador::Orquestador;
use nexus_ultimate_core::comms::intent_router::IntentRouter;
use nexus_ultimate_core::comms::telegram_bridge::TelegramBridge;
use nexus_ultimate_core::comms::types::Mensaje;
use std::env;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::info;

fn cargar_env_si_existe() {
    // Cargar ../.env primero (cuando se ejecuta desde core/)
    for path in ["../.env", ".env"] {
        if let Ok(content) = std::fs::read_to_string(path) {
            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                if let Some((key, val)) = line.split_once('=') {
                    let key = key.trim();
                    let val = val.trim().trim_matches('"');
                    if env::var(key).is_err() {
                        env::set_var(key, val);
                    }
                }
            }
        }
    }
}

#[tokio::main]
async fn main() {
    // Tracing
    let _ = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .try_init();

    cargar_env_si_existe();

    let token = match env::var("TELEGRAM_TOKEN") {
        Ok(t) => {
            info!("✅ TOKEN encontrado: ...{}", &t[t.len().saturating_sub(6)..]);
            t
        }
        Err(_) => {
            eprintln!("❌ TELEGRAM_TOKEN no está en .env");
            std::process::exit(1);
        }
    };

    let chat_id = env::var("TELEGRAM_CHAT_ID").ok();
    let admin_chat_id = chat_id.as_ref().and_then(|id| id.parse::<i64>().ok());

    info!("🧠 Inicializando ORQUESTADOR COMPLETO de NEXUS...");
    let hippocampus = Arc::new(ArtificialHippocampus::new(
        None,
        None,
        "data/nexus_memoria.lance",
    ));
    let orquestador = Orquestador::new(hippocampus).await;
    let orquestador = Arc::new(orquestador);
    info!("✅ Orquestador listo (46 órganos cerebrales activos)");

    // Router de intención (enruta a agentes especialistas)
    let router = Arc::new(IntentRouter::new());

    // Crear bridge de Telegram
    let (bridge, mut message_rx): (TelegramBridge, mpsc::UnboundedReceiver<Mensaje>) =
        TelegramBridge::new(&token, router.clone(), admin_chat_id);

    // Iniciar long-polling en background
    bridge.start();
    info!("📡 Telegram Bridge en long-polling — bot escuchando...");

    // Bienvenida al admin
    if let Some(cid) = admin_chat_id {
        let _ = bridge
            .send_message(
                &cid.to_string(),
                "🧬 *NEXUS Telegram Bridge activado* (integración real)\n\n\
                 ✅ Bot operativo con Orquestador completo\n\
                 📡 Escuchando...\n\n\
                 Escribe cualquier mensaje y te responderé con mi personalidad completa.",
            )
            .await;
    }

    info!("📥 Escuchando mensajes entrantes...");

    // Bucle principal: cada mensaje → orquestador.responder() → respuesta real
    while let Some(msg) = message_rx.recv().await {
        let chat = msg.chat_id.clone();
        let texto = msg.texto.clone();
        let remitente = msg.remitente.clone();

        info!("📩 [{}] {}: {}", msg.plataforma, remitente, texto);

        // Pipeline completo del orquestador (14 etapas: inmune, memoria,
        // emociones, Juicio Soberano, hemisferios, etc.)
        let respuesta = orquestador.responder(&texto).await;

        // Enviar la respuesta real de NEXUS
        let _ = bridge.send_message(&chat, &respuesta).await;
        info!("✅ Respuesta enviada ({} chars)", respuesta.len());
    }

    info!("👋 Bridge desconectado.");
}
