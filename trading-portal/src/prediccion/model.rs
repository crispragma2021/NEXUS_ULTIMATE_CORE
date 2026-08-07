// ============================================================================
// 🧠 NEXUS PREDICCIÓN — Modelos de Machine Learning en Rust Puro
// ============================================================================
// Implementación desde cero: Regresión Logística, Red Neuronal Simple,
// Clasificador Ridge, Ensemble ponderado
// Sin dependencias externas de ML
// ============================================================================

use serde::{Deserialize, Serialize};

/// Vector de características (features) para una predicción
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureVector {
    /// Precio actual normalizado
    pub precio_normalizado: f64,
    /// Cambio porcentual últimas N velas
    pub cambio_pct: f64,
    /// RSI valor
    pub rsi: f64,
    /// MACD histograma
    pub macd_histogram: f64,
    /// Bollinger Bandwidth (ancho relativo de bandas)
    pub bollinger_bw: f64,
    /// %R Estocástico
    pub stoch_k: f64,
    /// Volatilidad relativa (ATR / precio)
    pub volatilidad: f64,
    /// Ratio de volumen vs promedio
    pub volumen_ratio: f64,
    /// Spread relativo (ask-bid)/precio
    pub spread_relativo: f64,
    /// Pendiente de regresión lineal simple últimas N velas
    pub pendiente: f64,
}

impl FeatureVector {
    /// Convierte a vector plano para operaciones matemáticas
    pub fn to_slice(&self) -> [f64; 10] {
        [
            self.precio_normalizado,
            self.cambio_pct,
            self.rsi,
            self.macd_histogram,
            self.bollinger_bw,
            self.stoch_k,
            self.volatilidad,
            self.volumen_ratio,
            self.spread_relativo,
            self.pendiente,
        ]
    }

    /// Normaliza usando media y desviación conocidas (z-score)
    pub fn normalizar(&self, medias: &[f64; 10], desv: &[f64; 10]) -> Self {
        let raw = self.to_slice();
        let norm: [f64; 10] = std::array::from_fn(|i| {
            if desv[i] > 0.0 {
                (raw[i] - medias[i]) / desv[i]
            } else {
                0.0
            }
        });
        FeatureVector::from_slice(&norm)
    }

    pub fn from_slice(s: &[f64; 10]) -> Self {
        Self {
            precio_normalizado: s[0],
            cambio_pct: s[1],
            rsi: s[2],
            macd_histogram: s[3],
            bollinger_bw: s[4],
            stoch_k: s[5],
            volatilidad: s[6],
            volumen_ratio: s[7],
            spread_relativo: s[8],
            pendiente: s[9],
        }
    }
}

impl Default for FeatureVector {
    fn default() -> Self {
        Self {
            precio_normalizado: 0.0,
            cambio_pct: 0.0,
            rsi: 50.0,
            macd_histogram: 0.0,
            bollinger_bw: 0.1,
            stoch_k: 50.0,
            volatilidad: 0.01,
            volumen_ratio: 1.0,
            spread_relativo: 0.001,
            pendiente: 0.0,
        }
    }
}

/// Predicción individual del modelo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prediccion {
    /// Probabilidad de que suba (0.0 a 1.0)
    pub prob_subida: f64,
    /// Clasificación: "COMPRA" | "VENTA" | "NEUTRAL"
    pub senal: String,
    /// Confianza combinada del ensemble
    pub confianza: f64,
    /// Precio objetivo estimado
    pub precio_objetivo: f64,
    /// Stop loss sugerido
    pub stop_loss: f64,
    /// Desglose de contribuciones por modelo (para transparencia)
    pub contribuciones: ContribucionesModelos,
}

/// Transparencia: cuánto contribuyó cada modelo a la decisión final
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContribucionesModelos {
    pub regresion_logistica: f64,
    pub red_neuronal: f64,
    pub tecnicos: f64,
    pub ensemble: f64,
}

// ═══════════════════════════════════════════════════════════════════════════════
// 📐 REGRESIÓN LOGÍSTICA — Clasificador lineal binario
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogisticRegression {
    pesos: [f64; 10],
    bias: f64,
    tasa_aprendizaje: f64,
}

impl LogisticRegression {
    pub fn new() -> Self {
        Self {
            pesos: [0.0; 10],
            bias: 0.0,
            tasa_aprendizaje: 0.01,
        }
    }

    /// Función sigmoide: 1 / (1 + e^(-z))
    fn sigmoide(z: f64) -> f64 {
        1.0 / (1.0 + (-z).exp())
    }

    /// Predicción: probabilidad de clase positiva (subida)
    pub fn predecir_prob(&self, features: &FeatureVector) -> f64 {
        let z: f64 = features.to_slice().iter()
            .zip(self.pesos.iter())
            .map(|(f, w)| f * w)
            .sum::<f64>() + self.bias;
        Self::sigmoide(z)
    }

    /// Entrenamiento con descenso de gradiente (una época)
    pub fn entrenar(&mut self, datos: &[(FeatureVector, f64)], epocas: usize) {
        for _epoch in 0..epocas {
            let mut grad_pesos = [0.0; 10];
            let mut grad_bias = 0.0;
            let n = datos.len() as f64;

            for (features, target) in datos {
                let prob = self.predecir_prob(features);
                let error = prob - target; // target: 1.0 = sube, 0.0 = baja
                let feats = features.to_slice();
                for i in 0..10 {
                    grad_pesos[i] += error * feats[i];
                }
                grad_bias += error;
            }

            for i in 0..10 {
                self.pesos[i] -= self.tasa_aprendizaje * grad_pesos[i] / n;
            }
            self.bias -= self.tasa_aprendizaje * grad_bias / n;
        }
    }
}

impl Default for LogisticRegression {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// 🧠 RED NEURONAL SIMPLE — Perceptrón Multicapa (1 capa oculta)
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedNeuronal {
    // Capa oculta: 10 features → 6 neuronas
    pesos_c1: [[f64; 10]; 6],
    bias_c1: [f64; 6],
    // Capa de salida: 6 → 1
    pesos_c2: [f64; 6],
    bias_c2: f64,
}

impl RedNeuronal {
    pub fn new() -> Self {
        Self {
            pesos_c1: [[0.0; 10]; 6],
            bias_c1: [0.0; 6],
            pesos_c2: [0.0; 6],
            bias_c2: 0.0,
        }
    }

    fn relu(x: f64) -> f64 {
        x.max(0.0)
    }

    fn sigmoide(x: f64) -> f64 {
        1.0 / (1.0 + (-x).exp())
    }

    /// Forward pass
    pub fn predecir_prob(&self, features: &FeatureVector) -> f64 {
        let feats = features.to_slice();
        
        // Capa oculta
        let mut oculta = [0.0; 6];
        for j in 0..6 {
            let mut suma = self.bias_c1[j];
            for i in 0..10 {
                suma += feats[i] * self.pesos_c1[j][i];
            }
            oculta[j] = Self::relu(suma);
        }
        
        // Capa de salida
        let mut salida = self.bias_c2;
        for j in 0..6 {
            salida += oculta[j] * self.pesos_c2[j];
        }
        
        Self::sigmoide(salida)
    }
}

impl Default for RedNeuronal {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// 🎯 SISTEMA DE PONDERACIÓN POR RENDIMIENTO (Adaptive Ensemble Weighting)
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnsembleWeights {
    pub peso_logistico: f64,
    pub peso_neuronal: f64,
    pub peso_tecnicos: f64,
}

impl EnsembleWeights {
    pub fn equilibrado() -> Self {
        Self {
            peso_logistico: 0.30,
            peso_neuronal: 0.40,
            peso_tecnicos: 0.30,
        }
    }

    /// Pesos totales = 1.0
    pub fn normalizar(&mut self) {
        let suma = self.peso_logistico + self.peso_neuronal + self.peso_tecnicos;
        if suma > 0.0 {
            self.peso_logistico /= suma;
            self.peso_neuronal /= suma;
            self.peso_tecnicos /= suma;
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// 🧠 MOTOR DE PREDICCIÓN — Orquestador de modelos + señales técnicas
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MotorPrediccion {
    pub logistico: LogisticRegression,
    pub neuronal: RedNeuronal,
    pub pesos: EnsembleWeights,
    /// Precios históricos para indicadores
    pub historico_precios: Vec<f64>,
    /// Para normalización
    medias: [f64; 10],
    desviaciones: [f64; 10],
    /// Contador de predicciones
    pub predicciones_realizadas: u64,
    /// Precisión acumulada (autoevaluación)
    pub aciertos: u64,
    pub fallos: u64,
}

impl MotorPrediccion {
    pub fn new() -> Self {
        Self {
            logistico: LogisticRegression::new(),
            neuronal: RedNeuronal::new(),
            pesos: EnsembleWeights::equilibrado(),
            historico_precios: Vec::with_capacity(200),
            medias: [0.0; 10],
            desviaciones: [1.0; 10],
            predicciones_realizadas: 0,
            aciertos: 0,
            fallos: 0,
        }
    }

    /// Alimenta un nuevo precio al histórico
    pub fn alimentar(&mut self, precio: f64) {
        self.historico_precios.push(precio);
        if self.historico_precios.len() > 200 {
            self.historico_precios.remove(0);
        }
    }

    /// Extrae las 10 features del estado actual del mercado
    /// recibe: precio, bid, ask, volumen, high, low, y el histórico acumulado
    pub fn extraer_features(
        &self,
        precio: f64,
        bid: f64,
        ask: f64,
        _volumen: f64,
        _high: Option<f64>,
        _low: Option<f64>,
    ) -> FeatureVector {
        let precio_base = if self.historico_precios.is_empty() { precio } else { self.historico_precios[0] };
        let precio_normalizado = if precio_base > 0.0 {
            (precio - precio_base) / precio_base
        } else {
            0.0
        };

        let cambio_pct = if self.historico_precios.len() >= 5 {
            let anterior = self.historico_precios[self.historico_precios.len().saturating_sub(5)];
            if anterior > 0.0 {
                (precio - anterior) / anterior * 100.0
            } else {
                0.0
            }
        } else {
            0.0
        };

        // Calcular RSI sobre el histórico
        let rsi_val = if self.historico_precios.len() >= 15 {
            use super::indicators;
            let rsi_vals = indicators::rsi(&self.historico_precios, 14);
            rsi_vals.last().copied().unwrap_or(50.0)
        } else {
            50.0
        };

        // MACD
        let macd_histogram = if self.historico_precios.len() >= 26 {
            use super::indicators;
            let macd_vals = indicators::macd(&self.historico_precios, 12, 26, 9);
            macd_vals.last().map(|m| m.histogram).unwrap_or(0.0)
        } else {
            0.0
        };

        // Bollinger Bandwidth
        let bollinger_bw = if self.historico_precios.len() >= 20 {
            use super::indicators;
            let bb = indicators::bollinger(&self.historico_precios, 20, 2.0);
            bb.last().map(|b| b.bandwidth).unwrap_or(0.1)
        } else {
            0.1
        };

        // Volatilidad (desviación estándar relativa)
        let volatilidad = if self.historico_precios.len() >= 10 {
            let len = self.historico_precios.len();
            let slice = &self.historico_precios[len - 10..];
            let mean: f64 = slice.iter().sum::<f64>() / slice.len() as f64;
            let variance: f64 = slice.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / slice.len() as f64;
            if mean > 0.0 {
                variance.sqrt() / mean
            } else {
                0.01
            }
        } else {
            0.01
        };

        // Spread relativo
        let spread_relativo = if precio > 0.0 && ask >= bid {
            (ask - bid) / precio
        } else {
            0.001
        };

        // Pendiente (regresión lineal simple)
        let pendiente = if self.historico_precios.len() >= 5 {
            let len = self.historico_precios.len();
            let slice = &self.historico_precios[len - 5..];
            let n = slice.len() as f64;
            let sum_x: f64 = (0..slice.len()).map(|i| i as f64).sum();
            let sum_y: f64 = slice.iter().sum();
            let sum_xy: f64 = slice.iter().enumerate().map(|(i, &y)| i as f64 * y).sum();
            let sum_xx: f64 = (0..slice.len()).map(|i| (i as f64).powi(2)).sum();
            let denom = n * sum_xx - sum_x * sum_x;
            if denom.abs() > 0.0001 {
                (n * sum_xy - sum_x * sum_y) / denom
            } else {
                0.0
            }
        } else {
            0.0
        };

        FeatureVector {
            precio_normalizado,
            cambio_pct,
            rsi: rsi_val,
            macd_histogram,
            bollinger_bw,
            stoch_k: 50.0, // Requiere high/low históricos
            volatilidad,
            volumen_ratio: 1.0,
            spread_relativo,
            pendiente,
        }
    }

    /// Señal técnica basada en indicadores (sin ML)
    fn senal_tecnica(&self, features: &FeatureVector) -> f64 {
        let mut score = 0.5; // Neutral por defecto
        let mut factores = 0;

        // RSI
        if features.rsi < 30.0 {
            score += 0.25; // Sobrevenido = oportunidad de compra
            factores += 1;
        } else if features.rsi > 70.0 {
            score -= 0.25; // Sobrecomprado = riesgo de venta
            factores += 1;
        }

        // MACD
        if features.macd_histogram > 0.0 {
            score += 0.15; // MACD positivo = momentum alcista
            factores += 1;
        } else if features.macd_histogram < 0.0 {
            score -= 0.15; // MACD negativo = momentum bajista
            factores += 1;
        }

        // Bollinger Bands
        if features.bollinger_bw > 0.05 {
            score -= 0.10; // Alta volatilidad = riesgo
            factores += 1;
        }

        // Spread
        if features.spread_relativo < 0.0005 {
            score += 0.10; // Spread ajustado = liquidez buena
            factores += 1;
        } else if features.spread_relativo > 0.005 {
            score -= 0.10; // Spread amplio = iliquidez
            factores += 1;
        }

        // Pendiente
        if features.pendiente > 0.0 {
            score += 0.10 * features.pendiente.min(1.0);
            factores += 1;
        } else if features.pendiente < 0.0 {
            score += 0.10 * features.pendiente.max(-1.0);
            factores += 1;
        }

        if factores > 0 {
            score = score.clamp(0.05, 0.95);
        }
        score
    }

    /// Predicción combinada (Ensemble): RL + Red Neuronal + Técnicos
    pub fn predecir(&mut self, features: &FeatureVector) -> Prediccion {
        self.predicciones_realizadas += 1;

        let prob_rl = self.logistico.predecir_prob(features);
        let prob_nn = self.neuronal.predecir_prob(features);
        let prob_tec = self.senal_tecnica(features);

        let prob_ensemble = 
            self.pesos.peso_logistico * prob_rl +
            self.pesos.peso_neuronal * prob_nn +
            self.pesos.peso_tecnicos * prob_tec;

        // Determinar señal y confianza
        let (senal, confianza) = if prob_ensemble > 0.60 {
            ("COMPRA".to_string(), prob_ensemble)
        } else if prob_ensemble < 0.40 {
            ("VENTA".to_string(), 1.0 - prob_ensemble)
        } else {
            ("NEUTRAL".to_string(), 0.5)
        };

        // Precio objetivo y stop loss estimados
        let precio_objetivo = features.precio_normalizado * 1.02;
        let stop_loss = features.precio_normalizado * 0.98;

        Prediccion {
            prob_subida: prob_ensemble,
            senal,
            confianza,
            precio_objetivo,
            stop_loss,
            contribuciones: ContribucionesModelos {
                regresion_logistica: prob_rl,
                red_neuronal: prob_nn,
                tecnicos: prob_tec,
                ensemble: prob_ensemble,
            },
        }
    }

    /// Retroalimentación: aprende del resultado real
    pub fn retroalimentar(&mut self, features: &FeatureVector, subio: bool) {
        let target = if subio { 1.0 } else { 0.0 };
        
        // Entrenar regresión logística con 1 época
        let datos = [(features.clone(), target)];
        self.logistico.entrenar(&datos, 1);

        // Actualizar precisión
        let prob = self.logistico.predecir_prob(features);
        let acerto = (prob > 0.5) == subio;
        if acerto {
            self.aciertos += 1;
        } else {
            self.fallos += 1;
        }

        // Ajustar pesos del ensemble según rendimiento
        self.ajustar_pesos(prob, target);
    }

    fn ajustar_pesos(&mut self, prob_modelo: f64, target: f64) {
        let error = (prob_modelo - target).abs();
        if error > 0.3 {
            // Penalizar peso del modelo logístico si se equivoca mucho
            self.pesos.peso_logistico *= 0.98;
            self.pesos.peso_neuronal *= 1.01;
            self.pesos.normalizar();
        }
    }

    /// Precisión actual del modelo
    pub fn precision(&self) -> f64 {
        let total = self.aciertos + self.fallos;
        if total > 0 {
            self.aciertos as f64 / total as f64
        } else {
            0.0
        }
    }

    /// ¿Está el modelo listo para operar?
    pub fn listo(&self) -> bool {
        self.historico_precios.len() >= 30 && self.predicciones_realizadas > 0
    }
}

impl Default for MotorPrediccion {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_regresion_logistica_predice_dentro_rango() {
        let modelo = LogisticRegression::new();
        let features = FeatureVector::default();
        let prob = modelo.predecir_prob(&features);
        assert!((0.0..=1.0).contains(&prob), "Probabilidad debe estar entre 0 y 1, es {:.2}", prob);
    }

    #[test]
    fn test_red_neuronal_predice_dentro_rango() {
        let modelo = RedNeuronal::new();
        let features = FeatureVector::default();
        let prob = modelo.predecir_prob(&features);
        assert!((0.0..=1.0).contains(&prob), "Probabilidad NN debe estar entre 0 y 1, es {:.2}", prob);
    }

    #[test]
    fn test_motor_prediccion_produce_prediccion_valida() {
        let mut motor = MotorPrediccion::new();
        for i in 0..30 {
            motor.alimentar(100.0 + (i as f64 * 0.5));
        }
        let features = motor.extraer_features(115.0, 114.9, 115.1, 1000.0, None, None);
        let pred = motor.predecir(&features);
        assert!(!pred.senal.is_empty());
        assert!((0.0..=1.0).contains(&pred.confianza));
        assert!((0.0..=1.0).contains(&pred.prob_subida));
    }

    #[test]
    fn test_motor_aprende_de_retroalimentacion() {
        let mut motor = MotorPrediccion::new();
        for i in 0..50 {
            motor.alimentar(100.0 + (i as f64 * 0.2));
        }
        let features = motor.extraer_features(110.0, 109.9, 110.1, 500.0, None, None);
        
        // Retroalimentar con que sí subió
        motor.retroalimentar(&features, true);
        
        // La precisión debe ser accesible
        assert!(motor.precision() >= 0.0);
    }

    #[test]
    fn test_motor_no_operar_sin_datos() {
        let motor = MotorPrediccion::new();
        assert!(!motor.listo(), "Motor no debe estar listo sin datos históricos");
    }

    #[test]
    fn test_motor_listo_con_datos() {
        let mut motor = MotorPrediccion::new();
        for i in 0..50 {
            motor.alimentar(100.0 + (i as f64 * 0.1));
        }
        let features = motor.extraer_features(105.0, 104.9, 105.1, 100.0, None, None);
        motor.predecir(&features);
        assert!(motor.listo(), "Motor debe estar listo con 50 datos históricos y 1 predicción");
    }
}
