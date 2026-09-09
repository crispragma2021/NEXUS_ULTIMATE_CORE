// ============================================================================
// 🧬 NEXUS PREDICCIÓN — Analizador Completo de Mercado
// ============================================================================
// Puente entre los ticks raw del mercado y el sistema de predicción:
//
//   Tick raw → Extraer features → Predecir (ML) → Fusionar (multi-fuente) → Señal
//
// También gestiona el aprendizaje continuo (retroalimentación).
// ============================================================================

use super::model::{FeatureVector, MotorPrediccion, Prediccion};
use super::fusion::{FusionMultiFuente, FuenteDatos, SenalFuente, SenalFusionada};
use serde::{Deserialize, Serialize};
use chrono::Utc;

/// Resultado completo del análisis de mercado
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalisisMercado {
    /// Símbolo analizado
    pub simbolo: String,
    /// Precio actual
    pub precio: f64,
    /// Predicción del modelo ML
    pub prediccion_ml: Prediccion,
    /// Señal fusionada multi-fuente
    pub senal_fusionada: SenalFusionada,
    /// Acción final determinada
    pub accion_final: String,
    /// Confianza final combinada
    pub confianza_final: f64,
    /// Timestamp
    pub timestamp: i64,
    /// El motor está listo (tiene datos suficientes)?
    pub listo: bool,
}

/// 📊 Analizador Completo — Cerebro analítico del trading
///
/// Integra:
/// - Motor de predicción ML (Regresión Logística + Red Neuronal)
/// - Fusionador multi-fuente (técnicos + sentimiento + order flow)
/// - Sistema de retroalimentación y aprendizaje continuo
pub struct AnalizadorCompleto {
    pub motor_ml: MotorPrediccion,
    pub fusionador: FusionMultiFuente,
    /// Historial de análisis para telemetría
    historial: Vec<AnalisisMercado>,
    /// Precios altos y bajos para indicadores que los requieren
    highs: Vec<f64>,
    lows: Vec<f64>,
    /// Volúmenes acumulados
    volumenes: Vec<f64>,
}

impl AnalizadorCompleto {
    pub fn new() -> Self {
        Self {
            motor_ml: MotorPrediccion::new(),
            fusionador: FusionMultiFuente::new(),
            historial: Vec::with_capacity(100),
            highs: Vec::with_capacity(200),
            lows: Vec::with_capacity(200),
            volumenes: Vec::with_capacity(200),
        }
    }

    /// Alimenta un tick al sistema completo
    pub fn alimentar_tick(&mut self, precio: f64, bid: f64, ask: f64, volumen: f64) {
        // Actualizar precios históricos en el motor ML
        self.motor_ml.alimentar(precio);

        // Acumular high/low
        let high = bid.max(ask).max(precio);
        let low = bid.min(ask).min(precio);
        self.highs.push(high);
        self.lows.push(low);
        self.volumenes.push(volumen);

        // Limitar tamaño de histórico
        if self.highs.len() > 200 { self.highs.remove(0); }
        if self.lows.len() > 200 { self.lows.remove(0); }
        if self.volumenes.len() > 200 { self.volumenes.remove(0); }
    }

    /// Extrae features del estado actual
    pub fn extraer_features(&self, precio: f64, bid: f64, ask: f64, volumen: f64) -> FeatureVector {
        self.motor_ml.extraer_features(
            precio,
            bid,
            ask,
            volumen,
            self.highs.last().copied(),
            self.lows.last().copied(),
        )
    }

    /// Ejecuta el análisis completo de mercado
    pub fn analizar(&mut self, simbolo: &str, precio: f64, bid: f64, ask: f64, volumen: f64) -> AnalisisMercado {
        // 1. Alimentar el tick
        self.alimentar_tick(precio, bid, ask, volumen);

        // 2. Extraer features
        let features = self.extraer_features(precio, bid, ask, volumen);

        // 3. Predecir con ML
        let prediccion_ml = self.motor_ml.predecir(&features);

        // 4. Generar señales de fuentes técnicas para el fusionador
        let senal_tecnicos = SenalFuente {
            fuente: FuenteDatos::Tecnicos,
            senal: self.calcular_senal_tecnica(&features, precio, bid, ask),
            confianza: 0.80,
            detalle: prediccion_ml.contribuciones.tecnicos.to_string(),
        };

        let senal_ml = SenalFuente {
            fuente: FuenteDatos::MachineLearning,
            senal: prediccion_ml.prob_subida,
            confianza: prediccion_ml.confianza,
            detalle: format!(
                "RL:{:.3} NN:{:.3}",
                prediccion_ml.contribuciones.regresion_logistica,
                prediccion_ml.contribuciones.red_neuronal,
            ),
        };

        let senal_orderflow = SenalFuente {
            fuente: FuenteDatos::OrderFlow,
            senal: self.calcular_senal_orderflow(bid, ask),
            confianza: 0.70,
            detalle: format!("Bid:{:.2} Ask:{:.2} Spread:{:.6}", bid, ask, ask - bid),
        };

        // 5. Fusionar todas las fuentes
        let fuentes = vec![senal_tecnicos, senal_ml, senal_orderflow];
        let senal_fusionada = self.fusionador.fusionar(fuentes);

        // 6. Determinar acción final
        let accion_final = senal_fusionada.accion.clone();
        let confianza_final = senal_fusionada.confianza;

        let analisis = AnalisisMercado {
            simbolo: simbolo.to_string(),
            precio,
            prediccion_ml,
            senal_fusionada,
            accion_final,
            confianza_final,
            timestamp: Utc::now().timestamp_millis(),
            listo: self.motor_ml.listo(),
        };

        self.historial.push(analisis.clone());
        if self.historial.len() > 100 {
            self.historial.remove(0);
        }

        analisis
    }

    /// Señal técnica combinada (0.0 - 1.0)
    fn calcular_senal_tecnica(&self, features: &FeatureVector, precio: f64, bid: f64, ask: f64) -> f64 {
        let mut signal = 0.5;
        let mut count = 0;

        // RSI
        if features.rsi < 30.0 { signal += 0.25; count += 1; }
        else if features.rsi > 70.0 { signal -= 0.25; count += 1; }

        // MACD
        if features.macd_histogram > 0.0 { signal += 0.15; count += 1; }
        else if features.macd_histogram < 0.0 { signal -= 0.15; count += 1; }

        // Spread
        let spread = (ask - bid) / precio;
        if spread < 0.0005 { signal += 0.10; count += 1; }
        else if spread > 0.005 { signal -= 0.10; count += 1; }

        // Pendiente
        signal += features.pendiente * 0.1;

        if count > 0 { signal = signal.clamp(0.05, 0.95); }
        signal
    }

    /// Señal de order flow basada en bid/ask
    fn calcular_senal_orderflow(&self, bid: f64, ask: f64) -> f64 {
        if ask <= 0.0 { return 0.5; }
        let ratio = bid / ask;
        // ratio > 0.999 → presión compradora → bullish
        // ratio < 0.995 → presión vendedora → bearish
        if ratio > 0.999 {
            0.65 + ((ratio - 0.999) * 50.0).min(0.30)
        } else if ratio < 0.995 {
            0.35 - ((0.995 - ratio) * 50.0).min(0.30)
        } else {
            0.5
        }
    }

    /// Retroalimentación: aprende del resultado real
    pub fn retroalimentar(&mut self, precio_anterior: f64, precio_actual: f64) {
        let subio = precio_actual > precio_anterior;
        let features = self.extraer_features(precio_actual, precio_actual * 0.999, precio_actual * 1.001, 0.0);
        
        // Retroalimentar motor ML
        self.motor_ml.retroalimentar(&features, subio);

        // Retroalimentar fusionador con cada fuente
        // (simplificado - idealmente debería registrar qué fuentes acertaron)
        self.fusionador.retroalimentar_fuente(FuenteDatos::MachineLearning, subio);
        self.fusionador.retroalimentar_fuente(FuenteDatos::Tecnicos, subio);
    }

    /// Reporte de estado del analizador completo
    pub fn reporte(&self) -> serde_json::Value {
        serde_json::json!({
            "motor_ml": {
                "listo": self.motor_ml.listo(),
                "precision": self.motor_ml.precision(),
                "predicciones": self.motor_ml.predicciones_realizadas,
                "aciertos": self.motor_ml.aciertos,
                "fallos": self.motor_ml.fallos,
                "historico_precios": self.motor_ml.historico_precios.len(),
            },
            "fusionador": self.fusionador.reporte(),
            "analisis_realizados": self.historial.len(),
            "ultimo_analisis": self.historial.last().map(|a| serde_json::json!({
                "simbolo": a.simbolo,
                "accion": a.accion_final,
                "confianza": a.confianza_final,
                "timestamp": a.timestamp,
            })),
        })
    }
}

impl Default for AnalizadorCompleto {
    fn default() -> Self {
        Self::new()
    }
}
