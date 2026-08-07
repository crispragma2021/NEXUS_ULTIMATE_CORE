// ============================================================================
// 🧠 SAE — PLASTICITY ENGINE (STDP + Homeostasis + Structural Plasticity)
// ============================================================================
// Permite que el SAE aprenda en tiempo real (a diferencia de un Transformer
// con pesos congelados):
//
//   1. STDP (Spike-Timing-Dependent Plasticity) — ya existe en engine-puro:
//      Δw = A₊·e^{-Δt/τ₊} si post>pre (potenciación)
//      Δw = -A₋·e^{Δt/τ₋} si post<pre (depresión)
//   2. Homeostasis — cada neurona regula su tasa de disparo hacia un objetivo.
//   3. Structural Plasticity — poda de sinapsis débiles y creación de nuevas.
// ============================================================================

use rand::Rng;

/// Una sinapsis entre dos neuronas.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Synapse {
    pub pre: u32,
    pub post: u32,
    pub weight: f32,
    /// Traza de actividad (para homeostasis).
    pub trace: f32,
}

impl Synapse {
    pub fn new(pre: u32, post: u32, weight: f32) -> Self {
        Self {
            pre,
            post,
            weight,
            trace: 0.0,
        }
    }
}

/// Motor de plasticidad del SAE.
#[derive(Debug, Clone)]
pub struct PlasticityEngine {
    /// A+ (potenciación) y A- (depresión) de STDP.
    pub a_plus: f32,
    pub a_minus: f32,
    /// Constantes de tiempo (pasos).
    pub tau_plus: f32,
    pub tau_minus: f32,
    /// Tasa de disparo objetivo (homeostasis).
    pub target_rate: f32,
    /// Coeficiente de homeostasis.
    pub alpha_homeo: f32,
    /// Umbral de peso para poda (structural plasticity).
    pub prune_threshold: f32,
    /// Probabilidad de crear sinapsis nueva.
    pub neurogenesis_rate: f32,
    /// Límite de peso.
    pub max_weight: f32,
}

impl Default for PlasticityEngine {
    fn default() -> Self {
        Self {
            a_plus: 0.01,
            a_minus: 0.012,
            tau_plus: 20.0,
            tau_minus: 20.0,
            target_rate: 0.3,
            alpha_homeo: 0.01,
            prune_threshold: 0.005,
            neurogenesis_rate: 0.001,
            max_weight: 2.0,
        }
    }
}

impl PlasticityEngine {
    /// Aplica actualización STDP a una sinapsis dado el timing de spikes.
    ///
    /// - `dt`: diferencia temporal t_post - t_pre.
    ///   - dt > 0: post disparó después de pre → potenciación.
    ///   - dt < 0: post disparó antes de pre → depresión.
    pub fn stdp_update(&self, synapse: &mut Synapse, dt: f32) {
        if dt > 0.0 {
            let delta = self.a_plus * (-dt / self.tau_plus).exp();
            synapse.weight = (synapse.weight + delta).clamp(0.0, self.max_weight);
        } else {
            let delta = self.a_minus * (dt / self.tau_minus).exp();
            synapse.weight = (synapse.weight - delta).clamp(0.0, self.max_weight);
        }
    }

    /// Homeostasis: ajusta el peso hacia el objetivo de tasa de disparo.
    ///
    /// Si la neurona post dispara demasiado → reduce pesos de entrada.
    /// Si dispara poco → los aumenta.
    pub fn homeostasis(&self, synapse: &mut Synapse, tasa_post: f32) {
        let error = self.target_rate - tasa_post;
        synapse.weight *= 1.0 + self.alpha_homeo * error;
        synapse.weight = synapse.weight.clamp(0.0, self.max_weight);
        synapse.trace = tasa_post;
    }

    /// Podas sinapsis débiles (olvido). Devuelve las que sobreviven.
    pub fn prune(&self, synapses: &[Synapse]) -> Vec<Synapse> {
        synapses
            .iter()
            .copied()
            .filter(|s| s.weight > self.prune_threshold)
            .collect()
    }

    /// Crea sinapsis nuevas entre pares de neuronas co-activas (neurogénesis
    /// sináptica) durante la consolidación.
    pub fn neurogenesis(
        &self,
        neuronas: &[u32],
        activaciones: &[f32],
        max_synapses: usize,
    ) -> Vec<Synapse> {
        let mut rng = rand::thread_rng();
        let mut nuevas = Vec::new();
        for i in 0..neuronas.len() {
            for j in (i + 1)..neuronas.len() {
                // Dos neuronas co-activas y ambas con activación alta → conectar.
                if activaciones[i] > 0.5 && activaciones[j] > 0.5 {
                    if rng.gen::<f32>() < self.neurogenesis_rate && nuevas.len() < max_synapses {
                        let peso_inicial = 0.01 + rng.gen::<f32>() * 0.05;
                        nuevas.push(Synapse::new(neuronas[i], neuronas[j], peso_inicial));
                    }
                }
            }
        }
        nuevas
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stdp_potencia_si_post_tras_pre() {
        let pl = PlasticityEngine::default();
        let mut s = Synapse::new(0, 1, 0.5);
        let antes = s.weight;
        pl.stdp_update(&mut s, 5.0); // dt > 0 → potenciación
        assert!(s.weight > antes);
    }

    #[test]
    fn stdp_deprime_si_post_antes_pre() {
        let pl = PlasticityEngine::default();
        let mut s = Synapse::new(0, 1, 0.5);
        let antes = s.weight;
        pl.stdp_update(&mut s, -5.0); // dt < 0 → depresión
        assert!(s.weight < antes);
    }

    #[test]
    fn homeostasis_frena_neurona_hiperactiva() {
        let pl = PlasticityEngine::default();
        let mut s = Synapse::new(0, 1, 1.0);
        pl.homeostasis(&mut s, 0.9); // tasa muy alta → reduce peso
        assert!(s.weight < 1.0);
    }

    #[test]
    fn poda_elimina_sinapsis_debiles() {
        let pl = PlasticityEngine::default();
        let syn = vec![
            Synapse::new(0, 1, 0.001), // débil → poda
            Synapse::new(1, 2, 0.5),   // fuerte → sobrevive
        ];
        let sobreviven = pl.prune(&syn);
        assert_eq!(sobreviven.len(), 1);
        assert_eq!(sobreviven[0].post, 2);
    }

    #[test]
    fn neurogenesis_conecta_coactivas() {
        let mut pl = PlasticityEngine::default();
        pl.neurogenesis_rate = 1.0; // siempre crea
        let neuronas = vec![0u32, 1, 2];
        let activaciones = vec![0.9, 0.8, 0.1]; // 0 y 1 co-activas
        let nuevas = pl.neurogenesis(&neuronas, &activaciones, 10);
        assert!(!nuevas.is_empty());
        // Solo conecta neuronas co-activas (0-1), no las que incluyen a 2.
        assert!(nuevas.iter().all(|s| s.pre <= 1 && s.post <= 1));
    }
}
