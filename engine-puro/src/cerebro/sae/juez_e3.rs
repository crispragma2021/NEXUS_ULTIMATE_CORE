// ============================================================================
// 🧠 SAE v2 — JUEZ E3 (Evaluador de Coherencia del Bio-Transformer)
// ============================================================================
// Referenciado en ARQUITECTURA.md como el umbral de calidad para pasar de
// Fase 1 (frases coherentes) a Fase 2 (aprendizaje semántico profundo).
//
// Métricas cuantitativas (sin LLM externo — obra propia):
//   1. tasa_unk      : fracción de tokens generados que son <UNK> o slots
//                      vacíos del head (debe ser ~0).
//   2. coherencia    : fracción de tokens generados dentro del vocabulario
//                      real (complemento de tasa_unk).
//   3. diversidad    : tokens únicos / tokens totales (0..1). Alta = no repite
//                      en bucle; baja = patología (perseveración).
//   4. longitud_media: tokens promedio por secuencia generada.
//
// Puntuación E3 (0..1):
//   score = 0.6·coherencia + 0.4·diversidad
//
// Umbrales:
//   score >= 0.75  → Fase 2 (semántico profundo) — habilita destilación amplia.
//   score >= 0.50  → Fase 1 consolidada (frases coherentes).
//   score <  0.50  → Fase 0 (necesita más destilación desde NEXUS).
// ============================================================================

use crate::cerebro::sae::nucleo_numerico::Vocabulario;

/// Resultado de la evaluación de una secuencia.
#[derive(Debug, Clone)]
pub struct EvaluacionSecuencia {
    pub texto: String,
    pub tokens: Vec<usize>,
    pub tasa_unk: f32,
    pub coherencia: f32,
}

/// Resultado agregado del Juez E3 sobre un conjunto de muestras.
#[derive(Debug, Clone)]
pub struct DictamenE3 {
    pub muestras: usize,
    pub coherencia: f32,
    pub diversidad: f32,
    pub longitud_media: f32,
    pub score: f32,
    pub fase: FaseE3,
}

/// Fase cognitiva determinada por el Juez E3.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FaseE3 {
    /// Necesita más destilación desde NEXUS.
    Fase0,
    /// Fase 1 consolidada: frases coherentes.
    Fase1,
    /// Fase 2: aprendizaje semántico profundo habilitado.
    Fase2,
}

impl DictamenE3 {
    pub fn fase_a_texto(&self) -> &'static str {
        match self.fase {
            FaseE3::Fase0 => "Fase 0 — necesita más destilación desde NEXUS",
            FaseE3::Fase1 => "Fase 1 — frases coherentes (destilación en curso)",
            FaseE3::Fase2 => "Fase 2 — semántico profundo habilitado",
        }
    }
}

/// Evalúa una secuencia generada (ids + vocabulario) → coherencia.
pub fn evaluar_secuencia(ids: &[usize], vocabulario: &Vocabulario) -> EvaluacionSecuencia {
    let total = ids.len().max(1);
    let mut unk = 0usize;
    for id in ids {
        if *id == vocabulario.unk_id || *id >= vocabulario.tam() {
            unk += 1;
        }
    }
    let tasa_unk = unk as f32 / total as f32;
    EvaluacionSecuencia {
        texto: ids
            .iter()
            .map(|id| vocabulario.token_para(*id).to_string())
            .collect::<Vec<_>>()
            .join(" "),
        tokens: ids.to_vec(),
        tasa_unk,
        coherencia: 1.0 - tasa_unk,
    }
}

/// Dictamen final: agrega muestras y clasifica la fase.
///
/// `muestras` son los ids generados por el núcleo para cada estímulo semilla.
pub fn dictaminar(
    evaluadas: &[EvaluacionSecuencia],
    vocabulario: &Vocabulario,
) -> DictamenE3 {
    let n = evaluadas.len().max(1);

    // Coherencia media ponderada por longitud (las secuencias largas pesan más).
    let total_tokens: usize = evaluadas.iter().map(|e| e.tokens.len()).sum::<usize>().max(1);
    let tokens_ok: usize = evaluadas
        .iter()
        .flat_map(|e| e.tokens.iter())
        .filter(|id| **id != vocabulario.unk_id && **id < vocabulario.tam())
        .count();
    let coherencia = tokens_ok as f32 / total_tokens as f32;

    // Diversidad: tokens únicos sobre el total de posiciones generadas.
    let mut unicos = std::collections::HashSet::new();
    for e in evaluadas {
        for id in &e.tokens {
            if *id < vocabulario.tam() {
                unicos.insert(*id);
            }
        }
    }
    let diversidad = (unicos.len() as f32) / (total_tokens as f32).max(1.0);

    let longitud_media = total_tokens as f32 / n as f32;

    let score = 0.6 * coherencia + 0.4 * diversidad;

    let fase = if score >= 0.75 {
        FaseE3::Fase2
    } else if score >= 0.50 {
        FaseE3::Fase1
    } else {
        FaseE3::Fase0
    };

    DictamenE3 {
        muestras: evaluadas.len(),
        coherencia,
        diversidad,
        longitud_media,
        score,
        fase,
    }
}

/// Reporta el dictamen en formato legible.
pub fn reportar(d: &DictamenE3) -> String {
    format!(
        "🧑⚖️ JUEZ E3 — Dictamen:\n  muestras: {}\n  coherencia: {:.3}\n  diversidad: {:.3}\n  longitud media: {:.2}\n  score: {:.3}\n  fase: {}",
        d.muestras,
        d.coherencia,
        d.diversidad,
        d.longitud_media,
        d.score,
        d.fase_a_texto()
    )
}
