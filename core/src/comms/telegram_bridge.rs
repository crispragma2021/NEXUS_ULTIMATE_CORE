// ==========================================
// 🧬 NEXUS OMEGA — Telegram Bridge Bidireccional
// ==========================================
// Bot de Telegram en Rust puro (teloxide 0.12.2).
// Usa long-polling manual con get_updates() en vez del Dispatcher
// para evitar complejidades de tipos genéricos de dptree.
// ==========================================

use super::intent_router::IntentRouter;
use super::types::{Mensaje, NexusAgent, Plataforma};
use std::sync::Arc;
use teloxide::prelude::*;
use teloxide::types::{ChatAction, ParseMode, UpdateKind};
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

/// Bridge bidireccional de Telegram.
pub struct TelegramBridge {
    bot: Bot,
    router: Arc<IntentRouter>,
    message_tx: mpsc::UnboundedSender<Mensaje>,
    admin_chat_id: Option<i64>,
}

impl TelegramBridge {
    /// Crea un nuevo TelegramBridge.
    /// Devuelve (bridge, receptor de mensajes entrantes).
    pub fn new(
        token: &str,
        router: Arc<IntentRouter>,
        admin_chat_id: Option<i64>,
    ) -> (Self, mpsc::UnboundedReceiver<Mensaje>) {
        let (message_tx, message_rx) = mpsc::unbounded_channel();
        let bot = Bot::new(token.to_string());

        (
            Self {
                bot,
                router,
                message_tx,
                admin_chat_id,
            },
            message_rx,
        )
    }

    /// Inicia la escucha de mensajes vía long-polling en background.
    pub fn start(&self) {
        let bot = self.bot.clone();
        let router = self.router.clone();
        let message_tx = self.message_tx.clone();
        let admin_chat_id = self.admin_chat_id;

        tokio::spawn(async move {
            // Notificar al admin
            if let Some(admin_id) = admin_chat_id {
                let msg = format!(
                    "🧬 NEXUS OMEGA — Bot Conectado\n\n\
                     Agentes disponibles: /help\n\
                     Estado: 🟢 Operativo"
                );
                let _ = bot.send_message(ChatId(admin_id), msg).await;
            }

            info!("📡 [TELEGRAM] Bot iniciando long-polling...");

            let mut offset: i32 = 0;

            loop {
                // Obtener updates desde el offset actual.
                // Timeout 50s: Telegram mantiene el long-poll hasta ~50s;
                // un timeout menor corta la conexión y pierde mensajes.
                let updates = match bot.get_updates().offset(offset).timeout(50).send().await {
                    Ok(updates) => updates,
                    Err(e) => {
                        debug!(
                            "🔄 [TELEGRAM] get_updates timeout normal (long-poll): {}",
                            e
                        );
                        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                        continue;
                    }
                };

                for upd in updates {
                    // Actualizar offset para no recibir el mismo update dos veces
                    if upd.id >= offset {
                        offset = upd.id + 1;
                    }

                    // Solo procesar mensajes de texto
                    let msg = match upd.kind {
                        UpdateKind::Message(ref m) => m,
                        _ => continue,
                    };

                    // Ignorar bots
                    if let Some(from) = msg.from() {
                        if from.is_bot {
                            continue;
                        }
                    }

                    let texto = msg.text().unwrap_or("").to_string();
                    let chat_id = msg.chat.id;
                    let remitente = msg
                        .from()
                        .map(|u| {
                            u.first_name.clone()
                                + &u.last_name
                                    .as_ref()
                                    .map(|l| format!(" {}", l))
                                    .unwrap_or_default()
                        })
                        .unwrap_or_default();

                    // ── Comando /help ──────────────────────
                    if texto.trim().eq_ignore_ascii_case("/help")
                        || texto.trim().eq_ignore_ascii_case("/start")
                        || texto.trim().eq_ignore_ascii_case("/agentes")
                    {
                        let ayuda = router.ayuda_comandos();
                        let _ = bot
                            .send_message(chat_id, ayuda)
                            .parse_mode(ParseMode::MarkdownV2)
                            .await;
                        continue;
                    }

                    info!(
                        "📩 [TELEGRAM] Mensaje de '{}' ({}): {}",
                        remitente,
                        chat_id.0,
                        texto.chars().take(80).collect::<String>()
                    );

                    // En grupos, solo responder si hay mención directa
                    if msg.chat.is_group() || msg.chat.is_supergroup() {
                        let es_mencion = texto.starts_with('/')
                            || texto.starts_with('@')
                            || texto.to_lowercase().contains("nexus");
                        if !es_mencion {
                            continue;
                        }
                    }

                    let (agente, texto_limpio) = router.enrutar(&texto);

                    let mensaje = Mensaje {
                        id: msg.id.0.to_string(),
                        texto: texto_limpio,
                        agente: Some(agente),
                        chat_id: chat_id.0.to_string(),
                        remitente,
                        timestamp: chrono::Utc::now().to_rfc3339(),
                        plataforma: Plataforma::Telegram,
                        es_respuesta: false,
                    };

                    let _ = message_tx.send(mensaje);
                    let _ = bot.send_chat_action(chat_id, ChatAction::Typing).await;
                }

                // Pequeña pausa entre polls
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            }
        });
    }

    /// Envía un mensaje a un chat específico.
    /// Parte el texto en chunks de ≤4000 chars (límite de Telegram) y
    /// usa fallback a texto plano si MarkdownV2 falla por caracteres.
    pub async fn send_message(&self, chat_id: &str, texto: &str) -> Result<(), String> {
        let id: i64 = chat_id
            .parse()
            .map_err(|e| format!("ChatId inválido: {}", e))?;

        // Chunking: Telegram acepta máx. 4096 chars por mensaje
        const MAX_CHARS: usize = 4000;
        let chunks: Vec<&str> = if texto.len() <= MAX_CHARS {
            vec![texto]
        } else {
            let mut v = Vec::new();
            let mut rest = texto;
            while !rest.is_empty() {
                let cut = rest.char_indices().take(MAX_CHARS).last();
                let idx = cut.map(|(i, _)| i).unwrap_or(rest.len());
                v.push(&rest[..idx]);
                rest = &rest[idx..];
            }
            v
        };

        for chunk in &chunks {
            let formatted = Self::escapar_markdown(chunk);
            // Intento 1: MarkdownV2
            let res = self
                .bot
                .send_message(ChatId(id), &formatted)
                .parse_mode(ParseMode::MarkdownV2)
                .await;
            // Intento 2 (fallback): texto plano si MarkdownV2 falla
            if res.is_err() {
                let _ = self
                    .bot
                    .send_message(ChatId(id), *chunk)
                    .await
                    .map_err(|e| format!("Error enviando mensaje (plano): {}", e))?;
            }
        }
        Ok(())
    }

    /// Envía una alerta al administrador.
    pub async fn broadcast_alert(&self, mensaje: &str) -> Result<(), String> {
        let Some(admin_id) = self.admin_chat_id else {
            warn!("⚠️ [TELEGRAM] No hay admin_chat_id configurado.");
            return Err("TELEGRAM_ADMIN_ID no configurado".to_string());
        };
        let formatted = format!("🔱 [NEXUS-OMEGA]\n{}", Self::escapar_markdown(mensaje));
        self.bot
            .send_message(ChatId(admin_id), &formatted)
            .parse_mode(ParseMode::MarkdownV2)
            .await
            .map_err(|e| format!("Error enviando alerta: {}", e))?;
        info!("📡 [TELEGRAM] Alerta enviada al admin.");
        Ok(())
    }

    /// Escape de caracteres especiales para MarkdownV2 de Telegram.
    fn escapar_markdown(texto: &str) -> String {
        texto
            .chars()
            .map(|c| match c {
                '_' => "\\_".to_string(),
                '*' => "\\*".to_string(),
                '[' => "\\[".to_string(),
                ']' => "\\]".to_string(),
                '(' => "\\(".to_string(),
                ')' => "\\)".to_string(),
                '~' => "\\~".to_string(),
                '`' => "\\`".to_string(),
                '>' => "\\>".to_string(),
                '#' => "\\#".to_string(),
                '+' => "\\+".to_string(),
                '-' => "\\-".to_string(),
                '=' => "\\=".to_string(),
                '|' => "\\|".to_string(),
                '{' => "\\{".to_string(),
                '}' => "\\}".to_string(),
                '.' => "\\.".to_string(),
                '!' => "\\!".to_string(),
                other => other.to_string(),
            })
            .collect()
    }
}

/// Función legacy para compatibilidad con código existente.
pub async fn send_alert(message: &str) {
    dotenv::dotenv().ok();
    let token = std::env::var("TELEGRAM_TOKEN").unwrap_or_default();
    let chat_id = std::env::var("TELEGRAM_CHAT_ID").unwrap_or_default();

    if token.is_empty() || chat_id.is_empty() {
        warn!("⚠️ [TELEGRAM] Token o ChatID no configurados. Alerta omitida.");
        return;
    }

    let bot = Bot::new(token);
    let chat_id_num = match chat_id.parse::<i64>() {
        Ok(id) => id,
        Err(_) => {
            error!("❌ [TELEGRAM] ChatID inválido.");
            return;
        }
    };

    let formatted_msg = format!("🔱 [NEXUS-OMEGA]\n{}", message);

    match bot.send_message(ChatId(chat_id_num), formatted_msg).await {
        Ok(_) => info!("📡 [TELEGRAM] Alerta legacy enviada con éxito."),
        Err(e) => error!("❌ [TELEGRAM] Fallo al enviar alerta legacy: {}", e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escapar_markdown_basico() {
        let input = "_Hola_ *mundo*";
        let escaped = TelegramBridge::escapar_markdown(input);
        assert_eq!(escaped, "\\_Hola\\_ \\*mundo\\*");
    }

    #[test]
    fn test_escapar_markdown_sin_especiales() {
        let input = "Hola mundo normal";
        let escaped = TelegramBridge::escapar_markdown(input);
        assert_eq!(escaped, "Hola mundo normal");
    }
}
