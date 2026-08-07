// ============================================================================
// nexus_futures::ws — WebSocket streams para Binance Futures
// ============================================================================
// Market: wss://fstream.binance.com/ws (público, sin firma)
//          → book depth, trades agresivos (para CVD), mark price, funding rate
// User:   wss://fstream.binance.com/ws/<listenKey> (privado, requiere listenKey)
//          → ORDER_TRADE_UPDATE (fills), ACCOUNT_UPDATE (balance/posiciones)
// ============================================================================

use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use serde_json::Value;
use tokio::sync::mpsc;
use tokio_tungstenite::connect_async;
use tracing::{info, warn};

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

// ═══════════════════════════════════════════════════════════════════════════════
// Market WebSocket — Público (no requiere API keys)
// ═══════════════════════════════════════════════════════════════════════════════

/// Tipos de streams de mercado que podemos suscribir
#[derive(Debug, Clone)]
pub enum MarketStream {
    /// @depth20@100ms — libro de órdenes (20 niveles)
    Depth,
    /// @aggTrade — trades agresivos (útil para calcular CVD)
    AggTrade,
    /// @markPrice@1s — precio de marca cada 1 segundo
    MarkPrice,
    /// @fundingRate — funding rate del símbolo
    FundingRate,
    /// @bookTicker — mejor bid/ask en tiempo real
    BookTicker,
}

impl MarketStream {
    fn stream_name(&self, symbol: &str) -> String {
        let sym = symbol.to_lowercase();
        match self {
            MarketStream::Depth => format!("{}@depth20@100ms", sym),
            MarketStream::AggTrade => format!("{}@aggTrade", sym),
            MarketStream::MarkPrice => format!("{}@markPrice@1s", sym),
            MarketStream::FundingRate => format!("{}@fundingRate", sym),
            MarketStream::BookTicker => format!("{}@bookTicker", sym),
        }
    }
}

/// Cliente de WebSocket de mercado para Futures
pub struct FuturesMarketWs;

impl FuturesMarketWs {
    const BASE_WS: &str = "wss://fstream.binance.com/ws";

    /// Conecta a múltiples streams para un símbolo y envía los mensajes por un channel.
    /// Se reconecta automáticamente si la conexión cae.
    pub async fn subscribe(
        symbol: &str,
        streams: Vec<MarketStream>,
        tx: mpsc::UnboundedSender<Value>,
        shutdown: Arc<AtomicBool>,
    ) {
        let stream_names: Vec<String> = streams
            .iter()
            .map(|s| s.stream_name(symbol))
            .collect();

        let url = format!("{}/{}", Self::BASE_WS, stream_names.join("/"));
        info!("📡 [FUTURES-WS] Conectando a mercado: {}", url);

        loop {
            if shutdown.load(Ordering::Relaxed) {
                info!("🛑 [FUTURES-WS] Shutdown signal recibido, deteniendo mercado WS");
                break;
            }

            match tokio::time::timeout(
                tokio::time::Duration::from_secs(5),
                connect_async(&url),
            )
            .await
            {
                Ok(Ok((mut ws_stream, _))) => {
                    info!("✅ [FUTURES-WS] Conectado a streams de mercado para {}", symbol);

                    loop {
                        if shutdown.load(Ordering::Relaxed) {
                            break;
                        }

                        match tokio::time::timeout(
                            tokio::time::Duration::from_secs(30),
                            ws_stream.next(),
                        )
                        .await
                        {
                            Ok(Some(Ok(tokio_tungstenite::tungstenite::Message::Text(text)))) => {
                                if let Ok(val) = serde_json::from_str::<Value>(&text) {
                                    if tx.send(val).is_err() {
                                        warn!("⚠️ [FUTURES-WS] Canal de mercado cerrado");
                                        break;
                                    }
                                }
                            }
                            Ok(Some(Ok(tokio_tungstenite::tungstenite::Message::Ping(data)))) => {
                                let _ = ws_stream
                                    .send(tokio_tungstenite::tungstenite::Message::Pong(data))
                                    .await;
                            }
                            Ok(Some(Ok(_))) => {} // Binary/Pong/Close — ignorar
                            Ok(None) => {
                                warn!("⚠️ [FUTURES-WS] Stream de mercado cerrado por el servidor");
                                break;
                            }
                            Err(_) => {
                                warn!("⚠️ [FUTURES-WS] Timeout en stream de mercado, reconectando...");
                                break;
                            }
                            Ok(Some(Err(e))) => {
                                warn!("⚠️ [FUTURES-WS] Error en stream de mercado: {}", e);
                                break;
                            }
                        }
                    }
                }
                Ok(Err(e)) => {
                    warn!("⚠️ [FUTURES-WS] Error conectando a mercado: {}", e);
                }
                Err(_) => {
                    warn!("⚠️ [FUTURES-WS] Timeout conectando a mercado {}", symbol);
                }
            }

            if shutdown.load(Ordering::Relaxed) {
                break;
            }
            info!("🔄 [FUTURES-WS] Reconectando mercado {} en 3s...", symbol);
            tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
        }

        info!("👋 [FUTURES-WS] Stream de mercado {} finalizado", symbol);
    }

    /// Suscripción simple a un solo stream
    pub async fn subscribe_single(
        symbol: &str,
        stream: MarketStream,
        tx: mpsc::UnboundedSender<Value>,
        shutdown: Arc<AtomicBool>,
    ) {
        Self::subscribe(symbol, vec![stream], tx, shutdown).await;
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// User Data WebSocket — Privado (requiere listenKey)
// ═══════════════════════════════════════════════════════════════════════════════

/// Cliente de WebSocket de usuario para Futures (fills, posiciones, balance)
pub struct FuturesUserWs;

impl FuturesUserWs {
    const BASE_WS: &str = "wss://fstream.binance.com/ws";

    /// Conecta al stream de usuario con una listenKey.
    /// Envía eventos por `tx`. Requiere `keepalive_task` para renovar la listenKey cada 30 min.
    pub async fn connect(
        listen_key: &str,
        tx: mpsc::UnboundedSender<Value>,
        shutdown: Arc<AtomicBool>,
    ) {
        let url = format!("{}/{}", Self::BASE_WS, listen_key);
        info!("🔐 [FUTURES-USER-WS] Conectando a stream de usuario...");

        loop {
            if shutdown.load(Ordering::Relaxed) {
                break;
            }

            match tokio::time::timeout(
                tokio::time::Duration::from_secs(5),
                connect_async(&url),
            )
            .await
            {
                Ok(Ok((mut ws_stream, _))) => {
                    info!("✅ [FUTURES-USER-WS] Conectado a stream de usuario");

                    loop {
                        if shutdown.load(Ordering::Relaxed) {
                            break;
                        }

                        match tokio::time::timeout(
                            tokio::time::Duration::from_secs(30),
                            ws_stream.next(),
                        )
                        .await
                        {
                            Ok(Some(Ok(tokio_tungstenite::tungstenite::Message::Text(text)))) => {
                                if let Ok(val) = serde_json::from_str::<Value>(&text) {
                                    if tx.send(val).is_err() {
                                        warn!("⚠️ [FUTURES-USER-WS] Canal de usuario cerrado");
                                        break;
                                    }
                                }
                            }
                            Ok(Some(Ok(tokio_tungstenite::tungstenite::Message::Ping(data)))) => {
                                let _ = ws_stream
                                    .send(tokio_tungstenite::tungstenite::Message::Pong(data))
                                    .await;
                            }
                            Ok(Some(Ok(_))) => {}
                            Ok(None) => {
                                warn!("⚠️ [FUTURES-USER-WS] Stream de usuario cerrado (listenKey expirada?)");
                                break;
                            }
                            Err(_) => {
                                warn!("⚠️ [FUTURES-USER-WS] Timeout en stream de usuario, reconectando...");
                                break;
                            }
                            Ok(Some(Err(e))) => {
                                warn!("⚠️ [FUTURES-USER-WS] Error: {}", e);
                                break;
                            }
                        }
                    }
                }
                Ok(Err(e)) => {
                    warn!("⚠️ [FUTURES-USER-WS] Error conectando: {}", e);
                }
                Err(_) => {
                    warn!("⚠️ [FUTURES-USER-WS] Timeout conectando a stream de usuario");
                }
            }

            if shutdown.load(Ordering::Relaxed) {
                break;
            }
            info!("🔄 [FUTURES-USER-WS] Reconectando stream de usuario en 3s...");
            tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
        }

        info!("👋 [FUTURES-USER-WS] Stream de usuario finalizado");
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// CVD (Cumulative Volume Delta) — Calculado desde aggTrade
// ═══════════════════════════════════════════════════════════════════════════════

/// Mantiene el CVD (delta acumulado de volumen) a partir de trades agresivos.
/// `m` = maker (true → el trade fue iniciado por el comprador → la orden fue una venta agresiva).
/// Cuando `m == false` → compra agresiva → delta positivo.
pub struct CvdTracker {
    pub delta: f64,
}

impl CvdTracker {
    pub fn new() -> Self {
        Self { delta: 0.0 }
    }

    /// Procesa un aggTrade de Binance Futures y actualiza el delta.
    /// Retorna el CVD actualizado.
    pub fn process_agg_trade(&mut self, trade: &Value) -> f64 {
        // En aggTrade: "m" = maker side.
        // m == false → taker es buyer → compra agresiva → suma
        // m == true  → taker es seller → venta agresiva  → resta
        let qty = trade.get("q")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.0);
        let maker = trade.get("m")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        if !maker {
            self.delta += qty; // compra agresiva
        } else {
            self.delta -= qty; // venta agresiva
        }
        self.delta
    }

    pub fn reset(&mut self) {
        self.delta = 0.0;
    }
}
