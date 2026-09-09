// ============================================================================
// 🧠 SAE — DECODER (Área de Broca Digital: Spikes → Tokens)
// ============================================================================
// Mapea los patrones de activación de salida del bloque de atención a tokens
// de texto. No usa beam search ni sampling de Transformer: selección por
// competencia neuronal — cada token tiene un "vector preferido" y la palabra
// elegida es la más cercana en el espacio de asociación.
//
// La temperatura de muestreo es modulada por noradrenalina (exploración léxica).
// ============================================================================

use crate::cerebro::sistema_limbico::Neuroquimica;

/// Configuración del decodificador.
#[derive(Debug, Clone)]
pub struct DecoderConfig {
    /// Umbral mínimo de activación para emitir un token.
    pub umbral: f32,
    /// Temperatura de muestreo base.
    pub temperatura_base: f32,
}

impl Default for DecoderConfig {
    fn default() -> Self {
        Self {
            umbral: 0.3,
            temperatura_base: 0.8,
        }
    }
}

/// Un token candidato con su "vector preferido" (aprendido).
#[derive(Debug, Clone)]
pub struct TokenUnit {
    pub token: String,
    /// Vector de activación preferido (mismo largo que el output de atención).
    pub preferencia: Vec<f32>,
}

/// Elección decodificada.
#[derive(Debug, Clone)]
pub struct TokenChoice {
    pub token: String,
    pub activacion: f32,
    pub exploracion: bool,
}

/// Decodificador de spikes → tokens.
#[derive(Debug, Clone)]
pub struct Decoder {
    pub config: DecoderConfig,
    /// Vocabulario de tokens disponibles.
    pub vocabulario: Vec<TokenUnit>,
}

impl Decoder {
    pub fn new(config: DecoderConfig, vocabulario: Vec<TokenUnit>) -> Self {
        Self {
            config,
            vocabulario,
        }
    }

    /// Decodifica la activación de salida en un token.
    ///
    /// - Calcula la similitud coseno entre el output de atención y cada
    ///   token del vocabulario.
    /// - Aplica temperatura (modulada por noradrenalina) para exploración.
    /// - Si la mejor activación supera el umbral, emite el token; si no,
    ///   devuelve silencio (el sistema "decide no hablar").
    pub fn decode(&self, activacion_salida: &[f32], nq: &Neuroquimica) -> Option<TokenChoice> {
        if self.vocabulario.is_empty() {
            return None;
        }

        // Temperatura modulada por noradrenalina (alta NA → más exploración).
        let temperatura = self.config.temperatura_base * (0.5 + nq.adrenalina);

        // Similitud de cada token con la activación de salida.
        let mut scores: Vec<(usize, f32)> = self
            .vocabulario
            .iter()
            .enumerate()
            .map(|(i, unit)| {
                let s = cosine(&activacion_salida, &unit.preferencia);
                (i, s)
            })
            .collect();

        // Decidir si hablar: el mejor score debe superar el umbral (la
        // decisión de "expresarse" se basa en la activación disponible,
        // no en el token muestreado — evita silencio espurio con
        // temperatura alta / exploración).
        let mejor_score = scores.iter().map(|(_, s)| *s).fold(f32::MIN, f32::max);
        if mejor_score < self.config.umbral {
            return None; // silencio: nada supera el umbral
        }

        // Softmax con temperatura (probabilidades de selección).
        let probs = softmax_temperature(&scores, temperatura);

        // Selección por competencia neuronal (muestreo ponderado).
        let (idx, activacion) = sample_weighted(&scores, &probs);

        Some(TokenChoice {
            token: self.vocabulario[idx].token.clone(),
            activacion,
            exploracion: temperatura > 1.0,
        })
    }
}

/// Similitud coseno entre dos vectores.
fn cosine(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let mut dot = 0.0f32;
    let mut na = 0.0f32;
    let mut nb = 0.0f32;
    for i in 0..a.len() {
        dot += a[i] * b[i];
        na += a[i] * a[i];
        nb += b[i] * b[i];
    }
    if na == 0.0 || nb == 0.0 {
        0.0
    } else {
        dot / (na.sqrt() * nb.sqrt())
    }
}

/// Softmax con temperatura sobre las similitudes.
fn softmax_temperature(scores: &[(usize, f32)], temperatura: f32) -> Vec<f32> {
    let max = scores
        .iter()
        .map(|(_, s)| *s)
        .fold(f32::MIN, f32::max);
    let mut exp: Vec<f32> = scores
        .iter()
        .map(|(_, s)| ((s - max) / temperatura.max(0.05)).exp())
        .collect();
    let sum: f32 = exp.iter().sum();
    if sum > 0.0 {
        for e in exp.iter_mut() {
            *e /= sum;
        }
    }
    exp
}

/// Muestreo ponderado por probabilidades (competencia neuronal).
fn sample_weighted(scores: &[(usize, f32)], probs: &[f32]) -> (usize, f32) {
    let r: f32 = rand::random();
    let mut acc = 0.0f32;
    for (i, p) in probs.iter().enumerate() {
        acc += p;
        if r <= acc {
            return (scores[i].0, scores[i].1);
        }
    }
    // Fallback: el de mayor similitud.
    let (idx_max, score_max) = scores.iter().max_by(|a, b| a.1.partial_cmp(&b.1).unwrap()).unwrap();
    (*idx_max, *score_max)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn neuro(adrenalina: f32) -> Neuroquimica {
        Neuroquimica {
            adrenalina,
            ..Default::default()
        }
    }

    fn vocabulario() -> Vec<TokenUnit> {
        vec![
            TokenUnit {
                token: "gato".into(),
                preferencia: vec![1.0, 0.0, 0.0],
            },
            TokenUnit {
                token: "perro".into(),
                preferencia: vec![0.0, 1.0, 0.0],
            },
            TokenUnit {
                token: "pez".into(),
                preferencia: vec![0.0, 0.0, 1.0],
            },
        ]
    }

    #[test]
    fn decodifica_token_mas_similar() {
        let dec = Decoder::new(DecoderConfig::default(), vocabulario());
        // Activación alineada con "gato". El muestreo es estocástico, así que
        // se verifica que "gato" es el elegido con mayor frecuencia.
        let act = vec![0.9, 0.1, 0.0];
        let mut conteos = std::collections::HashMap::new();
        for _ in 0..200 {
            if let Some(c) = dec.decode(&act, &neuro(0.2)) {
                *conteos.entry(c.token).or_insert(0) += 1;
            }
        }
        let ganador = conteos
            .iter()
            .max_by(|a, b| a.1.cmp(b.1))
            .map(|(k, _)| k.clone())
            .unwrap();
        assert_eq!(ganador, "gato");
    }

    #[test]
    fn silencio_si_baja_activacion() {
        let mut cfg = DecoderConfig::default();
        cfg.umbral = 0.9;
        let dec = Decoder::new(cfg, vocabulario());
        // Activación ambigua y baja.
        let act = vec![0.3, 0.3, 0.3];
        assert!(dec.decode(&act, &neuro(0.2)).is_none());
    }

    #[test]
    fn noradrenalina_alta_marca_exploracion() {
        let dec = Decoder::new(DecoderConfig::default(), vocabulario());
        let act = vec![1.0, 0.0, 0.0];
        let choice = dec.decode(&act, &neuro(1.0)).unwrap();
        // Con NA alta, temperatura > 1 → exploración true.
        assert!(choice.exploracion);
    }
}
