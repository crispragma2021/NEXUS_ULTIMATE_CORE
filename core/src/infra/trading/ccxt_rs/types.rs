// 🔱 ccxt_rs — Tipos compartidos del arsenal de trading
// Ticker, OHLCV, Order, Balance, ExchangeInfo — todo tipado fuerte sin pérdida de precisión

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Representación de un ticker de mercado (precio actual + estadísticas 24h)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ticker {
    pub symbol: String,
    pub bid: f64,
    pub ask: f64,
    pub last: f64,
    pub high_24h: f64,
    pub low_24h: f64,
    pub volume_24h: f64,
    pub quote_volume_24h: f64,
    pub timestamp: DateTime<Utc>,
    pub change: f64,
    pub change_pct: f64,
}

/// Vela OHLCV (Open, High, Low, Close, Volume)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OHLCV {
    pub timestamp: DateTime<Utc>,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

/// Orden de trading
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub symbol: String,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub quantity: f64,
    pub price: Option<f64>,      // None para market orders
    pub stop_price: Option<f64>, // Stop-loss / take-profit
    pub client_order_id: Option<String>,
    pub time_in_force: Option<TimeInForce>,
}

/// Resultado de una orden ejecutada
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderResult {
    pub id: String,
    pub client_order_id: Option<String>,
    pub symbol: String,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub quantity: f64,
    pub filled_quantity: f64,
    pub price: f64,
    pub average_price: f64,
    pub status: OrderStatus,
    pub timestamp: DateTime<Utc>,
    pub fee: Option<Fee>,
}

/// Balance de una moneda específica
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceEntry {
    pub free: f64,
    pub used: f64,
    pub total: f64,
}

/// Balance completo de la cuenta
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Balance {
    pub assets: Vec<AssetBalance>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetBalance {
    pub currency: String,
    pub free: f64,
    pub used: f64,
    pub total: f64,
    pub usd_value: Option<f64>,
}

/// Información de un mercado/símbolo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketInfo {
    pub symbol: String,
    pub base: String,
    pub quote: String,
    pub active: bool,
    pub precision: MarketPrecision,
    pub limits: MarketLimits,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketPrecision {
    pub price: u32,
    pub quantity: u32,
    pub quote: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketLimits {
    pub quantity_min: f64,
    pub quantity_max: f64,
    pub price_min: f64,
    pub price_max: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fee {
    pub currency: String,
    pub cost: f64,
    pub rate: f64,
}

// --- Enums ---

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum OrderType {
    Market,
    Limit,
    StopLoss,
    StopLossLimit,
    TakeProfit,
    TakeProfitLimit,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum OrderStatus {
    Open,
    Closed,
    Canceled,
    Expired,
    Rejected,
    PartiallyFilled,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TimeInForce {
    GTC, // Good Til Cancelled
    IOC, // Immediate Or Cancel
    FOK, // Fill Or Kill
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Timeframe {
    M1,
    M3,
    M5,
    M15,
    M30,
    H1,
    H2,
    H4,
    H6,
    H8,
    H12,
    D1,
    D3,
    W1,
    MN1,
}

impl Timeframe {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::M1 => "1m",
            Self::M3 => "3m",
            Self::M5 => "5m",
            Self::M15 => "15m",
            Self::M30 => "30m",
            Self::H1 => "1h",
            Self::H2 => "2h",
            Self::H4 => "4h",
            Self::H6 => "6h",
            Self::H8 => "8h",
            Self::H12 => "12h",
            Self::D1 => "1d",
            Self::D3 => "3d",
            Self::W1 => "1w",
            Self::MN1 => "1M",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "1m" | "1min" => Some(Self::M1),
            "3m" | "3min" => Some(Self::M3),
            "5m" | "5min" => Some(Self::M5),
            "15m" | "15min" => Some(Self::M15),
            "30m" | "30min" => Some(Self::M30),
            "1h" | "1hour" => Some(Self::H1),
            "2h" | "2hour" => Some(Self::H2),
            "4h" | "4hour" => Some(Self::H4),
            "6h" | "6hour" => Some(Self::H6),
            "8h" | "8hour" => Some(Self::H8),
            "12h" | "12hour" => Some(Self::H12),
            "1d" | "1day" => Some(Self::D1),
            "3d" | "3day" => Some(Self::D3),
            "1w" | "1week" => Some(Self::W1),
            "1M" | "1month" => Some(Self::MN1),
            _ => None,
        }
    }
}

// --- Conversiones útiles ---

impl From<&str> for OrderSide {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "buy" | "long" | "bid" => Self::Buy,
            _ => Self::Sell,
        }
    }
}

impl std::fmt::Display for OrderSide {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Buy => write!(f, "buy"),
            Self::Sell => write!(f, "sell"),
        }
    }
}
