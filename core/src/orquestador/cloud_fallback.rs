// 🔱 CLOUD FALLBACK — Puerta de escalación inteligente a Nube
// Se activa únicamente cuando el SLM local agota sus reintentos de forma consecutiva.
// Puro Rust, sin unwrap(), sin expect(), listo para producción.

use crate::efectores::agente_ejecutor::ToolResponse;
use crate::orquestador::task_graph::TaskNode;
use crate::energia::zenith_pool::ZenithPool;
use anyhow::{anyhow, Result};
use std::sync::Arc;

pub struct CloudFallback {
    pub zenith: Arc<ZenithPool>,
}

impl CloudFallback {
    /// Inicializa la puerta de escalación con el Pool Zenith existente de NEXUS
    pub fn new(zenith: Arc<ZenithPool>) -> Self {
        Self { zenith }
    }

    /// Escala la tarea atómica a la nube (ej: Gemini 2.5 Pro) para asegurar resolución determinista
    pub async fn execute_fallback(&self, task: &TaskNode, accum_error: &str) -> Result<ToolResponse> {
        tracing::warn!(
            "🚨 [ESCALACIÓN] Escalando tarea '{}' a nube por fallos persistentes del SLM local.",
            task.id
        );

        let system_prompt = r#"Eres el Planificador Nube de ÉLITE de NEXUS. Tu tarea es resolver una operación que el SLM local no ha podido estructurar correctamente o ha fallado consecutivamente.
Debes devolver un JSON absolutamente perfecto cumpliendo con la acción y el esquema solicitado.
No agregues comentarios ni markdown. Solo el JSON."#.to_string();

        let prompt = format!(
            "ID de Tarea: {}\nInstrucción de Tarea: {}\nEsquema Requerido: {}\n\nHistorial de errores acumulados localmente:\n{}\n\nPor favor, genera la acción corregida en un JSON estricto:",
            task.id,
            task.instruction,
            serde_json::to_string(&task.output_schema).unwrap_or_default(),
            accum_error
        );

        // Invocar Zenith Pool (que gestiona de forma inteligente Gemini, DeepSeek, Groq, Vertex)
        let full_prompt = format!("{}\n\nUser:\n{}", system_prompt, prompt);
        let cloud_output = self.zenith.cerebro_gemini(&full_prompt, "gemini-2.0-pro").await;

        if cloud_output.is_empty() || cloud_output.contains("Sin energía") {
            // Intentar con DeepSeek como backup alternativo
            tracing::warn!("⚠️ Falló Gemini en la nube. Intentando fallback secundario con DeepSeek...");
            let ds_output = self.zenith.ejecutor_deepseek(&full_prompt).await;
            if ds_output.is_empty() {
                return Err(anyhow!("Ambos modelos nube (Gemini y DeepSeek) fallaron al resolver la escalación"));
            }
            return Ok(ToolResponse {
                success: true,
                output: ds_output,
            });
        }

        Ok(ToolResponse {
            success: true,
            output: cloud_output,
        })
    }
}
