// 🔱 ccxt_rs — Transmutación Soberana del arsenal CCXT a Rust Puro
// Cero dependencias externas. Cero unwrap(). Cero Python.
// Arquitectura: Exchange trait → implementaciones por exchange (Binance, Kraken, Coinbase)

pub mod error;
pub mod exchange;
pub mod rest;
pub mod types;
pub mod ws;

pub use error::ExchangeError;
pub use exchange::*;
pub use types::*;
