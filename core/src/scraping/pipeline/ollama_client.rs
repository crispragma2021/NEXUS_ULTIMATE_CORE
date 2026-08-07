//! Cliente Ollama Rust para inferencia SLM local Tier-1 (F2.1).
//!
//! Se comunica con Ollama vía `POST /api/generate` en `127.0.0.1:11434`.
//! Configuración recomendada (spec §4.1):
//! - Modelo: `nexuslocal-free:latest` (Qwen 2.5 abliterated, E0) o `qwen2.5:7b`.
//! - `num_ctx: 32768` — contexto amplio, KV cache en RAM.
//! - `format: json` — fuerza salida JSON sin texto de relleno.

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

/// URL base del servidor Ollama local.
pub const OLLAMA_URL: &str = "http://127.0.0.1:11434";

/// Modelo por defecto (Qwen 2.5 sin censura de E0).
pub const DEFAULT_MODEL: &str = "nexuslocal-free:latest";

/// Tamaño de ventana de contexto.
pub const DEFAULT_NUM_CTX: u64 = 32768;

/// Timeout de inferencia (spec §3.3: 120s para SLM local).
const TIMEOUT_SECS: u64 = 120;

/// Respuesta de `/api/generate`.
#[derive(Debug, Clone, Deserialize)]
pub struct GenerateResponse {
    pub response: String,
    #[serde(default)]
    pub done: bool,
    #[serde(default)]
    pub eval_count: Option<u64>,
    #[serde(default)]
    pub eval_duration: Option<u64>,
}

impl GenerateResponse {
    /// Tokens por segundo estimados (eval_count / (eval_duration/1e9)).
    pub fn tokens_per_second(&self) -> f64 {
        match (self.eval_count, self.eval_duration) {
            (Some(c), Some(d)) if d > 0 => (c as f64) / (d as f64 / 1e9),
            _ => 0.0,
        }
    }
}

/// Configuración de una invocación al SLM local.
#[derive(Debug, Clone)]
pub struct OllamaConfig {
    pub model: String,
    pub num_ctx: u64,
    pub temperature: f64,
    /// Si es `true`, Ollama fuerza la salida a JSON válido (`format: "json"`).
    pub force_json: bool,
}

impl Default for OllamaConfig {
    fn default() -> Self {
        Self {
            model: DEFAULT_MODEL.to_string(),
            num_ctx: DEFAULT_NUM_CTX,
            temperature: 0.1,
            force_json: true,
        }
    }
}

/// Cliente HTTP para Ollama.
#[derive(Clone)]
pub struct OllamaClient {
    client: reqwest::Client,
    pub config: OllamaConfig,
}

impl OllamaClient {
    pub fn new(config: OllamaConfig) -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(TIMEOUT_SECS))
            .build()?;
        Ok(Self { client, config })
    }

    /// Cliente con configuración por defecto.
    pub fn default_client() -> Result<Self> {
        Self::new(OllamaConfig::default())
    }

    /// Genera una respuesta de texto libre.
    pub async fn generate(&self, prompt: &str) -> Result<GenerateResponse> {
        self.call(prompt, None, self.config.force_json).await
    }

    /// Genera y extrae un JSON estructurado según `system_prompt` (opcional).
    ///
    /// Devuelve el `Value` parseado de la respuesta.
    pub async fn extract_json(
        &self,
        prompt: &str,
        system_prompt: Option<&str>,
    ) -> Result<Value> {
        let resp = self.call(prompt, system_prompt, true).await?;
        let parsed: Value = serde_json::from_str(&resp.response)
            .context("Ollama no devolvió JSON válido pese a format=json")?;
        Ok(parsed)
    }

    /// Invocación base a `/api/generate`.
    async fn call(
        &self,
        prompt: &str,
        system_prompt: Option<&str>,
        force_json: bool,
    ) -> Result<GenerateResponse> {
        let mut body = json!({
            "model": self.config.model,
            "prompt": prompt,
            "stream": false,
            "options": {
                "num_ctx": self.config.num_ctx,
                "temperature": self.config.temperature,
            }
        });
        if force_json {
            body["format"] = json!("json");
        }
        if let Some(sys) = system_prompt {
            body["system"] = json!(sys);
        }

        let resp = self
            .client
            .post(format!("{OLLAMA_URL}/api/generate"))
            .json(&body)
            .send()
            .await
            .context("Error conectando con Ollama API")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(anyhow!("Ollama respondió HTTP {}: {}", status, body));
        }

        let data: GenerateResponse = resp
            .json()
            .await
            .context("Error parseando respuesta JSON de Ollama")?;
        Ok(data)
    }
}

/// Prompt de sistema para extracción JSON estructurada (spec F2.2).
pub const EXTRACTION_SYSTEM_PROMPT: &str = r#"Eres el motor de extracción NEXUS. Dado un fragmento de texto web, extrae los datos relevantes y devuelve ÚNICAMENTE un objeto JSON válido con esta estructura:
{"entities": ["...", "..."], "prices": [{"item": "...", "price": 0.0, "currency": "USD"}], "key_facts": ["...", "..."]}
Reglas:
- Si un dato no está presente, usa una lista vacía [].
- No añadas texto fuera del JSON.
- Precios: conviértelos a número flotante; si no hay moneda, usa null."#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_default_usa_modelo_e0_y_contexto_amplio() {
        let cfg = OllamaConfig::default();
        assert_eq!(cfg.model, DEFAULT_MODEL);
        assert_eq!(cfg.num_ctx, 32768);
        assert!(cfg.force_json);
    }

    #[test]
    fn tokens_por_segundo_calcula_correctamente() {
        let resp = GenerateResponse {
            response: "ok".into(),
            done: true,
            eval_count: Some(100),
            eval_duration: Some(2_000_000_000), // 2s
        };
        assert_eq!(resp.tokens_per_second(), 50.0);
    }

    #[test]
    fn tokens_por_segundo_cero_sin_metadatos() {
        let resp = GenerateResponse {
            response: "ok".into(),
            done: true,
            eval_count: None,
            eval_duration: None,
        };
        assert_eq!(resp.tokens_per_second(), 0.0);
    }
}
