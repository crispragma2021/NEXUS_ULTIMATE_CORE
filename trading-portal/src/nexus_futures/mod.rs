// ============================================================================
// nexus_futures — Binance Futures USDT-M (fapi.binance.com)
// ============================================================================
// Largo/corto, leverage 1x-125x, SL/TP nativos, trailing stop,
// modo hedge (dual side), reduce-only, closePosition.
// Misma firma HMAC-SHA256 que el spot existente.
// ============================================================================

pub mod types;
pub mod client;
pub mod ws;
pub mod futures_loop;
pub mod simulacion;
pub mod backend;

pub use types::*;
pub use client::FuturesClient;
pub use ws::{FuturesMarketWs, FuturesUserWs};
pub use futures_loop::{FuturesOrchestrator, LlmBackend, LlmTradingDecision};
pub use simulacion::FuturesSimulator;
pub use backend::{FuturesBackend, como_trait_object};
