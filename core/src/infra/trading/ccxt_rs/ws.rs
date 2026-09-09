// 🔱 ccxt_rs — WebSocket unificado para streaming en vivo
// Maneja reconexión automática, heartbeats, y parseo de mensajes

use core::time::Duration;
use futures::stream::{unfold, Stream};
use futures::SinkExt;
use futures::StreamExt;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::Mutex;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;

use super::error::{ExchangeError, ExchangeResult};

/// Configuración de WebSocket
#[derive(Debug, Clone)]
pub struct WsConfig {
    pub url: String,
    pub ping_interval_secs: u64,
    pub reconnect_delay_ms: u64,
    pub max_reconnect_attempts: u32,
}

impl Default for WsConfig {
    fn default() -> Self {
        Self {
            url: String::new(),
            ping_interval_secs: 30,
            reconnect_delay_ms: 1000,
            max_reconnect_attempts: 10,
        }
    }
}

/// Estado de la conexión WebSocket
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WsState {
    Disconnected,
    Connecting,
    Connected,
    Reconnecting(u32),
}

/// Callback para procesar mensajes raw del WebSocket
pub type MessageCallback =
    Arc<dyn Fn(String) -> ExchangeResult<Option<serde_json::Value>> + Send + Sync>;

/// Gestor de WebSocket con reconexión automática
pub struct WsManager {
    config: WsConfig,
    state: Arc<Mutex<WsState>>,
    should_run: Arc<AtomicBool>,
    /// Callback para transformar mensajes raw en JSON
    parser: Option<MessageCallback>,
    exchange_name: String,
}

impl WsManager {
    pub fn new(config: WsConfig, exchange_name: &str) -> Self {
        Self {
            config,
            state: Arc::new(Mutex::new(WsState::Disconnected)),
            should_run: Arc::new(AtomicBool::new(false)),
            parser: None,
            exchange_name: exchange_name.to_string(),
        }
    }

    /// Establecer el parser de mensajes específico del exchange
    pub fn set_parser(&mut self, parser: MessageCallback) {
        self.parser = Some(parser);
    }

    /// Obtener el estado actual
    pub async fn state(&self) -> WsState {
        *self.state.lock().await
    }

    /// Conectar y mantener el WebSocket vivo
    /// Devuelve un stream de mensajes JSON parseados
    pub async fn connect(
        self: Arc<Self>,
    ) -> ExchangeResult<Pin<Box<dyn Stream<Item = ExchangeResult<serde_json::Value>> + Send>>> {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let should_run = self.should_run.clone();
        should_run.store(true, Ordering::Relaxed);
        let state = self.state.clone();
        let config = self.config.clone();
        let exchange = self.exchange_name.clone();
        let parser = self.parser.clone();
        let self_arc = self.clone();

        tokio::spawn(async move {
            let mut reconnect_attempts = 0u32;

            while should_run.load(Ordering::Relaxed) {
                *state.lock().await = WsState::Connecting;

                match connect_async(&config.url).await {
                    Ok((ws_stream, _response)) => {
                        *state.lock().await = WsState::Connected;
                        reconnect_attempts = 0;
                        tracing::info!("[{exchange}] WebSocket connected to {}", config.url);

                        let (mut write, mut read) = ws_stream.split();
                        let (ping_tx, mut ping_rx) = tokio::sync::mpsc::unbounded_channel::<()>();

                        // Tarea de heartbeat
                        let ping_interval = config.ping_interval_secs;
                        let ping_should_run = should_run.clone();
                        tokio::spawn(async move {
                            let mut interval =
                                tokio::time::interval(Duration::from_secs(ping_interval));
                            loop {
                                interval.tick().await;
                                if !ping_should_run.load(Ordering::Relaxed) {
                                    break;
                                }
                                let _ = ping_tx.send(());
                            }
                        });

                        // Bucle principal de mensajes
                        loop {
                            tokio::select! {
                                // Recibir mensaje del servidor
                                msg = read.next() => {
                                    match msg {
                                        Some(Ok(Message::Text(text))) => {
                                            // Parsear con el callback si existe
                                            if let Some(ref parser) = parser {
                                                match parser(text.clone()) {
                                                    Ok(Some(json)) => {
                                                        let _ = tx.send(Ok(json));
                                                    }
                                                    Ok(None) => {} // Mensaje ignorado (ping, etc)
                                                    Err(e) => {
                                                        tracing::warn!("[{exchange}] Parse error: {e}");
                                                    }
                                                }
                                            } else {
                                                // Sin parser, intentar parse directo
                                                match serde_json::from_str(&text) {
                                                    Ok(json) => { let _ = tx.send(Ok(json)); }
                                                    Err(e) => {
                                                        tracing::warn!("[{exchange}] JSON parse error: {e}");
                                                    }
                                                }
                                            }
                                        }
                                        Some(Ok(Message::Ping(data))) => {
                                            let _ = write.send(Message::Pong(data)).await;
                                        }
                                        Some(Ok(Message::Pong(_))) => {}
                                        Some(Ok(Message::Close(_))) => {
                                            tracing::info!("[{exchange}] WebSocket closed by server");
                                            break;
                                        }
                                        Some(Err(e)) => {
                                            tracing::warn!("[{exchange}] WebSocket error: {e}");
                                            break;
                                        }
                                        None => {
                                            tracing::info!("[{exchange}] WebSocket stream ended");
                                            break;
                                        }
                                        _ => {} // Binary, Frame - ignorar
                                    }
                                }
                                // Heartbeat
                                _ = ping_rx.recv() => {
                                    if let Err(e) = write.send(Message::Ping(vec![].into())).await {
                                        tracing::warn!("[{exchange}] Ping failed: {e}");
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        tracing::warn!("[{exchange}] Connection failed: {e}");
                    }
                }

                // Reconexión
                *state.lock().await = WsState::Reconnecting(reconnect_attempts);
                reconnect_attempts += 1;

                if reconnect_attempts > config.max_reconnect_attempts {
                    tracing::error!("[{exchange}] Max reconnection attempts reached");
                    let _ = tx.send(Err(ExchangeError::WebSocket {
                        exchange: exchange.clone(),
                        reason: "Max reconnection attempts reached".to_string(),
                    }));
                    break;
                }

                let delay = Duration::from_millis(config.reconnect_delay_ms);
                tokio::time::sleep(delay).await;
            }

            *state.lock().await = WsState::Disconnected;
        });

        // Convertimos el UnboundedReceiver en un Stream usando futures::stream::unfold
        // (cero nuevas dependencias — no usamos tokio-stream)
        let stream = unfold(
            rx,
            |mut rx: UnboundedReceiver<ExchangeResult<serde_json::Value>>| async move {
                rx.recv().await.map(|item| (item, rx))
            },
        );
        Ok(Box::pin(stream))
    }

    /// Desconectar
    pub fn disconnect(&self) {
        self.should_run.store(false, Ordering::Relaxed);
    }
}

impl Drop for WsManager {
    fn drop(&mut self) {
        self.should_run.store(false, Ordering::Relaxed);
    }
}
