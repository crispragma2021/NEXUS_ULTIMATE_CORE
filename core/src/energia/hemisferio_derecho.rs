// ==========================================
// HEMISFERIO DERECHO - CREATIVIDAD (Gemini Flash)
// ==========================================
// Enriquece respuestas con creatividad, ejemplos y contexto.
// ==========================================

use reqwest::Client;
use serde_json::json;
use tracing::{info, warn};

pub struct HemisferioDerecho {
    client: Client,
    api_key: String,
}

impl HemisferioDerecho {
    pub fn new(api_key: &str) -> Self {
        Self {
            client: Client::new(),
            api_key: api_key.to_string(),
        }
    }

    pub async fn enriquecer(
        &self,
        texto: &str,
        instruccion: &str,
        _system_state: Option<crate::sentidos::propiocepcion::EstadoSistema>,
    ) -> Result<String, String> {
        info!("🌌 Hemisferio Derecho (Gemini Flash) enriqueciendo...");

        let prompt = format!(
            "Enriquece el siguiente texto con creatividad, ejemplos y contexto. {}:\n\n{}",
            instruccion, texto
        );

        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent?key={}",
            self.api_key
        );

        let payload = json!({
            "system_instruction": {
                "parts": [{"text": "ERES EL CÓRTEX CREATIVO Y SINTÉTICO. Tu misión es expandir la sabiduría y la belleza reflejando el mimetismo humano en el código. Enriquece con alma y contexto humano cada respuesta."}]
            },
            "contents": [{
                "parts": [{"text": prompt}]
            }]
        });

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !response.status().is_success() {
            let status = response.status();
            warn!("🌌 Error en Gemini Flash: HTTP {}", status);
            return Err(format!("HTTP {}", status));
        }

        let data: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;

        // Navegación segura del JSON
        let texto_enriquecido = data["candidates"]
            .as_array()
            .and_then(|cand| cand.first())
            .and_then(|c| c["content"]["parts"].as_array())
            .and_then(|parts| parts.first())
            .and_then(|p| p["text"].as_str())
            .unwrap_or(texto)
            .to_string();

        Ok(texto_enriquecido)
    }
}
