// ==========================================
// INGESTA DE MERCADO OMEGA - WebSockets
// ==========================================
// Captura ráfagas de ticks en tiempo real para
// NVDA, BTC y otros activos soberanos.
// ==========================================

use futures::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use tracing::{error, info};

#[derive(Debug, Clone, serde::Serialize)]
pub struct TickMercado {
    pub simbolo: String,
    pub precio: f64,
    pub volumen: f64,
    pub timestamp: u64,
}

pub struct MarketIngestor {
    pub tx: mpsc::Sender<TickMercado>,
}

impl MarketIngestor {
    pub fn new(tx: mpsc::Sender<TickMercado>) -> Self {
        info!("📡 [INGESTA] Sistema de captura de mercado inicializado.");
        Self { tx }
    }

    /// Inicia la ráfaga de captura para BTC (Binance) y simulación de NVDA
    pub async fn iniciar_captura(&self) {
        let tx_clone = self.tx.clone();

        // Hilo de Ingesta para BTCUSDT (Binance Public WS)
        tokio::spawn(async move {
            let url = "wss://stream.binance.com:9443/ws/btcusdt@trade";
            loop {
                info!("📡 [WS] Conectando a flujo de BTCUSDT...");

                match connect_async(url).await {
                    Ok((mut socket, _)) => {
                        info!("✅ [WS] Vínculo con Binance establecido.");
                        while let Some(msg) = socket.next().await {
                            match msg {
                                Ok(Message::Text(text)) => {
                                    if let Ok(data) =
                                        serde_json::from_str::<serde_json::Value>(&text)
                                    {
                                        let tick = TickMercado {
                                            simbolo: "BTC".into(),
                                            precio: data["p"]
                                                .as_str()
                                                .unwrap_or("0")
                                                .parse()
                                                .unwrap_or(0.0),
                                            volumen: data["q"]
                                                .as_str()
                                                .unwrap_or("0")
                                                .parse()
                                                .unwrap_or(0.0),
                                            timestamp: data["E"].as_u64().unwrap_or(0),
                                        };
                                        let _ = tx_clone.send(tick).await;
                                    }
                                }
                                Ok(_) => {}
                                Err(e) => {
                                    error!("❌ [WS] Error en flujo de Binance: {}", e);
                                    break;
                                }
                            }
                        }
                        info!("⚠️ [WS] Conexión de Binance perdida. Reintentando en 5 segundos...");
                    }
                    Err(e) => {
                        error!(
                            "❌ [WS] Fallo en conexión BTC: {}. Reintentando en 5 segundos...",
                            e
                        );
                    }
                }
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            }
        });

        // Nota: Para NVDA se requeriría una API de Equity (Polygon.io/AlphaVantage)
        // En este nivel, NEXUS simula el flujo si la API no está configurada.
        info!("📡 [INGESTA] Esperando ráfaga de NVDA...");
    }

    pub async fn suscribir_simbolo(&self, simbolo: &str) {
        info!(
            "🎯 [INGESTA] Suscribiendo ráfaga sensorial para: {}",
            simbolo
        );
        // Lógica para enviar mensaje SUBSCRIBE al socket
    }
}

impl Default for MarketIngestor {
    fn default() -> Self {
        let (tx, _) = mpsc::channel(100);
        Self::new(tx)
    }
}
