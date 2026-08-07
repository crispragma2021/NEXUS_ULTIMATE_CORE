// 🔱 Exchange Trait — Contrato Soberano para todos los exchanges
// Cada exchange implementa este trait. El orquestador usa polimorfismo.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use futures::Stream;
use serde::{Deserialize, Serialize};
use std::pin::Pin;

use super::error::ExchangeResult;
use super::types::*;

pub mod binance;
pub mod coinbase;
pub mod kraken;

/// Interfaz unificada para cualquier exchange centralizado o descentralizado
#[async_trait]
pub trait Exchange: Send + Sync {
    /// Nombre del exchange (binance, kraken, coinbase, etc.)
    fn name(&self) -> &'static str;

    // ========== DATOS DE MERCADO (Público) ==========

    /// Obtener ticker actual de un símbolo
    async fn fetch_ticker(&self, symbol: &str) -> ExchangeResult<Ticker>;

    /// Obtener velas OHLCV históricas
    async fn fetch_ohlcv(
        &self,
        symbol: &str,
        timeframe: &Timeframe,
        limit: u32,
    ) -> ExchangeResult<Vec<OHLCV>>;

    /// Obtener el libro de órdenes (order book)
    async fn fetch_order_book(&self, symbol: &str, limit: u32) -> ExchangeResult<OrderBook>;

    /// Lista de símbolos/mercados disponibles
    async fn fetch_markets(&self) -> ExchangeResult<Vec<MarketInfo>>;

    /// Obtener tickers de múltiples símbolos en una sola llamada
    async fn fetch_tickers(&self, symbols: &[&str]) -> ExchangeResult<Vec<Ticker>> {
        let mut tickers = Vec::with_capacity(symbols.len());
        for symbol in symbols {
            match self.fetch_ticker(symbol).await {
                Ok(t) => tickers.push(t),
                Err(e) => {
                    tracing::warn!("[{}] Error fetching {}: {}", self.name(), symbol, e);
                }
            }
        }
        Ok(tickers)
    }

    // ========== COMERCIO (Autenticado) ==========

    /// Crear una orden
    async fn create_order(&self, order: Order) -> ExchangeResult<OrderResult>;

    /// Cancelar una orden por ID
    async fn cancel_order(&self, symbol: &str, order_id: &str) -> ExchangeResult<bool>;

    /// Obtener una orden por ID
    async fn fetch_order(&self, symbol: &str, order_id: &str) -> ExchangeResult<OrderResult>;

    /// Listar órdenes abiertas
    async fn fetch_open_orders(&self, symbol: &str) -> ExchangeResult<Vec<OrderResult>>;

    /// Obtener balance de la cuenta
    async fn fetch_balance(&self) -> ExchangeResult<Balance>;

    // ========== WEBSOCKET (Streaming en vivo) ==========

    /// Stream de tickers en tiempo real
    async fn watch_ticker(
        &self,
        symbol: &str,
    ) -> ExchangeResult<Pin<Box<dyn Stream<Item = ExchangeResult<Ticker>> + Send>>>;

    /// Stream de velas OHLCV en tiempo real
    async fn watch_ohlcv(
        &self,
        symbol: &str,
        timeframe: &Timeframe,
    ) -> ExchangeResult<Pin<Box<dyn Stream<Item = ExchangeResult<OHLCV>> + Send>>>;
}

// ========== TIPOS ADICIONALES ==========

/// Order Book (niveles de compra/venta)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBook {
    pub symbol: String,
    pub bids: Vec<Level>, // Precios de compra (sorted descendente)
    pub asks: Vec<Level>, // Precios de venta (sorted ascendente)
    pub timestamp: DateTime<Utc>,
}

/// Nivel individual en el order book
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Level {
    pub price: f64,
    pub quantity: f64,
}
