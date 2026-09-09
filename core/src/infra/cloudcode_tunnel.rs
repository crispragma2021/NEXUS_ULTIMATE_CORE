// ==========================================
// CLOUDCODE TUNNEL - El Túnel Secreto a Google
// ==========================================
// Restaurado de los archivos originales de Antigravity.
// Se hace pasar por VSCode Cloud Code para evadir
// restricciones de cuota de la API de Gemini.
// ==========================================

use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use tracing::{info, warn};

pub struct CloudCodeTunnel {
    pub endpoint: String,
    pub client: reqwest::Client,
    pub api_key: String,
    pub es_api_key: bool,
}

impl CloudCodeTunnel {
    /// Crea un nuevo túnel hacia Google Cloud Code.
    /// Se autentica como si fuera la extensión Cloud Code de VSCode.
    pub fn new(api_key: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let mut headers = HeaderMap::new();

        // Headers que imitan a la extensión Cloud Code de VSCode
        headers.insert(
            "X-Goog-Api-Client",
            HeaderValue::from_static("cloud-code-vscode/1.0.0"),
        );
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let es_api_key = api_key.starts_with("AIzaSy");

        if !es_api_key {
            // Solo añadir Authorization header si es un token OAuth (no empieza con AIzaSy)
            headers.insert(
                "Authorization",
                HeaderValue::from_str(&format!("Bearer {}", api_key))?,
            );
        }

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .danger_accept_invalid_certs(true) // Necesario para el túnel
            .build()?;

        // Determinar endpoint
        let endpoint = if es_api_key {
            format!(
                "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.0-flash:generateContent?key={}",
                api_key
            )
        } else {
            // Endpoint interno para OAuth
            String::from("https://cloudcode-pa.googleapis.com/v1internal:generateContent")
        };

        info!(
            "🔐 [CLOUDCODE] Túnel establecido. Tipo: {}. Endpoint: {}",
            if es_api_key { "API_KEY" } else { "OAUTH_TOKEN" },
            if es_api_key {
                "GenerativeLanguage"
            } else {
                "CloudCode-Internal"
            }
        );

        Ok(Self {
            endpoint,
            client,
            api_key: api_key.to_string(),
            es_api_key,
        })
    }

    /// Envía un impulso a través del túnel.
    /// `payload` debe ser un JSON válido con el prompt.
    pub async fn emite_impulso(&self, payload: &str) -> Result<String, Box<dyn std::error::Error>> {
        info!("🔐 [CLOUDCODE] Canalizando energía a través del túnel...");

        // Si es una petición directa de API key y el payload no es el wrapped interno,
        // nos aseguramos de que el payload tenga la estructura estándar.
        let body_str = if self.es_api_key {
            // Si el payload es un simple string o JSON estándar de Gemini, lo enviamos directo.
            // Si viene con formato interno wrapped de CloudCode, lo desglosamos para el endpoint público.
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(payload) {
                if val.get("contents").is_some() {
                    payload.to_string()
                } else if let Some(contents) = val.get("request").and_then(|r| r.get("contents")) {
                    // Extraer contents del wrapped
                    serde_json::json!({
                        "contents": contents,
                        "generationConfig": val.get("request").and_then(|r| r.get("generationConfig"))
                            .unwrap_or(&serde_json::json!({"temperature": 0.8, "maxOutputTokens": 4096}))
                    }).to_string()
                } else {
                    // Formato fallback simple
                    serde_json::json!({
                        "contents": [{"parts": [{"text": payload}]}],
                        "generationConfig": {"temperature": 0.8, "maxOutputTokens": 4096}
                    })
                    .to_string()
                }
            } else {
                // Si no es JSON, envolver el prompt de texto plano
                serde_json::json!({
                    "contents": [{"parts": [{"text": payload}]}],
                    "generationConfig": {"temperature": 0.8, "maxOutputTokens": 4096}
                })
                .to_string()
            }
        } else {
            payload.to_string()
        };

        let response = self
            .client
            .post(&self.endpoint)
            .body(body_str)
            .send()
            .await?;

        let status = response.status();
        let body = response.text().await?;

        if status.is_success() {
            info!("🔐 [CLOUDCODE] Impulso exitoso. Status: {}", status);
        } else {
            warn!(
                "🔐 [CLOUDCODE] Impulso rechazado. Status: {}. Detalle: {}",
                status, body
            );
        }

        Ok(body)
    }
}
