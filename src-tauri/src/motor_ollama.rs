// ============================================================================
// 🧬 MOTOR OLLAMA — Órgano de lenguaje natural local (Qwen2.5-7B vía Ollama)
// ============================================================================
// Encapsula la lógica de razonador_local (CoT) y razonador_busqueda
// como sub-órganos internos. Expone una interfaz limpia para que el
// NexoPuroEngine invoque al LLM local sin acoplamiento directo.
//
// Integración: NexoPuroEngine inyecta contexto cognitivo (identidad,
// emociones, OCEAN) antes de llamar a MotorOllama, y post-procesa la
// respuesta cruda del LLM a través de MotorFonacion.
// ============================================================================

// ─── Sub-órganos internos (refactorizados desde módulos top-level) ──────
#[path = "razonador_local.rs"]
pub mod razonador_local;

#[path = "razonador_busqueda.rs"]
pub mod razonador_busqueda;

use reqwest::Client;

// ─── Configuración del Motor ────────────────────────────────────────────

/// Configuración del MotorOllama.
/// Se inicializa con la API base y nombre del modelo desde env vars.
pub struct MotorOllama {
    pub api_base: String,
    pub model_name: String,
    client: Client,
}

impl MotorOllama {
    /// Crea una nueva instancia con los valores de entorno o defaults.
    /// Variables de entorno:
    /// - OLLAMA_API_BASE (default: http://localhost:11434)
    /// - OLLAMA_MODEL_NAME (default: qwen2.5:7b-instruct-q4_K_M)
    pub fn new() -> Self {
        let api_base = std::env::var("OLLAMA_API_BASE")
            .unwrap_or_else(|_| "http://localhost:11434".to_string());
        let model_name = std::env::var("OLLAMA_MODEL_NAME")
            .unwrap_or_else(|_| "qwen2.5:7b-instruct-q4_K_M".to_string());

        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(180))
            .build()
            .unwrap_or_default();

        Self { api_base, model_name, client }
    }

    /// Crea una instancia con configuración explícita.
    pub fn con_config(api_base: String, model_name: String) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(180))
            .build()
            .unwrap_or_default();

        Self { api_base, model_name, client }
    }

    // ─── Métodos de procesamiento ──────────────────────────────────────

    /// Procesa un prompt con Chain-of-Thought completo.
    /// El prompt DEBE estar contextualizado (el engine inyecta identidad,
    /// emociones, etc. antes de llamar a este método).
    ///
    /// Retorna la respuesta razonada completa con metadatos de latencia.
    pub async fn procesar_con_cot(&self, prompt: &str) -> razonador_local::RespuestaRazonada {
        razonador_local::procesar_con_cot(prompt, &self.api_base, &self.model_name).await
    }

    /// Procesa un prompt con búsqueda inteligente (Tool-Augmented Reasoning).
    /// Si no necesita búsqueda o falla, retorna None.
    ///
    /// El prompt DEBE estar contextualizado.
    pub async fn procesar_con_busqueda(&self, prompt: &str) -> Option<razonador_busqueda::RespuestaBusqueda> {
        razonador_busqueda::procesar_con_busqueda(prompt, &self.api_base, &self.model_name).await
    }

    /// Llama directamente al LLM sin CoT ni búsqueda.
    /// Útil para respuestas rápidas donde el engine ya clasificó que no
    /// necesita razonamiento aumentado.
    pub async fn llamada_directa(&self, prompt: &str) -> String {
        // Usamos el helper NDJSON de razonador_local
        let system_prompt = r#"Eres un asistente útil y conciso. Responde directamente a la pregunta."#;
        let messages = vec![
            serde_json::json!({"role": "system", "content": system_prompt}),
            serde_json::json!({"role": "user", "content": prompt}),
        ];

        let request_body = serde_json::json!({
            "model": self.model_name,
            "messages": messages,
            "options": {
                "temperature": 0.3,
                "top_p": 0.9,
                "top_k": 40,
            },
            "stream": false,
        });

        match self.client
            .post(format!("{}/api/chat", self.api_base))
            .json(&request_body)
            .send()
            .await
        {
            Ok(res) => {
                match razonador_local::extraer_contenido_ollama(res).await {
                    Ok(contenido) => contenido,
                    Err(e) => format!("❌ Error parseando respuesta de Ollama: {}", e),
                }
            }
            Err(e) => format!("❌ Error de conexión con Ollama: {}", e),
        }
    }
}

impl Default for MotorOllama {
    fn default() -> Self {
        Self::new()
    }
}
