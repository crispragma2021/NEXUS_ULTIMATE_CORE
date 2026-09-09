// ============================================================================
// 🧠 NEXUS PREDICCIÓN — Módulo de Machine Learning y Señales de Trading
// ============================================================================
// Este módulo implementa el sistema de predicción multi-modelo de NEXUS:
//
// - 📊 indicators :: Indicadores técnicos clásicos (RSI, MACD, Bollinger, etc.)
// - 🤖 model      :: Modelos ML (Regresión Logística, Red Neuronal, Ensemble)
// - 🔮 fusion     :: Fusión de señales multi-fuente (técnicas + ML + sentimiento)
// - 🧬 analisis_completo :: Analizador completo que intega todo
//
// ⚡ Cero dependencias externas de ML. Todo implementado en Rust puro.
// ============================================================================

pub mod indicators;
pub mod model;
pub mod fusion;
pub mod analisis_completo;

// Re-export principales
pub use model::{MotorPrediccion, FeatureVector, Prediccion, LogisticRegression, RedNeuronal, EnsembleWeights};
pub use indicators::{rsi, macd, sma, ema, bollinger, atr, stochastic, clasificar_rsi, clasificar_macd};
pub use fusion::{FusionMultiFuente, FuenteDatos, SenalFusionada, PesoFuente};
pub use analisis_completo::{AnalizadorCompleto, AnalisisMercado};
