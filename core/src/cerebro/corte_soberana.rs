// ⚖️ CORTE SOBERANA NEXUS — Motor Híbrido Gemini-DeepSeek
// Sincronía perfecta entre Contexto (Gemini) y Lógica (DeepSeek).

use crate::energia::zenith_pool::ZenithPool;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::info;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DebateEntry {
    pub role: String,
    pub model: String,
    pub argument: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Verdict {
    pub final_decision: String,
    pub reasoning: String,
    pub confidence: f32,
    pub debate_log: Vec<DebateEntry>,
}

pub struct CorteSoberana {
    pool: Arc<ZenithPool>,
}

impl CorteSoberana {
    pub fn new() -> Self {
        Self {
            pool: Arc::new(ZenithPool::new()),
        }
    }

    pub async fn debatir(&self, asunto: &str) -> Verdict {
        info!("⚖️ Sesión Judicial Híbrida Iniciada...");
        let mut log = Vec::new();

        // 1. ANÁLISIS DE CONTEXTO (Prioridad Vertex AI / Gemini 3)
        let analista_prompt = format!(
            "Analiza este asunto con TODO tu contexto: {}. Sé técnico y preciso.",
            asunto
        );
        let analisis = self
            .pool
            .responder_estrategico(&analista_prompt, "contexto_judicial")
            .await;
        log.push(DebateEntry {
            role: "Analista".to_string(),
            model: "Gemini 3 (Vertex/Studio)".to_string(),
            argument: analisis.clone(),
        });

        // 2. VEREDICTO LÓGICO (DeepSeek R1)
        let juez_prompt = format!(
            "Actúa como el JUEZ SOBERANO de NEXUS. Evalúa este análisis previo:\n{}\n\nSobre el asunto: {}\n\nEmite tu VEREDICTO FINAL: [EJECUTAR] o [ABORTAR]. Sé extremadamente crítico.",
            analisis, asunto
        );
        let veredicto_raw = self.pool.ejecutor_deepseek(&juez_prompt).await;

        let decision = if veredicto_raw.to_uppercase().contains("[EJECUTAR]") {
            "EJECUTAR".to_string()
        } else if veredicto_raw.to_uppercase().contains("[ABORTAR]") {
            "ABORTAR".to_string()
        } else {
            "ABORTAR".to_string() // Por seguridad, si hay duda, se aborta.
        };

        Verdict {
            final_decision: decision,
            reasoning: veredicto_raw,
            confidence: 0.98,
            debate_log: log,
        }
    }
}
