// ============================================================================
// 🧠 SAE — GENERATOR: Generación Guiada por Intención (IGG)
// ============================================================================
// INNOVACIÓN frente al LLM autorregresivo estándar.
//
// Un LLM genera token a token por PREDICCIÓN ESTADÍSTICA: P(w_t | w_<t).
// No tiene "mensaje objetivo" → sufre deriva de coherencia (off-topic drift).
//
// El cerebro humano NO funciona así: la corteza prefrontal mantiene la
// INTENCIÓN COMPLETA del mensaje (M), y el área de Broca la articula palabra
// por palabra, REDUCIÉNDOLA hasta agotarla.
//
// ──────────────────────────────────────────────────────────────────────────
//   M        = vector de intención (mensaje completo, codificado una vez)
//   R₀       = M                              (intención residual inicial)
//   g(w_t)   = preferencia[i] · ⟨R_t, pref_i⟩ (lo que el token expresa)
//   R_{t+1}  = R_t − α · g(w_t)               (consumo de significado)
//   STOP si  ‖R_{t+1}‖ < ε                    (homeostasis: mensaje expresado)
// ──────────────────────────────────────────────────────────────────────────
//
// Esta es una DESCOMPOSICIÓN ORTOGONAL ITERATIVA (estilo Gram-Schmidt) del
// vector de intención: cada palabra extrae la componente del significado que
// mejor representa. Resultado: generación SEMÁNTICA-GLOBAL (reduce un mensaje)
// en lugar de LOCAL-ESTADÍSTICA (predice el siguiente token).
//
// La neuroquímica modula el proceso:
//   - Dopamina alta → α grande (avanza rápido, mensaje directo).
//   - Cortisol alto → ε alto (se detiene antes, túnel cognitivo).
//   - Noradrenalina → ruido en la selección (asociaciones remotas).
// ============================================================================

use crate::cerebro::sae::decoder::TokenUnit;
use crate::cerebro::sistema_limbico::Neuroquimica;

/// Configuración del generador IGG.
#[derive(Debug, Clone)]
pub struct GeneratorConfig {
    /// Coeficiente de consumo de significado por token (0.0..1.0).
    pub alfa: f32,
    /// Umbral de homeostasis: ‖R‖ por debajo de ε → silencio.
    pub epsilon: f32,
    /// Longitud máxima de la secuencia (cortafuegos).
    pub max_tokens: usize,
    /// Ruido de selección (noradrenalina base).
    pub ruido_base: f32,
}

impl Default for GeneratorConfig {
    fn default() -> Self {
        Self {
            alfa: 0.35,
            epsilon: 0.08,
            max_tokens: 24,
            ruido_base: 0.05,
        }
    }
}

/// Un paso de generación.
#[derive(Debug, Clone)]
pub struct GenerationStep {
    pub token: String,
    pub activacion: f32,
    /// Norma del residual ANTES de emitir el token.
    pub residual_norm: f32,
    /// ¿Fue el paso final (homeostasis alcanzada)?
    pub final_step: bool,
}

/// Resultado de una secuencia generada.
#[derive(Debug, Clone)]
pub struct GeneratedSequence {
    pub tokens: Vec<String>,
    pub steps: Vec<GenerationStep>,
    /// Norma del residual final (cuánto del mensaje quedó sin expresar).
    pub residual_final: f32,
    /// ¿Terminó por homeostasis (ε) o por límite de tokens?
    pub terminated_by_homeostasis: bool,
}

/// Codificador de intención: combina estado interno + memoria en M.
#[derive(Debug, Clone)]
pub struct IntentionEncoder {
    /// Peso del estado interno vs memoria en la intención.
    pub peso_estado: f32,
}

impl Default for IntentionEncoder {
    fn default() -> Self {
        Self { peso_estado: 0.5 }
    }
}

impl IntentionEncoder {
    /// Codifica la intención M a partir de:
    ///   - `estado_vector`: estado interno (neuroquímica, tensión, etc.)
    ///   - `memoria`: activaciones de los tokens de memoria (su preferencia).
    ///
    /// M = peso_estado·estado + (1−peso_estado)·Σ activaciones de memoria.
    /// El resultado se normaliza a norma 1 (para que ‖R‖ decaiga hacia ε).
    pub fn encode(
        &self,
        estado_vector: &[f32],
        memoria: &[TokenUnit],
        memoria_act: &[f32],
    ) -> Vec<f32> {
        let dim = estado_vector.len().max(
            memoria
                .iter()
                .map(|u| u.preferencia.len())
                .max()
                .unwrap_or(0),
        );
        let mut m = vec![0.0f32; dim];

        // Componente de estado.
        for i in 0..dim.min(estado_vector.len()) {
            m[i] += self.peso_estado * estado_vector[i];
        }

        // Componente de memoria (suma ponderada de preferencias activas).
        let peso_mem = 1.0 - self.peso_estado;
        for (unit, act) in memoria.iter().zip(memoria_act.iter()) {
            for i in 0..dim.min(unit.preferencia.len()) {
                m[i] += peso_mem * act * unit.preferencia[i];
            }
        }

        normalize(&m)
    }
}

/// Generador guiado por intención.
#[derive(Debug, Clone)]
pub struct Generator {
    pub config: GeneratorConfig,
    pub encoder: IntentionEncoder,
}

impl Generator {
    pub fn new(config: GeneratorConfig) -> Self {
        Self {
            config,
            encoder: IntentionEncoder::default(),
        }
    }

    /// Genera una secuencia reduciendo la intención hasta la homeostasis.
    ///
    /// A diferencia del LLM autorregresivo, el token NO se realimenta como
    /// contexto: lo que guía cada paso es el RESIDUAL R que queda por expresar.
    pub fn generate(
        &self,
        estado_vector: &[f32],
        memoria: &[TokenUnit],
        memoria_act: &[f32],
        nq: &Neuroquimica,
    ) -> GeneratedSequence {
        // 1. Codificar la intención completa una sola vez.
        let mut residual = self.encoder.encode(estado_vector, memoria, memoria_act);
        let mut r_norm = norm(&residual);

        let mut tokens = Vec::new();
        let mut steps = Vec::new();

        // Moduladores neuroquímicos.
        let alfa = (self.config.alfa * (0.6 + nq.dopamina * 0.8)).min(1.0);
        let epsilon = self.config.epsilon * (1.0 + nq.cortisol * 0.5);
        let ruido = self.config.ruido_base + nq.adrenalina * 0.3;

        for _ in 0..self.config.max_tokens {
            // Homeostasis: ¿queda algo por expresar?
            if r_norm < epsilon {
                break;
            }

            // 2. Selección del token: el que mejor proyecta sobre el residual.
            let Some((token, pref, activacion)) =
                self.select_token(&residual, memoria, ruido, nq)
            else {
                break;
            };

            // 3. Consumo de significado: R -= α·g(w), g(w)=pref·⟨R,pref⟩.
            let proyeccion = dot(&residual, &pref);
            let mut nuevo_residual = residual.clone();
            for i in 0..nuevo_residual.len().min(pref.len()) {
                nuevo_residual[i] -= alfa * proyeccion * pref[i];
            }

            tokens.push(token.clone());
            steps.push(GenerationStep {
                token,
                activacion,
                residual_norm: r_norm,
                final_step: false,
            });

            residual = nuevo_residual;
            r_norm = norm(&residual);
        }

        let terminated_by_homeostasis = r_norm < epsilon && !tokens.is_empty();
        if let Some(last) = steps.last_mut() {
            last.final_step = terminated_by_homeostasis;
        }

        GeneratedSequence {
            tokens,
            steps,
            residual_final: r_norm,
            terminated_by_homeostasis,
        }
    }

    /// Selecciona el token con mayor proyección sobre el residual, con ruido.
    fn select_token(
        &self,
        residual: &[f32],
        memoria: &[TokenUnit],
        ruido: f32,
        nq: &Neuroquimica,
    ) -> Option<(String, Vec<f32>, f32)> {
        if memoria.is_empty() {
            return None;
        }

        // Score = proyección normalizada (coseno) + ruido noradrenérgico.
        let mut scored: Vec<(usize, f32)> = memoria
            .iter()
            .enumerate()
            .map(|(i, unit)| {
                let cos = cosine(residual, &unit.preferencia);
                let noise = (rand::random::<f32>() - 0.5) * 2.0 * ruido;
                (i, cos + noise)
            })
            .collect();

        // Competencia: softmax con temperatura por noradrenalina.
        let temp = 0.4 + nq.adrenalina * 0.8;
        let probs = softmax(&scored.iter().map(|(_, s)| *s).collect::<Vec<_>>(), temp);
        let idx = sample_weighted(&scored, &probs);

        let unit = &memoria[idx];
        let activacion = cosine(residual, &unit.preferencia);
        Some((unit.token.clone(), unit.preferencia.clone(), activacion))
    }

    /// Formatea la secuencia como texto (unión de tokens).
    pub fn format(seq: &GeneratedSequence) -> String {
        seq.tokens.join(" ")
    }
}

// ── Utilidades matemáticas ──────────────────────────────────────────────

fn dot(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum::<f32>()
}

fn norm(v: &[f32]) -> f32 {
    v.iter().map(|x| x * x).sum::<f32>().sqrt()
}

fn normalize(v: &[f32]) -> Vec<f32> {
    let n = norm(v);
    if n == 0.0 {
        v.to_vec()
    } else {
        v.iter().map(|x| x / n).collect()
    }
}

fn cosine(a: &[f32], b: &[f32]) -> f32 {
    let na = norm(a);
    let nb = norm(b);
    if na == 0.0 || nb == 0.0 {
        0.0
    } else {
        dot(a, b) / (na * nb)
    }
}

fn softmax(scores: &[f32], temperatura: f32) -> Vec<f32> {
    let max = scores.iter().cloned().fold(f32::MIN, f32::max);
    let mut exp: Vec<f32> = scores
        .iter()
        .map(|s| ((s - max) / temperatura.max(0.05)).exp())
        .collect();
    let sum: f32 = exp.iter().sum();
    if sum > 0.0 {
        for e in exp.iter_mut() {
            *e /= sum;
        }
    }
    exp
}

fn sample_weighted(scored: &[(usize, f32)], probs: &[f32]) -> usize {
    let r: f32 = rand::random();
    let mut acc = 0.0f32;
    for (i, p) in probs.iter().enumerate() {
        acc += p;
        if r <= acc {
            return scored[i].0;
        }
    }
    scored
        .iter()
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
        .map(|(i, _)| *i)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cerebro::sistema_limbico::Neuroquimica;

    fn neuro(dopamina: f32, cortisol: f32, adrenalina: f32) -> Neuroquimica {
        Neuroquimica {
            dopamina,
            cortisol,
            adrenalina,
            ..Default::default()
        }
    }

    fn vocabulario_ortogonal() -> Vec<TokenUnit> {
        // Tokens cuyas preferencias son una base ortonormal (R³).
        vec![
            TokenUnit {
                token: "rojo".into(),
                preferencia: vec![1.0, 0.0, 0.0],
            },
            TokenUnit {
                token: "verde".into(),
                preferencia: vec![0.0, 1.0, 0.0],
            },
            TokenUnit {
                token: "azul".into(),
                preferencia: vec![0.0, 0.0, 1.0],
            },
        ]
    }

    #[test]
    fn intencion_compuesta_se_descompone_en_sus_tokens() {
        // Intención = 0.7·rojo + 0.5·verde (normalizada).
        let estado = vec![0.7, 0.5, 0.0];
        let memoria = vocabulario_ortogonal();
        let gen = Generator::new(GeneratorConfig::default());
        let seq = gen.generate(&estado, &memoria, &[1.0; 3], &neuro(0.5, 0.0, 0.0));
        assert!(!seq.tokens.is_empty());
        // Debe contener rojo y verde (los componentes de la intención).
        assert!(seq.tokens.contains(&"rojo".to_string()));
        assert!(seq.tokens.contains(&"verde".to_string()));
    }

    #[test]
    fn termina_por_homeostasis_con_intencion_alineada() {
        // Intención alineada con un único token (rojo). El residual decae
        // geométricamente con factor (1−α) por paso, alcanzando ε.
        let mut cfg = GeneratorConfig::default();
        cfg.epsilon = 0.01;
        cfg.max_tokens = 50; // margen para que decaiga
        let gen = Generator::new(cfg);
        let estado = vec![1.0, 0.0, 0.0];
        let memoria = vocabulario_ortogonal();
        // Solo rojo activo en memoria (refuerza la intención hacia rojo).
        let seq = gen.generate(&estado, &memoria, &[1.0, 0.0, 0.0], &neuro(0.5, 0.0, 0.0));
        assert!(seq.terminated_by_homeostasis);
        assert!(seq.residual_final < 0.1);
    }

    #[test]
    fn cortafuegos_max_tokens() {
        let mut cfg = GeneratorConfig::default();
        cfg.epsilon = 0.0; // nunca por homeostasis
        cfg.max_tokens = 5;
        cfg.alfa = 0.01; // consume poco → no alcanza ε
        let gen = Generator::new(cfg);
        let estado = vec![1.0, 1.0, 1.0];
        let memoria = vocabulario_ortogonal();
        let seq = gen.generate(&estado, &memoria, &[1.0; 3], &neuro(0.0, 0.0, 0.0));
        assert!(!seq.terminated_by_homeostasis);
        assert!(seq.tokens.len() <= 5);
    }

    #[test]
    fn cortisol_detiene_antes() {
        let mut cfg = GeneratorConfig::default();
        cfg.epsilon = 0.01;
        let gen = Generator::new(cfg);
        let estado = vec![1.0, 1.0, 1.0];
        let memoria = vocabulario_ortogonal();

        let calmado = gen.generate(&estado, &memoria, &[1.0; 3], &neuro(0.5, 0.0, 0.0));
        let estresado = gen.generate(&estado, &memoria, &[1.0; 3], &neuro(0.5, 0.9, 0.0));
        // Cortisol alto → ε mayor → menos tokens.
        assert!(estresado.tokens.len() <= calmado.tokens.len());
    }

    #[test]
    fn formato_une_tokens() {
        let seq = GeneratedSequence {
            tokens: vec!["rojo".into(), "verde".into()],
            steps: vec![],
            residual_final: 0.0,
            terminated_by_homeostasis: true,
        };
        assert_eq!(Generator::format(&seq), "rojo verde");
    }
}
