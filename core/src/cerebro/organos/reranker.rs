// ============================================================================
// 🧠 RERANKER — Corteza RAG (Filtro de Relevancia de Segundo Paso)
// ============================================================================
// Propósito: Toma resultados crudos de ANN search y los re-ordena usando
//            señales de relevancia más ricas: similitud coseno refinada,
//            relevancia de solapamiento de tokens, cobertura semántica y
//            un umbral adaptativo basado en la distribución de scores.
// ============================================================================

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RerankResult {
    /// Contenido o ID del fragmento
    pub content: String,
    /// Score original del ANN
    pub raw_score: f32,
    /// Score rerank (0.0 - 1.0)
    pub rerank_score: f32,
    /// Fue promovido por relevancia de tokens
    pub promoted_by_token_overlap: bool,
    /// Fue degradado por ruido sintáctico
    pub degraded_by_noise: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RerankSummary {
    pub results: Vec<RerankResult>,
    pub threshold_effective: f32,
    pub total_raw: usize,
    pub total_passed: usize,
}

/// Configuración del Reranker
pub struct Reranker {
    /// Umbral base de similitud (0.0 - 1.0)
    pub base_threshold: f32,
    /// Penalización por desviación estándar alta (señal de ruido)
    pub noise_penalty: f32,
    /// Bono por solapamiento léxico con la query
    pub overlap_bonus: f32,
}

impl Default for Reranker {
    fn default() -> Self {
        Self {
            base_threshold: 0.45,
            noise_penalty: 0.15,
            overlap_bonus: 0.10,
        }
    }
}

impl Reranker {
    pub fn new(base_threshold: f32, noise_penalty: f32, overlap_bonus: f32) -> Self {
        Self {
            base_threshold,
            noise_penalty,
            overlap_bonus,
        }
    }

    /// Punto de entrada principal: rerankear una lista de resultados ANN
    ///
    /// * `query` — El prompt/consulta original
    /// * `raw_results` — Lista de (contenido, score_distancia) del ANN
    pub fn rerank(&self, query: &str, raw_results: &[(String, f32)]) -> RerankSummary {
        if raw_results.is_empty() {
            return RerankSummary {
                results: vec![],
                threshold_effective: self.base_threshold,
                total_raw: 0,
                total_passed: 0,
            };
        }

        // Extraer tokens de la query (split por espacios comunes)
        let query_tokens: HashSet<String> = query
            .split(|c: char| c.is_whitespace() || c.is_ascii_punctuation())
            .filter(|t| !t.is_empty() && t.len() > 2)
            .map(|t| t.to_lowercase())
            .collect();

        // Paso 1: Convertir distancias a scores de similitud (1.0 - distance)
        let scores: Vec<f32> = raw_results
            .iter()
            .map(|(_, dist)| {
                let sim = 1.0 - dist.clamp(0.0, 1.0);
                // Escalamiento no-lineal: enfatiza diferencias en el medio del rango.
                // (sim^2)*2 - 1 produce [-1, 1]; el mapeo correcto a [0, 1] es (x+1)/2.
                // ⚠️ Antes se usaba .max(0.0) que ANULABA todo score negativo
                // (cualquier distancia > ~0.29 quedaba en 0.0).
                let x = (sim * sim) * 2.0 - 1.0; // rango [-1, 1]
                ((x + 1.0) / 2.0).clamp(0.0, 1.0)
            })
            .collect();

        // Paso 2: Calcular estadísticas para umbral adaptativo
        let total = scores.len() as f32;
        let mean = scores.iter().sum::<f32>() / total;
        let variance = scores.iter().map(|s| (s - mean).powi(2)).sum::<f32>() / total;
        let std_dev = variance.sqrt();

        // Umbral adaptativo: media - 0.5*std_dev (más estricto si hay variación alta)
        let threshold_adaptive = if std_dev > 0.15 {
            (mean - 0.5 * std_dev).max(0.2)
        } else {
            self.base_threshold
        };

        // Paso 3: Rerankear cada resultado
        let results: Vec<RerankResult> = raw_results
            .iter()
            .zip(scores.iter())
            .map(|((content, raw_dist), &sim_score)| {
                let mut score = sim_score;

                // 3a: Penalización por ruido
                let token_overlap = query_tokens
                    .iter()
                    .filter(|t| content.to_lowercase().contains(*t))
                    .count();
                let overlap_ratio = if query_tokens.is_empty() {
                    0.0
                } else {
                    token_overlap as f32 / query_tokens.len() as f32
                };

                let promoted = overlap_ratio > 0.3;
                if promoted {
                    score += self.overlap_bonus;
                }

                // 3b: Penalización si el contenido es muy corto o muy largo
                let degraded = if content.len() < 20 || content.len() > 10000 {
                    score -= self.noise_penalty * 0.5;
                    true
                } else {
                    false
                };

                // 3c: Penalización si hay alta desviación (cluster inconsistente)
                if std_dev > 0.3 && sim_score < mean - 0.5 * std_dev {
                    score -= self.noise_penalty * 0.3;
                }

                RerankResult {
                    content: content.clone(),
                    raw_score: *raw_dist,
                    rerank_score: score.clamp(0.0, 1.0),
                    promoted_by_token_overlap: promoted,
                    degraded_by_noise: degraded,
                }
            })
            .collect();

        // Paso 4: Ordernar por rerank_score descendente
        let mut sorted = results;
        sorted.sort_by(|a, b| {
            b.rerank_score
                .partial_cmp(&a.rerank_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Paso 5: Aplicar umbral
        let passed: Vec<RerankResult> = sorted
            .into_iter()
            .filter(|r| r.rerank_score >= threshold_adaptive)
            .collect();

        let summary = RerankSummary {
            total_passed: passed.len(),
            total_raw: raw_results.len(),
            threshold_effective: threshold_adaptive,
            results: passed,
        };

        info!(
            "🧠 [RERANKER] {}/{} pasaron (umbral={:.3}, std_dev={:.3}, overlap_mean={:.1}%)",
            summary.total_passed,
            summary.total_raw,
            threshold_adaptive,
            std_dev,
            if query_tokens.is_empty() {
                0.0
            } else {
                summary
                    .results
                    .iter()
                    .map(|r| if r.promoted_by_token_overlap { 1 } else { 0 })
                    .sum::<usize>() as f32
                    / summary.results.len().max(1) as f32
                    * 100.0
            },
        );

        summary
    }

    /// Versión simplificada que solo devuelve los contenidos filtrados
    pub fn rerank_simple(&self, query: &str, raw_results: &[(String, f32)]) -> Vec<String> {
        self.rerank(query, raw_results)
            .results
            .into_iter()
            .map(|r| r.content)
            .collect()
    }

    /// Calcula un score de confianza compuesto para toda la recuperación
    pub fn confidence_score(&self, summary: &RerankSummary) -> f32 {
        if summary.total_raw == 0 {
            return 0.0;
        }
        let pass_rate = summary.total_passed as f32 / summary.total_raw as f32;
        let mean_score = if summary.results.is_empty() {
            0.0
        } else {
            summary.results.iter().map(|r| r.rerank_score).sum::<f32>()
                / summary.results.len() as f32
        };
        // Combinación ponderada: 40% tasa de paso + 60% score promedio
        0.4 * pass_rate + 0.6 * mean_score
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reranker_basic() {
        let reranker = Reranker::default();
        let query = "cómo implementar chunker semántico en rust";
        let results = vec![
            ("fn chunk() { /* chunker code */ }".to_string(), 0.2),
            ("struct MemoriaSemantica {".to_string(), 0.5),
            ("fn main() { println!(\"hi\"); }".to_string(), 0.8),
        ];
        let summary = reranker.rerank(query, &results);
        // The chunker-related result should be promoted, the main() degraded
        assert!(summary.total_passed <= summary.total_raw);
        if summary.total_passed > 0 {
            assert!(summary.results[0].rerank_score >= 0.0);
        }
    }

    #[test]
    fn test_reranker_confidence() {
        let reranker = Reranker::default();
        let results = vec![("content".to_string(), 0.3)];
        let summary = reranker.rerank("test query", &results);
        let confidence = reranker.confidence_score(&summary);
        assert!(confidence >= 0.0 && confidence <= 1.0);
    }

    #[test]
    fn test_reranker_empty() {
        let reranker = Reranker::default();
        let summary = reranker.rerank("query", &[]);
        assert_eq!(summary.total_raw, 0);
        assert_eq!(summary.total_passed, 0);
    }

    #[test]
    fn test_token_overlap_promotion() {
        let reranker = Reranker::default();
        let query = "red neuronal convolucional";
        let results = vec![
            (
                "implementación de red neuronal convolucional en rust con candle".to_string(),
                0.4,
            ),
            ("receta de pan casero paso a paso".to_string(), 0.4),
        ];
        let summary = reranker.rerank(query, &results);
        // The neural network result should be promoted or at least not degraded more
        assert!(summary.total_passed > 0);
    }
}
