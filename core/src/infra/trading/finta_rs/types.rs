// 🔱 finta_rs — Tipos compartidos para indicadores técnicos

use serde::{Deserialize, Serialize};

/// Vela OHLCV (precios de entrada para los indicadores)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Candle {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

impl Candle {
    pub fn new(open: f64, high: f64, low: f64, close: f64, volume: f64) -> Self {
        Self {
            open,
            high,
            low,
            close,
            volume,
        }
    }
}

/// Conjunto de series de precios que necesita cualquier indicador
#[derive(Debug, Clone)]
pub struct PriceSeries {
    pub open: Vec<f64>,
    pub high: Vec<f64>,
    pub low: Vec<f64>,
    pub close: Vec<f64>,
    pub volume: Vec<f64>,
}

impl PriceSeries {
    pub fn from_klines(klines: &[Candle]) -> Self {
        let n = klines.len();
        let mut open = Vec::with_capacity(n);
        let mut high = Vec::with_capacity(n);
        let mut low = Vec::with_capacity(n);
        let mut close = Vec::with_capacity(n);
        let mut volume = Vec::with_capacity(n);

        for k in klines {
            open.push(k.open);
            high.push(k.high);
            low.push(k.low);
            close.push(k.close);
            volume.push(k.volume);
        }

        Self {
            open,
            high,
            low,
            close,
            volume,
        }
    }

    pub fn from_close(close: Vec<f64>) -> Self {
        let n = close.len();
        Self {
            open: vec![0.0; n],
            high: vec![0.0; n],
            low: vec![0.0; n],
            close,
            volume: vec![0.0; n],
        }
    }

    pub fn len(&self) -> usize {
        self.close.len()
    }

    pub fn is_empty(&self) -> bool {
        self.close.is_empty()
    }
}

/// Resultado de un indicador técnico
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndicatorResult {
    pub name: String,
    /// Valores principales del indicador
    pub values: Vec<f64>,
    /// Línea de señal (para MACD, Stochastic, etc.)
    pub signal: Option<Vec<f64>>,
    /// Histograma o línea extra (MACD histogram)
    pub extra: Option<Vec<f64>>,
}

/// Banda de Bollinger
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BollingerBands {
    pub upper: Vec<f64>,
    pub middle: Vec<f64>,
    pub lower: Vec<f64>,
}

/// Resultado de MACD
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MACDResult {
    pub macd_line: Vec<f64>,
    pub signal_line: Vec<f64>,
    pub histogram: Vec<f64>,
}

/// Resultado de Stochastic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StochasticResult {
    pub k: Vec<f64>, // %K línea rápida
    pub d: Vec<f64>, // %D línea lenta (señal)
}
