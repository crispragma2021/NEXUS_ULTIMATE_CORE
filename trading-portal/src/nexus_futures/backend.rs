// ============================================================================
// nexus_futures::backend — Abstracción de backend de futuros
// ============================================================================
// Permite intercambiar entre:
//   - FuturesClient    (API real de Binance Futures, fapi.binance.com)
//   - FuturesSimulator (paper trading sin API, alimentado por el feed local)
//
// Se usa `BoxFuture` de futures-util (ya disponible) en lugar de async-trait,
// de modo que el orquestador y los handlers REST puedan guardar
// `Arc<dyn FuturesBackend>` sin dependencias nuevas.
//
// NOTA sobre lifetimes: los métodos con argumentos por referencia (symbol, req)
// clonan los argumentos dentro de `async move`; así el future resultante solo
// captura `&self` (la lifetime del trait object) + datos owned, y el trait no
// necesita parámetros de lifetime explícitos.
// ============================================================================

use futures_util::future::BoxFuture;

use super::types::*;
use super::client::FuturesClient;
use super::simulacion::FuturesSimulator;

/// Contrato mínimo compartido por el cliente real y el simulador.
/// Todas las firmas coinciden 1:1 con las de FuturesClient.
pub trait FuturesBackend: Send + Sync {
    /// GET /fapi/v2/account — balance + posiciones
    fn account_info(&self) -> BoxFuture<'_, anyhow::Result<FuturesAccountInfo>>;

    /// GET /fapi/v2/positionRisk — posiciones abiertas (opcional filtro por símbolo)
    fn positions(&self, symbol: Option<&str>) -> BoxFuture<'_, anyhow::Result<Vec<FuturesPosition>>>;

    /// POST /fapi/v1/leverage — cambiar leverage (1-125)
    fn set_leverage(&self, symbol: &str, leverage: u32) -> BoxFuture<'_, anyhow::Result<LeverageResponse>>;

    /// POST /fapi/v1/positionSide/dual — modo hedge (dual) o one-way
    fn set_position_mode(&self, dual: bool) -> BoxFuture<'_, anyhow::Result<serde_json::Value>>;

    /// POST /fapi/v1/order — colocar orden (MARKET/LIMIT/STOP/TP)
    fn place_order(&self, req: &FuturesOrderRequest) -> BoxFuture<'_, anyhow::Result<FuturesOrderResponse>>;

    /// DELETE /fapi/v1/allOpenOrders — cancelar todas las órdenes de un símbolo
    fn cancel_all_orders(&self, symbol: &str) -> BoxFuture<'_, anyhow::Result<serde_json::Value>>;

    /// GET /fapi/v1/openOrders — órdenes abiertas (opcional filtro por símbolo)
    fn open_orders(&self, symbol: Option<&str>) -> BoxFuture<'_, anyhow::Result<Vec<FuturesOrderResponse>>>;

    /// GET /fapi/v2/userTrades — historial de trades
    fn trade_history_v2(&self, symbol: &str, limit: Option<u32>) -> BoxFuture<'_, anyhow::Result<Vec<FuturesTrade>>>;

    /// GET /fapi/v1/premiumIndex + openInterest + topTraders — snapshot de mercado
    fn market_snapshot(&self, symbol: &str) -> BoxFuture<'_, anyhow::Result<MarketSnapshot>>;
}

impl FuturesBackend for FuturesClient {
    fn account_info(&self) -> BoxFuture<'_, anyhow::Result<FuturesAccountInfo>> {
        Box::pin(FuturesClient::account_info(self))
    }

    fn positions(&self, symbol: Option<&str>) -> BoxFuture<'_, anyhow::Result<Vec<FuturesPosition>>> {
        let symbol = symbol.map(|s| s.to_string());
        let me = self;
        Box::pin(async move { FuturesClient::positions(me, symbol.as_deref()).await })
    }

    fn set_leverage(&self, symbol: &str, leverage: u32) -> BoxFuture<'_, anyhow::Result<LeverageResponse>> {
        let symbol = symbol.to_string();
        let me = self;
        Box::pin(async move { FuturesClient::set_leverage(me, &symbol, leverage).await })
    }

    fn set_position_mode(&self, dual: bool) -> BoxFuture<'_, anyhow::Result<serde_json::Value>> {
        Box::pin(FuturesClient::set_position_mode(self, dual))
    }

    fn place_order(&self, req: &FuturesOrderRequest) -> BoxFuture<'_, anyhow::Result<FuturesOrderResponse>> {
        let req = req.clone();
        let me = self;
        Box::pin(async move { FuturesClient::place_order(me, &req).await })
    }

    fn cancel_all_orders(&self, symbol: &str) -> BoxFuture<'_, anyhow::Result<serde_json::Value>> {
        let symbol = symbol.to_string();
        let me = self;
        Box::pin(async move { FuturesClient::cancel_all_orders(me, &symbol).await })
    }

    fn open_orders(&self, symbol: Option<&str>) -> BoxFuture<'_, anyhow::Result<Vec<FuturesOrderResponse>>> {
        let symbol = symbol.map(|s| s.to_string());
        let me = self;
        Box::pin(async move { FuturesClient::open_orders(me, symbol.as_deref()).await })
    }

    fn trade_history_v2(&self, symbol: &str, limit: Option<u32>) -> BoxFuture<'_, anyhow::Result<Vec<FuturesTrade>>> {
        let symbol = symbol.to_string();
        let me = self;
        Box::pin(async move { FuturesClient::trade_history_v2(me, &symbol, limit).await })
    }

    fn market_snapshot(&self, symbol: &str) -> BoxFuture<'_, anyhow::Result<MarketSnapshot>> {
        let symbol = symbol.to_string();
        let me = self;
        Box::pin(async move { FuturesClient::market_snapshot(me, &symbol).await })
    }
}

impl FuturesBackend for FuturesSimulator {
    fn account_info(&self) -> BoxFuture<'_, anyhow::Result<FuturesAccountInfo>> {
        Box::pin(FuturesSimulator::account_info(self))
    }

    fn positions(&self, symbol: Option<&str>) -> BoxFuture<'_, anyhow::Result<Vec<FuturesPosition>>> {
        let symbol = symbol.map(|s| s.to_string());
        let me = self;
        Box::pin(async move { FuturesSimulator::positions(me, symbol.as_deref()).await })
    }

    fn set_leverage(&self, symbol: &str, leverage: u32) -> BoxFuture<'_, anyhow::Result<LeverageResponse>> {
        let symbol = symbol.to_string();
        let me = self;
        Box::pin(async move { FuturesSimulator::set_leverage(me, &symbol, leverage).await })
    }

    fn set_position_mode(&self, dual: bool) -> BoxFuture<'_, anyhow::Result<serde_json::Value>> {
        Box::pin(FuturesSimulator::set_position_mode(self, dual))
    }

    fn place_order(&self, req: &FuturesOrderRequest) -> BoxFuture<'_, anyhow::Result<FuturesOrderResponse>> {
        let req = req.clone();
        let me = self;
        Box::pin(async move { FuturesSimulator::place_order(me, &req).await })
    }

    fn cancel_all_orders(&self, symbol: &str) -> BoxFuture<'_, anyhow::Result<serde_json::Value>> {
        let symbol = symbol.to_string();
        let me = self;
        Box::pin(async move { FuturesSimulator::cancel_all_orders(me, &symbol).await })
    }

    fn open_orders(&self, symbol: Option<&str>) -> BoxFuture<'_, anyhow::Result<Vec<FuturesOrderResponse>>> {
        let symbol = symbol.map(|s| s.to_string());
        let me = self;
        Box::pin(async move { FuturesSimulator::open_orders(me, symbol.as_deref()).await })
    }

    fn trade_history_v2(&self, symbol: &str, limit: Option<u32>) -> BoxFuture<'_, anyhow::Result<Vec<FuturesTrade>>> {
        let symbol = symbol.to_string();
        let me = self;
        Box::pin(async move { FuturesSimulator::trade_history_v2(me, &symbol, limit).await })
    }

    fn market_snapshot(&self, symbol: &str) -> BoxFuture<'_, anyhow::Result<MarketSnapshot>> {
        let symbol = symbol.to_string();
        let me = self;
        Box::pin(async move { FuturesSimulator::market_snapshot(me, &symbol).await })
    }
}

/// Helper: convierte cualquier backend concreto en un trait object.
pub fn como_trait_object<T: FuturesBackend + 'static>(backend: std::sync::Arc<T>) -> std::sync::Arc<dyn FuturesBackend> {
    backend as std::sync::Arc<dyn FuturesBackend>
}
