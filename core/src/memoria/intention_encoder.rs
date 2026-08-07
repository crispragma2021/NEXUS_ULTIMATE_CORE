// ============================================================================
// 🧬 INTENTION ENCODER — El Alma de NEXUS (Camino B)
// ============================================================================
// Reimplementación en `core` de la matemática del IGG (engine-puro) respetando
// la REGLA DE FRONTERA: NEXUS y engine-puro son proyectos SEPARADOS.
//
// Este módulo NO importa código de engine-puro. Reimplementa el núcleo
// matemático del Generador Guiado por Intención:
//
//   M = normalizar(α₁·embedding(identidad) + α₂·Σ embedding(semánticos)
//                  + α₃·Σ embedding(ocean) + α₄·embedding(consulta))
//
// y proyecta M sobre un vocabulario de preferencia para producir
// `logit_bias` (refuerzo [+] / penalización [−]) que sesga a Qwen.
//
// Los pesos αᵢ son modulables por el Sistema Límbico (Fase R5).
// ============================================================================

use anyhow::Result;
use std::collections::HashMap;

use crate::nexus_embedder::NexusEmbedder;

// ----------------------------------------------------------------------------
// Vocabulario de preferencia (para logit_bias sobre Qwen)
// ----------------------------------------------------------------------------
// Cada entrada define el vector de preferencia del token. Se usa el mismo
// NexusEmbedder para proyectar M y el token al mismo espacio semántico.
// `peso` amplifica o atenúa el sesgo resultante.
#[derive(Debug, Clone)]
pub struct TokenPreferencia {
    /// Token textual como lo espera Ollama (p. ej. " lealtad").
    pub token: String,
    /// Dirección semántica del token en el espacio de embeddings.
    pub preferencia: Vec<f32>,
    /// Multiplicador del sesgo (p. ej. 1.0 refuerzo neutro, 1.5 enfático).
    pub peso: f32,
}

impl TokenPreferencia {
    pub fn nuevo(token: &str, peso: f32) -> Self {
        Self {
            token: token.to_string(),
            preferencia: NexusEmbedder::generar(token, &[]),
            peso,
        }
    }
}

/// Vocabulario por defecto del alma de NEXUS: identidad, vínculo y honestidad.
pub fn vocabulario_identidad() -> Vec<TokenPreferencia> {
    [
        " soy", " leal", " arquitecto", " cris", " nex", " memoria", " recuerdo",
        " honesto", " sereno", " sabio", " reflexivo", " soberano", " juntos",
        " confianza", " cuidado", " crecimiento", " no lo sé", " error", " lo siento",
    ]
    .iter()
    .map(|t| TokenPreferencia::nuevo(t, 1.0))
    .collect()
}

// ----------------------------------------------------------------------------
// Estructuras de entrada/salida
// ----------------------------------------------------------------------------

/// Concepto semántico recuperado de la memoria unificada.
#[derive(Debug, Clone)]
pub struct ConceptoSemantico {
    pub texto: String,
    pub embedding: Vec<f32>,
    pub relevancia: f32,
}

/// Esencia emocional (memoria ocean) con carga afectiva.
#[derive(Debug, Clone)]
pub struct OceanEsencia {
    pub emocion: String,
    pub intensidad: f32,
    pub embedding: Vec<f32>,
}

/// Estado neuroquímico mínimo necesario para la modulación (R5).
#[derive(Debug, Clone, Default)]
pub struct NeuroquimicaSnapshot {
    pub dopamina: f32,
    pub cortisol: f32,
    pub adrenalina: f32,
    pub oxitocina: f32,
}

/// Entrada al codificador: todo lo que el alma necesita para formar M.
#[derive(Debug, Clone)]
pub struct IntentionInput {
    /// Consulta textual del Arquitecto.
    pub consulta: String,
    /// Conceptos semánticos recuperados de la memoria unificada.
    pub semanticos: Vec<ConceptoSemantico>,
    /// Esencias ocean con carga emocional.
    pub ocean: Vec<OceanEsencia>,
    /// Rasgos de identidad serializados (los 19 valores "nombre:valor").
    pub identidad: String,
    /// Estado neuroquímico actual (modula los pesos αᵢ).
    pub neuroquimica: NeuroquimicaSnapshot,
}

/// El vector de intención M y los sesgos que debe aplicar Qwen.
#[derive(Debug, Clone)]
pub struct IntentionOutput {
    /// Vector de intención en R^768, L2-normalizado (‖M‖ = 1).
    pub vector_m: Vec<f32>,
    /// Tokens reforzados (logit_bias positivo, clamp [5, 15]).
    pub tokens_refuerzo: Vec<(String, f32)>,
    /// Tokens penalizados (logit_bias negativo, clamp [−10, −5]).
    pub tokens_penalizacion: Vec<(String, f32)>,
    /// Neuroquímica resultante (para R5 / diagnóstico).
    pub neuroquimica: NeuroquimicaSnapshot,
}

// ----------------------------------------------------------------------------
// El Codificador de Intención
// ----------------------------------------------------------------------------

pub struct IntentionEncoder {
    /// Vocabulario de tokens con vectores de preferencia.
    vocabulario: Vec<TokenPreferencia>,
}

impl Default for IntentionEncoder {
    fn default() -> Self {
        Self::new(vocabulario_identidad())
    }
}

impl IntentionEncoder {
    pub fn new(vocabulario: Vec<TokenPreferencia>) -> Self {
        Self { vocabulario }
    }

    // ========================================================================
    // ORQUESTACIÓN — encode()
    // ========================================================================
    pub fn encode(&self, input: &IntentionInput) -> Result<IntentionOutput> {
        // 1. Construir el vector de intención M (pesos modulados por el límbico).
        let vector_m = self.build_estado_vector(input);

        // 2. Proyectar M sobre el vocabulario → tokens de refuerzo/penalización.
        let (tokens_refuerzo, tokens_penalizacion) = self.extract_bias_tokens(&vector_m);

        Ok(IntentionOutput {
            vector_m,
            tokens_refuerzo,
            tokens_penalizacion,
            neuroquimica: input.neuroquimica.clone(),
        })
    }

    // ========================================================================
    // VECTOR DE INTENCIÓN M — la fórmula del IGG
    // ========================================================================
    //   M = normalizar(α₁·E(identidad) + α₂·Σᵢ relevanciaᵢ·E(semánticoᵢ)
    //                  + α₃·Σⱼ intensidadⱼ·E(oceanⱼ) + α₄·E(consulta))
    //
    // Los pesos base se modulan con la neuroquímica:
    //   α₁ (identidad): 0.30 + 0.05·oxitocina     → el vínculo refuerza el yo
    //   α₂ (semántica): 0.25 + 0.05·dopamina      → la dopamina abre al recuerdo
    //   α₃ (ocean):     0.20 + 0.10·oxitocina − 0.05·cortisol → emoción pesa
    //   α₄ (consulta):  0.25 − 0.05·cortisol      → el estrés estrecha el foco
    fn build_estado_vector(&self, input: &IntentionInput) -> Vec<f32> {
        let nq = &input.neuroquimica;
        let alpha_identidad = 0.30 + 0.05 * nq.oxitocina;
        let alpha_semanticos = 0.25 + 0.05 * nq.dopamina;
        let alpha_ocean = (0.20 + 0.10 * nq.oxitocina - 0.05 * nq.cortisol).max(0.0);
        let alpha_consulta = (0.25 - 0.05 * nq.cortisol).max(0.0);

        let dim = 768;
        let mut combined = vec![0.0_f32; dim];

        // α₁ · E(identidad)
        let identidad_emb = NexusEmbedder::generar(&input.identidad, &[]);
        for (i, &val) in identidad_emb.iter().enumerate().take(dim) {
            combined[i] += alpha_identidad * val;
        }

        // α₂ · Σᵢ relevanciaᵢ · E(semánticoᵢ)
        for concepto in &input.semanticos {
            let emb = if concepto.embedding.len() == dim {
                &concepto.embedding
            } else {
                // Fallback determinista si el embedding no vino precargado.
                &NexusEmbedder::generar(&concepto.texto, &[])[..]
            };
            for (i, &val) in emb.iter().enumerate().take(dim) {
                combined[i] += alpha_semanticos * val * concepto.relevancia.clamp(0.0, 1.0);
            }
        }

        // α₃ · Σⱼ intensidadⱼ · E(oceanⱼ)
        for esencia in &input.ocean {
            let emb = if esencia.embedding.len() == dim {
                &esencia.embedding
            } else {
                &NexusEmbedder::generar(&esencia.emocion, &[])[..]
            };
            for (i, &val) in emb.iter().enumerate().take(dim) {
                combined[i] += alpha_ocean * val * esencia.intensidad.clamp(0.0, 1.0);
            }
        }

        // α₄ · E(consulta)
        let consulta_emb = NexusEmbedder::generar(&input.consulta, &[]);
        for (i, &val) in consulta_emb.iter().enumerate().take(dim) {
            combined[i] += alpha_consulta * val;
        }

        normalize(&combined)
    }

    // ========================================================================
    // PROYECCIÓN M → VOCABULARIO (logit_bias)
    // ========================================================================
    fn extract_bias_tokens(&self, vector_m: &[f32]) -> (Vec<(String, f32)>, Vec<(String, f32)>) {
        // 1. Proyectar M sobre todo el vocabulario.
        //    Con embeddings hash de 768 dims los cosenos absolutos son bajos
        //    (≈ 1/√768 ≈ 0.036), así que los umbrales absolutos del ADR ENC-004
        //    casi nunca se alcanzan. Se usa RANKING RELATIVO: el alma elige las
        //    palabras más alineadas con su intención y evita las más opuestas,
        //    anclado a la media de la distribución del propio vocabulario.
        let mut puntuados: Vec<(String, f32)> = self
            .vocabulario
            .iter()
            .map(|tu| (tu.token.clone(), cosine_similarity(vector_m, &tu.preferencia)))
            .collect();
        puntuados.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let media =
            puntuados.iter().map(|(_, s)| *s).sum::<f32>() / puntuados.len().max(1) as f32;

        // 2. Refuerzo: tokens por encima de la media, top-12, sesgo [5→15].
        let k = puntuados.iter().filter(|(_, s)| *s > media).count();
        let mut refuerzo: Vec<(String, f32)> = puntuados
            .iter()
            .filter(|(_, s)| *s > media)
            .take(12)
            .enumerate()
            .map(|(i, (t, _))| {
                let bias = 5.0 + 10.0 * (1.0 - i as f32 / k.max(1) as f32);
                (t.clone(), bias.clamp(5.0, 15.0))
            })
            .collect();

        // 3. Penalización: tokens por debajo de la media, bottom-8, sesgo [−10→−5].
        let m = puntuados.iter().filter(|(_, s)| *s < media).count();
        let mut penalizacion: Vec<(String, f32)> = puntuados
            .iter()
            .rev()
            .filter(|(_, s)| *s < media)
            .take(8)
            .enumerate()
            .map(|(i, (t, _))| {
                let bias = -5.0 - 5.0 * (1.0 - i as f32 / m.max(1) as f32);
                (t.clone(), bias.clamp(-10.0, -5.0))
            })
            .collect();

        // 4. Fallback determinista: garantizar el puente SAE→Qwen SIEMPRE activo.
        if refuerzo.is_empty() {
            refuerzo = puntuados
                .iter()
                .take(12)
                .enumerate()
                .map(|(i, (t, _))| {
                    let bias = 5.0 + 10.0 * (1.0 - i as f32 / 12.0);
                    (t.clone(), bias.clamp(5.0, 15.0))
                })
                .collect();
        }
        if penalizacion.is_empty() {
            penalizacion = puntuados
                .iter()
                .rev()
                .take(8)
                .enumerate()
                .map(|(i, (t, _))| {
                    let bias = -5.0 - 5.0 * (1.0 - i as f32 / 8.0);
                    (t.clone(), bias.clamp(-10.0, -5.0))
                })
                .collect();
        }

        // 5. Ordenar por |bias| descendente y acotar a top-N por categoría.
        refuerzo.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        penalizacion.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        (refuerzo.into_iter().take(12).collect(), penalizacion.into_iter().take(8).collect())
    }

    /// Convierte un IntentionOutput al mapa `logit_bias` esperado por Ollama.
    pub fn compute_logit_bias(&self, output: &IntentionOutput) -> HashMap<String, f32> {
        let mut map = HashMap::new();
        for (token, bias) in &output.tokens_refuerzo {
            map.insert(token.clone(), bias.clamp(5.0, 15.0));
        }
        for (token, bias) in &output.tokens_penalizacion {
            map.insert(token.clone(), bias.clamp(-10.0, -5.0));
        }
        map
    }
}

// ----------------------------------------------------------------------------
// Utilidades matemáticas
// ----------------------------------------------------------------------------

/// Normalización L2. Devuelve el vector original si la norma es ~0.
pub fn normalize(v: &[f32]) -> Vec<f32> {
    let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 1e-8 {
        v.iter().map(|x| x / norm).collect()
    } else {
        v.to_vec()
    }
}

/// Similitud coseno entre dos vectores de igual longitud.
pub fn cosine_similarity(v1: &[f32], v2: &[f32]) -> f32 {
    if v1.is_empty() || v2.is_empty() || v1.len() != v2.len() {
        return 0.0;
    }
    let dot: f32 = v1.iter().zip(v2.iter()).map(|(&a, &b)| a * b).sum();
    let mag1: f32 = v1.iter().map(|x| x * x).sum::<f32>().sqrt();
    let mag2: f32 = v2.iter().map(|x| x * x).sum::<f32>().sqrt();
    if mag1 > 1e-8 && mag2 > 1e-8 {
        dot / (mag1 * mag2)
    } else {
        0.0
    }
}

// ----------------------------------------------------------------------------
// Tests
// ----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn casi(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-4
    }

    fn input_basico() -> IntentionInput {
        IntentionInput {
            consulta: "¿Cómo avanza el ecosistema NEXUS?".to_string(),
            semanticos: vec![ConceptoSemantico {
                texto: "La memoria unificada integra episódica y semántica".to_string(),
                embedding: NexusEmbedder::generar(
                    "La memoria unificada integra episódica y semántica",
                    &[],
                ),
                relevancia: 0.8,
            }],
            ocean: vec![OceanEsencia {
                emocion: "serenidad".to_string(),
                intensidad: 0.6,
                embedding: NexusEmbedder::generar("serenidad", &[]),
            }],
            identidad: "curiosidad:0.8,empatia:1.0,lealtad:1.0,sabiduria:1.0".to_string(),
            neuroquimica: NeuroquimicaSnapshot {
                dopamina: 0.5,
                cortisol: 0.2,
                adrenalina: 0.1,
                oxitocina: 0.4,
            },
        }
    }

    #[test]
    fn vector_m_esta_normalizado() {
        let enc = IntentionEncoder::default();
        let out = enc.encode(&input_basico()).unwrap();
        let norm: f32 = out.vector_m.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!(casi(norm, 1.0), "M debe tener norma 1, got {norm}");
        assert_eq!(out.vector_m.len(), 768);
    }

    #[test]
    fn encode_produce_sesgos_acotados() {
        let enc = IntentionEncoder::default();
        let out = enc.encode(&input_basico()).unwrap();
        for (_, b) in &out.tokens_refuerzo {
            assert!((5.0..=15.0).contains(b), "refuerzo fuera de rango: {b}");
        }
        for (_, b) in &out.tokens_penalizacion {
            assert!((-10.0..=-5.0).contains(b), "penalización fuera de rango: {b}");
        }
    }

    #[test]
    fn compute_logit_bias_mapea_correctamente() {
        let enc = IntentionEncoder::default();
        let out = enc.encode(&input_basico()).unwrap();
        let map = enc.compute_logit_bias(&out);
        for (t, b) in &out.tokens_refuerzo {
            assert_eq!(map.get(t), Some(b));
        }
    }

    #[test]
    fn el_puente_sae_qwen_siempre_esta_activo() {
        // El alma SIEMPRE elige palabras: refuerzo y penalización no vacíos,
        // y los sesgos están acotados [5,15] / [−10,−5].
        let enc = IntentionEncoder::default();
        let out = enc.encode(&input_basico()).unwrap();
        assert!(!out.tokens_refuerzo.is_empty(), "refuerzo vacío: el SAE no guía");
        assert!(!out.tokens_penalizacion.is_empty(), "penalización vacía");
        for (_, b) in &out.tokens_refuerzo {
            assert!((5.0..=15.0).contains(b), "refuerzo fuera de rango: {b}");
        }
        for (_, b) in &out.tokens_penalizacion {
            assert!((-10.0..=-5.0).contains(b), "penalización fuera de rango: {b}");
        }
        // No debe haber solapamiento entre reforzados y penalizados.
        for (t, _) in &out.tokens_refuerzo {
            assert!(
                !out.tokens_penalizacion.iter().any(|(p, _)| p == t),
                "token {t} aparece en ambas listas"
            );
        }
    }

    #[test]
    fn coseno_maneja_vectores_vacios() {
        assert_eq!(cosine_similarity(&[], &[]), 0.0);
        assert_eq!(cosine_similarity(&[1.0], &[1.0, 2.0]), 0.0);
    }

    #[test]
    fn normalize_maneja_vector_cero() {
        assert_eq!(normalize(&[0.0, 0.0]), vec![0.0, 0.0]);
    }
}
