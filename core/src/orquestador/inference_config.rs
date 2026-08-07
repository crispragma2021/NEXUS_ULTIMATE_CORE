// 🔱 SLM INFERENCE CONFIG — Parámetros de inferencia para Modelos Locales Pequeños (SLMs)
// Configuración determinista y estricta para forzar formato JSON y evitar alucinaciones.
// Puro Rust, sin unwrap(), sin expect(), listo para producción.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SLMInferenceConfig {
    pub temperature: f32,          // 0.0 — determinista
    pub top_p: f32,                // 0.1 — vocabulario muy restringido
    pub top_k: u32,                // 1 — solo el token más probable
    pub repeat_penalty: f32,       // 1.1 — evita bucles repetitivos
    pub max_tokens: u32,           // 512 — suficiente para JSON de tool calls
    pub stop_tokens: Vec<String>,  // Tokens de parada
    pub json_mode: bool,           // Forzar modo JSON en Ollama/mistralrs
}

impl Default for SLMInferenceConfig {
    fn default() -> Self {
        Self {
            temperature: 0.0,
            top_p: 0.1,
            top_k: 1,
            repeat_penalty: 1.1,
            max_tokens: 512,
            stop_tokens: vec![
                "<|eot_id|>".to_string(),
                "<|im_end|>".to_string(),
                "</s>".to_string(),
                "}\n\n".to_string(),
            ],
            json_mode: true,
        }
    }
}
