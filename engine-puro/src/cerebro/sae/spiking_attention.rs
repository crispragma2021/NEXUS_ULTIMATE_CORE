// ============================================================================
// 🧠 SAE — SPIKING ATTENTION BLOCK (Atención Biológica con Spikes)
// ============================================================================
// El corazón del SAE. Implementa la atención como en un Transformer
// (Q·K^T → normalización → V) pero con sustrato biológico:
//
//   sim(q,k) = ∫_{t-τ}^{t} s_q(u)·s_k(u)·e^{-(t-u)/τ} du   (convolución temporal)
//
// Diferencias frente a Transformer estándar:
//   1. Q·K es CONVOLUCIÓN TEMPORAL de spikes, no producto punto instantáneo.
//   2. La softmax se sustituye por COMPETENCIA LATERAL (winner-take-most).
//   3. La atención se MODULA por neuroquímica (dopamina/cortisol/adrenalina).
// ============================================================================

use crate::cerebro::sae::state_encoder::SpikeTrain;
use crate::cerebro::sistema_limbico::Neuroquimica;

/// Configuración del bloque de atención.
#[derive(Debug, Clone)]
pub struct AttentionConfig {
    /// Constante de tiempo de membrana (τ en pasos de tiempo).
    pub tau: f32,
    /// Número de cabezas de atención.
    pub num_heads: usize,
    /// Factor de modulación dopaminérgica.
    pub mu_da: f32,
    /// Ruido noradrenérgico (0.0 = determinista, 1.0 = caótico).
    pub eta: f32,
    /// Umbral de competencia lateral.
    pub competencia: f32,
}

impl Default for AttentionConfig {
    fn default() -> Self {
        Self {
            tau: 3.0,
            num_heads: 4,
            mu_da: 1.0,
            eta: 0.05,
            competencia: 0.2,
        }
    }
}

/// Resultado de una pasada de atención.
#[derive(Debug, Clone)]
pub struct AttentionOutput {
    /// Vector de activación de salida por neurona de valor (suma ponderada).
    pub output: Vec<f32>,
    /// Distribución de atención por par (query_idx, key_idx) — para análisis.
    pub attention_matrix: Vec<Vec<f32>>,
    /// Cabezas activas tras la modulación de cortisol (debug).
    pub active_heads: usize,
}

/// Bloque de atención spiking.
#[derive(Debug, Clone)]
pub struct SpikingAttention {
    pub config: AttentionConfig,
}

impl SpikingAttention {
    pub fn new(config: AttentionConfig) -> Self {
        Self { config }
    }
}

impl Default for SpikingAttention {
    fn default() -> Self {
        Self::new(AttentionConfig::default())
    }
}

impl SpikingAttention {

    /// Ejecuta atención sobre trenes de spikes de Query y Key.
    ///
    /// - `q_trains`: trenes de spikes de las neuronas Query.
    /// - `k_trains`: trenes de spikes de las neuronas Key.
    /// - `values`:   valores continuos asociados a cada Key (V).
    /// - `nq`:       neuroquímica para modular la atención.
    pub fn forward(
        &self,
        q_trains: &[SpikeTrain],
        k_trains: &[SpikeTrain],
        values: &[f32],
        nq: &Neuroquimica,
    ) -> AttentionOutput {
        let n = k_trains.len().min(values.len());
        if n == 0 || q_trains.is_empty() {
            return AttentionOutput {
                output: vec![],
                attention_matrix: vec![],
                active_heads: 0,
            };
        }

        // 1. Matriz de similitud temporal Q·K (convolución de spikes).
        let mut sim = vec![vec![0.0f32; n]; q_trains.len()];
        for (qi, qt) in q_trains.iter().enumerate() {
            for (ki, kt) in k_trains.iter().enumerate().take(n) {
                sim[qi][ki] = self.temporal_similarity(qt, kt);
            }
        }

        // 2. Neuromodulación dopaminérgica: escala la "ganancia" de atención.
        let da_mod = 0.5 + nq.dopamina * self.config.mu_da;

        // 3. Modulación de cortisol: reduce cabezas activas (túnel cognitivo).
        let heads = self.config.num_heads.max(1);
        let active_heads = if nq.cortisol > 0.6 {
            (heads / 2).max(1)
        } else {
            heads
        };

        // 4. Distribución de atención por competencia lateral + neuromodulación.
        let mut attention = vec![vec![0.0f32; n]; q_trains.len()];
        for qi in 0..q_trains.len() {
            // Escalar por dopamina y añadir ruido noradrenérgico.
            let mut scores: Vec<f32> = (0..n)
                .map(|ki| {
                    let base = sim[qi][ki] * da_mod;
                    let noise = if self.config.eta > 0.0 {
                        let r: f32 = rand::random();
                        (r - 0.5) * 2.0 * self.config.eta
                    } else {
                        0.0
                    };
                    base + noise
                })
                .collect();

            // Hiperfoco dopaminérgico: DA alta amplifica el contraste
            // (score^γ con γ ∝ dopamina) → el foco se exagera.
            let ganancia = 0.5 + nq.dopamina; // 0.5..1.5
            for s in scores.iter_mut() {
                let v = (*s).max(0.0);
                *s = v.powf(ganancia);
            }

            // Competencia lateral: winner-take-most con suavizado.
            self.lateral_competition(&mut scores);

            // Escalar distribución para que sume ~1 (distribución de atención).
            let total: f32 = scores.iter().sum();
            if total > 0.0 {
                for s in scores.iter_mut() {
                    *s /= total;
                }
            }
            attention[qi] = scores;
        }

        // 5. Output: suma ponderada de valores por la atención (como V).
        let mut output = vec![0.0f32; q_trains.len()];
        for qi in 0..q_trains.len() {
            let mut acc = 0.0f32;
            for ki in 0..n {
                acc += attention[qi][ki] * values[ki];
            }
            output[qi] = acc;
        }

        AttentionOutput {
            output,
            attention_matrix: attention,
            active_heads,
        }
    }

    /// Convolución temporal de dos trenes de spikes con decaimiento exponencial.
    fn temporal_similarity(&self, a: &SpikeTrain, b: &SpikeTrain) -> f32 {
        let len = a.len().min(b.len());
        if len == 0 {
            return 0.0;
        }
        let mut acc = 0.0f32;
        for t in 0..len {
            if a[t] == 1 && b[t] == 1 {
                // Coincidencia instantánea; se pondera por decaimiento temporal
                // (los spikes recientes importan más).
                let decay = (-((len - t) as f32) / self.config.tau).exp();
                acc += decay;
            }
        }
        acc / len as f32
    }

    /// Competencia lateral: refuerza los máximos y suprime los débiles.
    fn lateral_competition(&self, scores: &mut [f32]) {
        if scores.is_empty() {
            return;
        }
        let max = scores.iter().cloned().fold(f32::MIN, f32::max);
        let thresh = max * self.config.competencia;
        for s in scores.iter_mut() {
            if *s < thresh {
                *s *= 0.1; // supresión lateral
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn neuro(dopamina: f32, cortisol: f32) -> Neuroquimica {
        Neuroquimica {
            dopamina,
            cortisol,
            ..Default::default()
        }
    }

    #[test]
    fn atiende_mas_a_la_key_correlacionada() {
        let att = SpikingAttention::default();
        // Query correlacionada con key[0] (mismos spikes).
        let q = vec![1u8, 1, 0, 1, 1, 0, 1, 1];
        let k0 = vec![1u8, 1, 0, 1, 1, 0, 1, 1]; // igual a q
        let k1 = vec![0u8, 0, 1, 0, 0, 1, 0, 0]; // opuesta a q
        let q_trains = vec![q];
        let k_trains = vec![k0, k1];
        let values = vec![10.0, 1.0];
        let out = att.forward(&q_trains, &k_trains, &values, &neuro(0.5, 0.1));
        // La key[0] debe recibir más atención.
        assert!(out.attention_matrix[0][0] > out.attention_matrix[0][1]);
        // El output debe estar cerca del valor de la key dominante.
        assert!(out.output[0] > 5.0);
    }

    #[test]
    fn dopamina_alta_enfoca_y_baja_dispersa() {
        let att = SpikingAttention::default();
        let q = vec![1u8; 8];
        let k0 = vec![1u8; 8];
        let k1 = vec![0u8, 1, 0, 1, 0, 1, 0, 1];
        let q_trains = vec![q];
        let k_trains = vec![k0.clone(), k1];
        let values = vec![10.0, 10.0];

        let alta_da = att.forward(&q_trains, &k_trains, &values, &neuro(1.0, 0.0));
        let baja_da = att.forward(&q_trains, &k_trains, &values, &neuro(0.0, 0.0));

        // Con dopamina alta el foco en k0 es más fuerte.
        let foco_alta = alta_da.attention_matrix[0][0] / alta_da.attention_matrix[0][1];
        let foco_baja = baja_da.attention_matrix[0][0] / baja_da.attention_matrix[0][1];
        assert!(foco_alta > foco_baja);
    }

    #[test]
    fn cortisol_reduce_cabezas_activas() {
        let att = SpikingAttention::default();
        let q_trains = vec![vec![1u8; 8]; 2];
        let k_trains = vec![vec![1u8; 8]; 2];
        let values = vec![1.0, 2.0];
        let estres = att.forward(&q_trains, &k_trains, &values, &neuro(0.5, 0.9));
        assert!(estres.active_heads < att.config.num_heads);
    }
}
