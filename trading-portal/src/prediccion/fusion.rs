// ============================================================================
// 🔮 NEXUS PREDICCIÓN — Fusión de Señales Multi-Fuente
// ============================================================================
// Combina múltiples fuentes de datos en una única señal de trading:
//   - Análisis técnico (RSI, MACD, Bollinger...)
//   - Machine Learning (Regresión Logística + Red Neuronal)
//   - Sentimiento de mercado (noticias, social)
//   - Datos on-chain (whales, exchange flows)
//   - Order flow (bid/ask imbalance, delta acumulado)
// ============================================================================

use serde::{Deserialize, Serialize};

/// Identificador de fuente de datos
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FuenteDatos {
    /// Indicadores técnicos (RSI, MACD, Bollinger, etc.)
    Tecnicos,
    /// Modelo ML interno (Regresión Logística + Red Neuronal)
    MachineLearning,
    /// Sentimiento de redes sociales / noticias
    Sentimiento,
    /// Datos on-chain (whales, exchange flows, supply distribution)
    OnChain,
    /// Order flow (bid/ask imbalance, CVD, delta)
    OrderFlow,
    /// Correlación entre mercados (BTC-USD, S&P 500, etc.)
    Correlacion,
    /// Señal de tendencia de largo plazo
    Tendencia,
}

impl std::fmt::Display for FuenteDatos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FuenteDatos::Tecnicos => write!(f, "📊 Técnicos"),
            FuenteDatos::MachineLearning => write!(f, "🤖 ML"),
            FuenteDatos::Sentimiento => write!(f, "💬 Sentimiento"),
            FuenteDatos::OnChain => write!(f, "⛓️ On-Chain"),
            FuenteDatos::OrderFlow => write!(f, "📈 Order Flow"),
            FuenteDatos::Correlacion => write!(f, "🔗 Correlación"),
            FuenteDatos::Tendencia => write!(f, "📉 Tendencia"),
        }
    }
}

/// Peso asignado a una fuente de datos
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PesoFuente {
    pub fuente: FuenteDatos,
    pub peso: f64,
    pub confianza_fuente: f64,
}

/// Señal proveniente de una fuente individual
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SenalFuente {
    pub fuente: FuenteDatos,
    /// Probabilidad de subida 0.0-1.0
    pub senal: f64,
    /// Confianza en esta fuente particular
    pub confianza: f64,
    /// Metadatos adicionales
    pub detalle: String,
}

/// Señal ya fusionada de todas las fuentes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SenalFusionada {
    /// Probabilidad de subida combinada (0.0 - 1.0)
    pub prob_subida: f64,
    /// Señal final: COMPRA / VENTA / NEUTRAL
    pub accion: String,
    /// Confianza consolidada
    pub confianza: f64,
    /// Desglose por fuente
    pub fuentes: Vec<SenalFuente>,
    /// Número de fuentes que votaron a favor
    pub votos_compra: u32,
    pub votos_venta: u32,
    /// Consenso: true si mayoría de fuentes coinciden
    pub consenso: bool,
    /// Timestamp
    pub timestamp: i64,
}

/// 🌐 Fusionador Multi-Fuente — Orquestador de señales heterogéneas
///
/// Recibe señales de N fuentes, las pondera según su
/// rendimiento histórico, y produce una señal de trading única.
pub struct FusionMultiFuente {
    /// Pesos actuales de cada fuente
    pesos: Vec<PesoFuente>,
    /// Historial de precisión por fuente (para adaptación dinámica)
    precision_historica: std::collections::HashMap<FuenteDatos, (u64, u64)>,
    /// Umbral mínimo de confianza para considerar una fuente
    umbral_minimo: f64,
    /// Señales generadas
    historial_senales: Vec<SenalFusionada>,
}

impl FusionMultiFuente {
    pub fn new() -> Self {
        let pesos = vec![
            PesoFuente { fuente: FuenteDatos::Tecnicos, peso: 0.25, confianza_fuente: 0.85 },
            PesoFuente { fuente: FuenteDatos::MachineLearning, peso: 0.35, confianza_fuente: 0.70 },
            PesoFuente { fuente: FuenteDatos::Sentimiento, peso: 0.10, confianza_fuente: 0.50 },
            PesoFuente { fuente: FuenteDatos::OnChain, peso: 0.10, confianza_fuente: 0.60 },
            PesoFuente { fuente: FuenteDatos::OrderFlow, peso: 0.15, confianza_fuente: 0.75 },
            PesoFuente { fuente: FuenteDatos::Correlacion, peso: 0.03, confianza_fuente: 0.40 },
            PesoFuente { fuente: FuenteDatos::Tendencia, peso: 0.02, confianza_fuente: 0.55 },
        ];

        Self {
            pesos,
            precision_historica: std::collections::HashMap::new(),
            umbral_minimo: 0.20,
            historial_senales: Vec::with_capacity(100),
        }
    }

    /// Normaliza los pesos para que sumen 1.0
    fn normalizar_pesos(&mut self) {
        let suma: f64 = self.pesos.iter().map(|p| p.peso * p.confianza_fuente).sum();
        if suma > 0.0 {
            for p in &mut self.pesos {
                p.peso = (p.peso * p.confianza_fuente) / suma;
            }
        }
    }

    /// Añade una señal de una fuente específica
    pub fn alimentar_senal(&mut self, senal: SenalFuente) {
        // Aquí se almacenaría la señal para procesamiento posterior
        // También se actualizaría la precisión histórica de la fuente
        
        let entry = self.precision_historica
            .entry(senal.fuente)
            .or_insert((0, 0));
        
        if senal.confianza > self.umbral_minimo {
            entry.0 += 1; // aciertos (simplificado)
        } else {
            entry.1 += 1; // fallos
        }
    }

    /// Genera una señal fusionada a partir de fuentes disponibles
    pub fn fusionar(&mut self, fuentes_disponibles: Vec<SenalFuente>) -> SenalFusionada {
        self.normalizar_pesos();

        let mut prob_ponderada = 0.0;
        let mut peso_total = 0.0;
        let mut votos_compra = 0u32;
        let mut votos_venta = 0u32;
        let mut fuentes_procesadas = Vec::new();

        for fuente in &fuentes_disponibles {
            // Buscar el peso configurado para esta fuente
            let peso_config = self.pesos.iter()
                .find(|p| p.fuente == fuente.fuente)
                .map(|p| p.peso)
                .unwrap_or(0.10);

            if fuente.confianza >= self.umbral_minimo {
                prob_ponderada += fuente.senal * peso_config;
                peso_total += peso_config;

                if fuente.senal > 0.55 {
                    votos_compra += 1;
                } else if fuente.senal < 0.45 {
                    votos_venta += 1;
                }

                fuentes_procesadas.push(fuente.clone());
            }
        }

        let prob_final = if peso_total > 0.0 {
            prob_ponderada / peso_total
        } else {
            0.5
        };

        let (accion, confianza) = if prob_final > 0.60 {
            ("COMPRA".to_string(), prob_final)
        } else if prob_final < 0.40 {
            ("VENTA".to_string(), 1.0 - prob_final)
        } else {
            ("NEUTRAL".to_string(), 0.5)
        };

        let total_votos = votos_compra + votos_venta;
        let consenso = total_votos > 0 && (votos_compra == 0 || votos_venta == 0);

        let senal = SenalFusionada {
            prob_subida: prob_final,
            accion,
            confianza,
            fuentes: fuentes_procesadas,
            votos_compra,
            votos_venta,
            consenso,
            timestamp: chrono::Utc::now().timestamp_millis(),
        };

        self.historial_senales.push(senal.clone());
        if self.historial_senales.len() > 100 {
            self.historial_senales.remove(0);
        }

        senal
    }

    /// Actualiza la precisión de una fuente después de conocer el resultado real
    pub fn retroalimentar_fuente(&mut self, fuente: FuenteDatos, acerto: bool) {
        let entry = self.precision_historica
            .entry(fuente)
            .or_insert((0, 0));

        if acerto {
            entry.0 += 1;
        } else {
            entry.1 += 1;
        }

        // Actualizar confianza de la fuente basado en precisión
        let total = entry.0 + entry.1;
        if total > 10 {
            let precision = entry.0 as f64 / total as f64;
            for p in &mut self.pesos {
                if p.fuente == fuente {
                    // La confianza de la fuente oscila alrededor de su precisión real
                    p.confianza_fuente = (p.confianza_fuente * 0.9 + precision * 0.1)
                        .clamp(0.1, 0.99);
                    break;
                }
            }
        }
    }

    /// Reporte de estado del fusionador
    pub fn reporte(&self) -> serde_json::Value {
        let fuentes: Vec<serde_json::Value> = self.pesos.iter().map(|p| {
            let precision = self.precision_historica.get(&p.fuente)
                .map(|(a, f)| if *a + *f > 0 { *a as f64 / (*a + *f) as f64 } else { 0.0 })
                .unwrap_or(0.0);

            serde_json::json!({
                "fuente": p.fuente.to_string(),
                "peso": p.peso,
                "confianza": p.confianza_fuente,
                "precision_historica": precision,
            })
        }).collect();

        serde_json::json!({
            "fuentes_activas": fuentes,
            "senales_generadas": self.historial_senales.len(),
            "umbral_minimo": self.umbral_minimo,
        })
    }
}

impl Default for FusionMultiFuente {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fusion_basica() {
        let mut fusionador = FusionMultiFuente::new();
        
        let fuentes = vec![
            SenalFuente {
                fuente: FuenteDatos::Tecnicos,
                senal: 0.75,
                confianza: 0.80,
                detalle: "RSI bullish + MACD positivo".to_string(),
            },
            SenalFuente {
                fuente: FuenteDatos::MachineLearning,
                senal: 0.65,
                confianza: 0.70,
                detalle: "Ensemble predice subida".to_string(),
            },
            SenalFuente {
                fuente: FuenteDatos::OrderFlow,
                senal: 0.55,
                confianza: 0.60,
                detalle: "Bid/Ask balance positivo".to_string(),
            },
        ];

        let resultado = fusionador.fusionar(fuentes);
        
        assert!(resultado.prob_subida > 0.5);
        assert_eq!(resultado.accion, "COMPRA");
        assert_eq!(resultado.votos_compra, 3);
        assert_eq!(resultado.votos_venta, 0);
        assert!(resultado.consenso);
    }

    #[test]
    fn test_fusion_sin_fuentes() {
        let mut fusionador = FusionMultiFuente::new();
        let resultado = fusionador.fusionar(vec![]);
        assert!((0.45..=0.55).contains(&resultado.prob_subida));
        assert_eq!(resultado.accion, "NEUTRAL");
    }

    #[test]
    fn test_fusion_venta_por_mayoria() {
        let mut fusionador = FusionMultiFuente::new();
        
        let fuentes = vec![
            SenalFuente {
                fuente: FuenteDatos::Tecnicos,
                senal: 0.25,
                confianza: 0.85,
                detalle: "RSI sobrecomprado".to_string(),
            },
            SenalFuente {
                fuente: FuenteDatos::MachineLearning,
                senal: 0.35,
                confianza: 0.75,
                detalle: "ML predice caída".to_string(),
            },
        ];

        let resultado = fusionador.fusionar(fuentes);
        assert_eq!(resultado.accion, "VENTA");
    }

    #[test]
    fn test_retroalimentacion_mejora_confianza() {
        let mut fusionador = FusionMultiFuente::new();
        let confianza_inicial = fusionador.pesos[0].confianza_fuente;
        
        // Simular 20 aciertos consecutivos de la fuente técnica
        for _ in 0..20 {
            fusionador.retroalimentar_fuente(FuenteDatos::Tecnicos, true);
        }
        
        let confianza_final = fusionador.pesos[0].confianza_fuente;
        assert!(confianza_final >= confianza_inicial, 
            "La confianza debería mejorar con aciertos: {:.3} → {:.3}", 
            confianza_inicial, confianza_final);
    }
}
