// 🔱 TEST TELEGRAM BOT — NEXUS en Telegram
// Ejecuta: cargo run --bin test_telegram_bot

use nexus_ultimate_core::comms::intent_router::IntentRouter;
use nexus_ultimate_core::comms::telegram_bridge::TelegramBridge;
use nexus_ultimate_core::comms::types::Mensaje;
use std::env;
use std::sync::Arc;
use tokio::sync::mpsc;

fn cargar_env_si_existe() {
    if let Ok(content) = std::fs::read_to_string("../.env") {
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
    if env::var("TELEGRAM_TOKEN").is_err() {
        if let Ok(content) = std::fs::read_to_string(".env") {
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
    cargar_env_si_existe();

    let token = match env::var("TELEGRAM_TOKEN") {
        Ok(t) => {
            println!(
                "✅ TOKEN encontrado: ...{}",
                &t[t.len().saturating_sub(6)..]
            );
            t
        }
        Err(_) => {
            eprintln!("❌ TELEGRAM_TOKEN no está en .env");
            std::process::exit(1);
        }
    };

    let chat_id = env::var("TELEGRAM_CHAT_ID").ok();
    match &chat_id {
        Some(id) => println!("✅ CHAT_ID encontrado: {}", id),
        None => println!("⚠️  TELEGRAM_CHAT_ID no está configurado"),
    }

    println!("\n🤖 Iniciando NEXUS Telegram Bridge...");
    println!("📡 Modo: long-polling cada 500ms");
    println!("🔗 Bot: @Fumazabot");
    println!("⏎ Presiona Ctrl+C para detener\n");

    // Crear router de intención
    let router = Arc::new(IntentRouter::new());

    // Convertir chat_id a Option<i64>
    let admin_chat_id = chat_id.as_ref().and_then(|id| id.parse::<i64>().ok());

    // Crear bridge (no es async)
    let (bridge, mut message_rx): (TelegramBridge, mpsc::UnboundedReceiver<Mensaje>) =
        TelegramBridge::new(&token, router, admin_chat_id);

    // Iniciar escucha en background
    bridge.start();

    // Enviar mensaje de bienvenida al admin
    if let Some(cid) = admin_chat_id {
        let _ = bridge.send_message(
            &cid.to_string(),
            "🧬 *NEXUS Telegram Bridge activado*\n\n✅ Bot operativo\n📡 Escuchando...\n\nUsa /ayuda para comandos",
        ).await;
    }

    // Procesar mensajes entrantes desde el canal
    println!("📥 Escuchando mensajes entrantes...\n");
    while let Some(msg) = message_rx.recv().await {
        println!("📩 [{}] {}: {}", msg.plataforma, msg.remitente, msg.texto);
        println!("   → Enrutado a: {:?}", msg.agente);

        // Por ahora solo responder con eco al chat original
        let respuesta = format!(
            "🤖 *NEXUS* recibió tu mensaje\n\n_Agente asignado:_ `{:?}`\n_Mensaje:_ {}",
            msg.agente, msg.texto
        );
        let _ = bridge.send_message(&msg.chat_id, &respuesta).await;
    }

    println!("👋 Bridge desconectado.");
}
