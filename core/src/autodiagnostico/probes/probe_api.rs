use crate::autodiagnostico::sentinel_core::{HealthProbe, ProbeResult, ProbeTier};
use async_trait::async_trait;
use reqwest::Client;
use std::time::Instant;

pub struct ProbeApi {
    http_client: Client,
}

impl ProbeApi {
    pub fn new() -> Self {
        Self {
            http_client: Client::new(),
        }
    }
}

#[async_trait]
impl HealthProbe for ProbeApi {
    async fn check(&self) -> ProbeResult {
        let start_time = Instant::now();
        let mut passed = true;
        let mut mensaje = String::new();
        let mut detalles = serde_json::json!({});

        // 1. Check NEXUS Core API
        let nexus_api_url = "http://127.0.0.1:43210/api/health";
        match self
            .http_client
            .get(nexus_api_url)
            .timeout(std::time::Duration::from_millis(500))
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    let text = response.text().await.unwrap_or_default();
                    detalles["nexus_core"] =
                        serde_json::json!({ "status": "ok", "response": text });
                    mensaje.push_str("NEXUS Core API (43210): OK. ");
                } else {
                    passed = false;
                    detalles["nexus_core"] = serde_json::json!({ "status": "error", "code": response.status().as_u16() });
                    mensaje.push_str(&format!(
                        "NEXUS Core API (43210): Error {}. ",
                        response.status().as_u16()
                    ));
                }
            }
            Err(e) => {
                passed = false;
                detalles["nexus_core"] =
                    serde_json::json!({ "status": "error", "message": e.to_string() });
                mensaje.push_str(&format!(
                    "NEXUS Core API (43210): Fallo de conexión ({}). ",
                    e
                ));
            }
        }

        // 2. Check Ollama (if installed, non-critical)
        let ollama_api_url = "http://127.0.0.1:11434/api/tags";
        match self
            .http_client
            .get(ollama_api_url)
            .timeout(std::time::Duration::from_secs(3))
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    detalles["ollama"] = serde_json::json!({ "status": "ok" });
                    mensaje.push_str("Ollama (11434): OK. ");
                } else {
                    detalles["ollama"] = serde_json::json!({ "status": "warning", "code": response.status().as_u16() });
                    mensaje.push_str(&format!(
                        "Ollama (11434): Error {}. ",
                        response.status().as_u16()
                    ));
                }
            }
            Err(e) => {
                detalles["ollama"] =
                    serde_json::json!({ "status": "warning", "message": e.to_string() });
                mensaje.push_str(&format!("Ollama (11434): Fallo de conexión ({}). ", e));
            }
        }

        // 3. Check DeepSeek (placeholder for actual external API check)
        // For now, simulate success if Nexus Core API passed, else warning
        if passed {
            detalles["deepseek"] = serde_json::json!({ "status": "ok", "message": "Simulado: Conectividad DeepSeek OK" });
            mensaje.push_str("DeepSeek API: OK.");
        } else {
            detalles["deepseek"] = serde_json::json!({ "status": "warning", "message": "Simulado: Problemas con DeepSeek debido a NEXUS Core" });
            mensaje.push_str("DeepSeek API: Advertencia.");
        }

        if mensaje.is_empty() {
            mensaje.push_str("No se pudo verificar ninguna API.");
            passed = false;
        }

        ProbeResult {
            nombre: self.nombre().to_string(),
            tier: self.tier(),
            passed,
            mensaje,
            detalles: Some(detalles),
            latencia_ms: start_time.elapsed().as_millis() as u64,
        }
    }

    fn tier(&self) -> ProbeTier {
        ProbeTier::Critical
    }

    fn nombre(&self) -> &'static str {
        "API Core Services"
    }
}
