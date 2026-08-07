// 📊 ORDER BOOK DEPTH NEXUS — Escudo de Liquidez
// Análisis de la profundidad de mercado para evitar slippage y trampas.

use serde::{Deserialize, Serialize};
use tracing::{info, warn};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MarketLiquidity {
    pub bids_total: f64,
    pub asks_total: f64,
    pub spread: f64,
    pub wall_detected: bool,
}

pub struct DepthAnalyzer;

impl DepthAnalyzer {
    pub fn analizar_liquidez(bids: Vec<(f64, f64)>, asks: Vec<(f64, f64)>) -> MarketLiquidity {
        let bids_total: f64 = bids.iter().map(|(p, q)| p * q).sum();
        let asks_total: f64 = asks.iter().map(|(p, q)| p * q).sum();
        
        let best_bid = bids.first().map(|(p, _)| *p).unwrap_or(0.0);
        let best_ask = asks.first().map(|(p, _)| *p).unwrap_or(0.0);
        let spread = if best_bid > 0.0 { (best_ask - best_bid) / best_bid } else { 0.0 };

        // Detectar muros de compra (Paredes de liquidez)
        let wall_detected = bids_total > (asks_total * 1.5);

        MarketLiquidity {
            bids_total,
            asks_total,
            spread,
            wall_detected,
        }
    }
}
