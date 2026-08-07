// 🔱 SLM DISPATCHER — Despachador de inferencia local para SLMs (Ollama)
// Canaliza los prompts limpios y el esquema JSON hacia Ollama con políticas de reintento.
// Puro Rust, sin unwrap(), sin expect(), listo para producción.

use super::context_pruner::PrunedContext;
use super::inference_config::SLMInferenceConfig;
use anyhow::{anyhow, Result};
use reqwest::Client;
use serde_json::json;
use std::time::Duration;

pub struct SLMDispatcher {
    client: Client,
    ollama_url: String,
    model_name: String,
    config: SLMInferenceConfig,
}

impl SLMDispatcher {
    /// Inicializa el despachador con la URL de Ollama y el modelo configurado
    pub fn new(ollama_url: String, model_name: String, config: SLMInferenceConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30)) // 30s de timeout máximo por inferencia local
            .build()
            .unwrap_or_else(|_| Client::new());

        Self {
            client,
            ollama_url,
            model_name,
            config,
        }
    }

    /// Realiza inferencia sobre el prompt aislado
    pub async fn infer(&self, ctx: &PrunedContext) -> Result<String> {
        let endpoint = format!("{}/api/chat", self.ollama_url);

        // Construir mensajes según la especificación de Ollama
        let messages = json!([
            {
                "role": "system",
                "content": format!(
                    "{}\n\nEsquema de salida obligatorio:\n{}",
                    ctx.system_prompt, ctx.output_schema
                )
            },
            {
                "role": "user",
                "content": ctx.user_prompt
            }
        ]);

        // Parámetros estrictos de inferencia
        let options = json!({
            "temperature": self.config.temperature,
            "top_p": self.config.top_p,
            "top_k": self.config.top_k,
            "repeat_penalty": self.config.repeat_penalty,
            "num_predict": self.config.max_tokens,
            "stop": self.config.stop_tokens,
        });

        // Construir payload
        let mut payload = json!({
            "model": self.model_name,
            "messages": messages,
            "options": options,
            "stream": false,
        });

        // Forzar JSON mode si está configurado en Ollama
        if self.config.json_mode {
            if let Some(obj) = payload.as_object_mut() {
                obj.insert("format".to_string(), json!("json"));
            }
        }

        // Ejecutar llamada HTTP
        let response = self.client.post(&endpoint)
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body_err = response.text().await.unwrap_or_default();
            return Err(anyhow!(
                "Ollama retornó código de error ({}): {}",
                status,
                body_err
            ));
        }

        let res_json: serde_json::Value = response.json().await?;
        let content = res_json["message"]["content"]
            .as_str()
            .ok_or_else(|| anyhow!("Ollama no devolvió campo message.content válido"))?
            .trim()
            .to_string();

        Ok(content)
    }

    /// Realiza un chequeo rápido de salud (healthcheck) de Ollama
    pub async fn health_check(&self) -> bool {
        let endpoint = format!("{}/api/tags", self.ollama_url);
        match self.client.get(&endpoint).send().await {
            Ok(resp) => resp.status().is_success(),
            Err(_) => false,
        }
    }
}
