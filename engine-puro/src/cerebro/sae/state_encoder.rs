// ============================================================================
// 🧠 SAE — STATE ENCODER (Estado Interno → Trenes de Spikes)
// ============================================================================
// Análogo al tálamo: convierte el estado continuo del sistema (neuroquímica,
// asambleas, monólogo) en trenes de spikes que alimentan la atención.
//
// Codificación:
//   - Rate coding: tasa de disparo ∝ intensidad del estado
//   - Poisson sampling: los spikes no son deterministas (ruido biológico)
// ============================================================================

use rand::{Rng, SeedableRng};

/// Un tren de spikes: vector de 0/1 a lo largo de pasos de tiempo.
pub type SpikeTrain = Vec<u8>;

/// Dimensión de estado del sistema que codifica el encoder.
#[derive(Debug, Clone, Default)]
pub struct EstadoEntrada {
    /// Vector de estado normalizado [0..1] por dimensión.
    pub vector: Vec<f32>,
    /// Etiquetas descriptivas por dimensión (para decodificación/debug).
    pub etiquetas: Vec<String>,
}

/// Codifica estado continuo → trenes de spikes (rate + Poisson).
#[derive(Debug, Clone)]
pub struct StateEncoder {
    /// Ventana temporal (número de pasos por tren).
    pub ventana_t: usize,
    /// Tasa máxima de disparo (spikes por paso).
    pub tasa_max: f32,
    /// Semilla para reproducibilidad (None = aleatoria).
    pub seed: Option<u64>,
}

impl Default for StateEncoder {
    fn default() -> Self {
        Self {
            ventana_t: 16,
            tasa_max: 0.8,
            seed: None,
        }
    }
}

impl StateEncoder {
    pub fn new(ventana_t: usize, tasa_max: f32) -> Self {
        Self {
            ventana_t,
            tasa_max,
            seed: None,
        }
    }

    /// Codifica el estado completo en un tren de spikes por dimensión.
    ///
    /// Cada dimensión del estado produce un tren binario de longitud
    /// `ventana_t`. La densidad de 1s es proporcional al valor de la dimensión.
    pub fn encode(&self, estado: &EstadoEntrada) -> Vec<SpikeTrain> {
        let mut rng: rand::rngs::StdRng = match self.seed {
            Some(s) => rand::rngs::StdRng::seed_from_u64(s),
            None => rand::rngs::StdRng::from_entropy(),
        };

        estado
            .vector
            .iter()
            .map(|&v| self.encode_one(v, &mut rng))
            .collect()
    }

    /// Codifica un solo valor [0..1] en un tren binario.
    fn encode_one(&self, valor: f32, rng: &mut impl rand::Rng) -> SpikeTrain {
        let v = valor.clamp(0.0, 1.0);
        let tasa = v * self.tasa_max;
        (0..self.ventana_t)
            .map(|_| if rng.gen::<f32>() < tasa { 1 } else { 0 })
            .collect()
    }

    /// Decodifica un tren de spikes a su "tasa media" (0..1).
    /// Útil para verificar round-trip y para normalización.
    pub fn decode_rate(tren: &SpikeTrain) -> f32 {
        if tren.is_empty() {
            return 0.0;
        }
        let n = tren.iter().filter(|&&s| s == 1).count();
        n as f32 / tren.len() as f32
    }

    /// Dependencia media (correlación) entre dos trenes: qué fracción de
    /// pasos tienen ambos spikes activos. Esto es el "spike coincidence" que
    /// alimenta la atención temporal.
    pub fn spike_coincidence(a: &SpikeTrain, b: &SpikeTrain) -> f32 {
        if a.len() != b.len() || a.is_empty() {
            return 0.0;
        }
        let coincidentes = a
            .iter()
            .zip(b.iter())
            .filter(|(&x, &y)| x == 1 && y == 1)
            .count();
        coincidentes as f32 / a.len() as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn estado_alto_genera_mas_spikes() {
        let enc = StateEncoder::new(200, 0.9);
        let alto = enc.encode_one(0.9, &mut rand::thread_rng());
        let bajo = enc.encode_one(0.1, &mut rand::thread_rng());
        let tasa_alta = StateEncoder::decode_rate(&alto);
        let tasa_baja = StateEncoder::decode_rate(&bajo);
        assert!(tasa_alta > tasa_baja);
    }

    #[test]
    fn estado_cero_no_dispara() {
        let enc = StateEncoder::new(100, 0.9);
        let tren = enc.encode_one(0.0, &mut rand::thread_rng());
        assert_eq!(StateEncoder::decode_rate(&tren), 0.0);
    }

    #[test]
    fn coincidencia_todos_spikes_es_1() {
        // Ambos trenes con spike en todos los pasos → coincidencia 1.0.
        let a = vec![1u8, 1, 1, 1, 1];
        let b = vec![1u8, 1, 1, 1, 1];
        assert_eq!(StateEncoder::spike_coincidence(&a, &b), 1.0);
    }

    #[test]
    fn coincidencia_ortogonal_es_0() {
        let a = vec![1u8, 1, 1, 0, 0, 0];
        let b = vec![0u8, 0, 0, 1, 1, 1];
        assert_eq!(StateEncoder::spike_coincidence(&a, &b), 0.0);
    }

    #[test]
    fn encode_dimensiones_por_estado() {
        let enc = StateEncoder::default();
        let estado = EstadoEntrada {
            vector: vec![0.5, 0.8, 0.2],
            etiquetas: vec!["dopamina".into(), "activacion".into(), "calma".into()],
        };
        let trenes = enc.encode(&estado);
        assert_eq!(trenes.len(), 3);
        assert_eq!(trenes[0].len(), enc.ventana_t);
    }
}
