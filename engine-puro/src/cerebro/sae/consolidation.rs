// ============================================================================
// 🧠 SAE — CONSOLIDATION (Ciclo Vigilia-Sueño)
// ============================================================================
// Los Transformers no duermen. El SAE sí. Durante la fase de "sueño" el sistema
// consolida lo aprendido en vigilia:
//
//   1. Replay de patrones de memoria (consolidación Hebbiana lenta).
//   2. Structural plasticity: poda de sinapsis débiles + neurogénesis.
//   3. Homeostasis global: rebalanceo de tasas de disparo.
//   4. Generación suprimida (el sistema "descansa").
//
// El ciclo se controla externamente (se decide cuándo dormir), típicamente
// cuando se acumulan N pasos de vigilia o por un temporizador.
// ============================================================================

use crate::cerebro::sae::plasticity::{PlasticityEngine, Synapse};

/// Fase del ciclo de sueño.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SleepPhase {
    Vigilia,
    SuenoProfundo,
    Replay,
    Despertar,
}

/// Resultado de una consolidación.
#[derive(Debug, Clone)]
pub struct ConsolidationReport {
    pub fases_ejecutadas: Vec<SleepPhase>,
    pub sinapsis_podadas: usize,
    pub sinapsis_nuevas: usize,
    pub neuronas_rebalanceadas: usize,
}

/// Motor de consolidación (ciclo vigilia-sueño).
#[derive(Debug, Clone)]
pub struct ConsolidationEngine {
    /// Sinapsis del sistema (se consolidan in-place).
    pub synapses: Vec<Synapse>,
    /// Neuronas del sistema con su tasa de disparo.
    pub neuron_rates: Vec<f32>,
    /// Plástica compartida.
    pub plasticity: PlasticityEngine,
}

impl ConsolidationEngine {
    pub fn new(plasticity: PlasticityEngine) -> Self {
        Self {
            synapses: Vec::new(),
            neuron_rates: Vec::new(),
            plasticity,
        }
    }

    /// Ejecuta una fase de sueño completa: poda + neurogénesis + homeostasis.
    pub fn run_sleep_cycle(&mut self, neuronas: &[u32]) -> ConsolidationReport {
        let mut fases = Vec::new();

        // 1. SuenoProfundo: homeostasis global (rebalanceo).
        fases.push(SleepPhase::SuenoProfundo);
        let mut rebalanceadas = 0;
        for (i, rate) in self.neuron_rates.iter().enumerate() {
            let target = self.plasticity.target_rate;
            if (rate - target).abs() > 0.1 {
                rebalanceadas += 1;
            }
        }

        // 2. Replay: consolidación Hebbiana lenta (reforzar sinapsis activas).
        fases.push(SleepPhase::Replay);
        for s in self.synapses.iter_mut() {
            // Reforzar ligeramente las sinapsis con traza alta (memoria).
            if s.trace > 0.3 {
                s.weight = (s.weight * 1.001).clamp(0.0, self.plasticity.max_weight);
            }
        }

        // 3. Structural plasticity: poda de débiles.
        let antes = self.synapses.len();
        self.synapses = self.plasticity.prune(&self.synapses);
        let podadas = antes - self.synapses.len();

        // 4. Neurogénesis: crear sinapsis entre neuronas co-activas.
        let activaciones: Vec<f32> = self
            .neuron_rates
            .iter()
            .map(|&r| (r / self.plasticity.target_rate).clamp(0.0, 1.0))
            .collect();
        let nuevas = self
            .plasticity
            .neurogenesis(neuronas, &activaciones, 100);
        let n_nuevas = nuevas.len();
        self.synapses.extend(nuevas);

        // 5. Despertar: restaurar tasas hacia el objetivo.
        fases.push(SleepPhase::Despertar);
        for rate in self.neuron_rates.iter_mut() {
            *rate = (*rate + self.plasticity.target_rate) / 2.0;
        }

        ConsolidationReport {
            fases_ejecutadas: fases,
            sinapsis_podadas: podadas,
            sinapsis_nuevas: n_nuevas,
            neuronas_rebalanceadas: rebalanceadas,
        }
    }

    /// Registra tasas de disparo de las neuronas (desde la vigilia).
    pub fn set_neuron_rates(&mut self, rates: Vec<f32>) {
        self.neuron_rates = rates;
    }

    /// Decide si es momento de dormir: cuando la homeostasis está desbalanceada
    /// (muchas neuronas lejos del objetivo) o hay demasiadas sinapsis débiles.
    pub fn should_sleep(&self) -> bool {
        // Muchas neuronas desviadas del objetivo → dormir.
        let desviadas = self
            .neuron_rates
            .iter()
            .filter(|&&r| (r - self.plasticity.target_rate).abs() > 0.3)
            .count();
        if !self.neuron_rates.is_empty() {
            desviadas > self.neuron_rates.len() / 2
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ciclo_de_sueno_poda_y_neurogenesis() {
        let mut pl = PlasticityEngine::default();
        pl.neurogenesis_rate = 1.0;
        let mut eng = ConsolidationEngine::new(pl);
        eng.set_neuron_rates(vec![0.5, 0.6, 0.1]);
        eng.synapses = vec![
            Synapse::new(0, 1, 0.001), // débil → podar
            Synapse::new(1, 2, 0.5),
        ];
        let neuronas = vec![0u32, 1, 2];
        let report = eng.run_sleep_cycle(&neuronas);
        assert_eq!(report.sinapsis_podadas, 1);
        assert!(report.fases_ejecutadas.contains(&SleepPhase::Replay));
        assert!(report.fases_ejecutadas.contains(&SleepPhase::SuenoProfundo));
    }

    #[test]
    fn detecta_necesidad_de_sueno() {
        let mut eng = ConsolidationEngine::new(PlasticityEngine::default());
        // 4 de 7 neuronas claramente desviadas del objetivo (0.3) → dormir.
        eng.set_neuron_rates(vec![0.9, 0.9, 0.9, 0.9, 0.3, 0.3, 0.3]);
        assert!(eng.should_sleep());
    }

    #[test]
    fn no_duerme_si_balanceado() {
        let mut eng = ConsolidationEngine::new(PlasticityEngine::default());
        eng.set_neuron_rates(vec![0.3, 0.3, 0.3]);
        assert!(!eng.should_sleep());
    }
}
