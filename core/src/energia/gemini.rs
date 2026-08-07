// ==========================================
// GEMINI 3 FLASH - ARQUITECTO ESTRATÉGICO
// ==========================================
// Cerebro del Triunvirato. Planifica, decide,
// y delega tareas complejas al Ejecutor.
// ==========================================

use reqwest::Client;
use serde_json::{json, Value};
use std::env;

pub struct GeminiArchitect {
    client: Client,
    api_key: String,
    model: String,
}

impl Default for GeminiArchitect {
    fn default() -> Self {
        Self::new()
    }
}

impl GeminiArchitect {
    pub fn new() -> Self {
        let api_key = env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY no está configurada");

        Self {
            client: Client::new(),
            api_key,
            model: "gemini-2.5-flash".to_string(),
        }
    }

    // Pensamiento estratégico (Gemini decide qué hacer)
    pub async fn pensar(&self, prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            self.model, self.api_key
        );

        let payload = json!({
            "contents": [{
                "parts": [{
                    "text": format!(
                        "Eres el ARQUITECTO de NEXUS, el cerebro estratégico. \
                         Analiza la siguiente petición y decide si:\n\
                         1. RESPUESTA_DIRECTA (si es simple, responde tú mismo)\n\
                         2. DELEGAR_A_EJECUTOR (si requiere código, análisis local o sin censura)\n\n\
                         Responde con un JSON:\n\
                         {{\"accion\": \"RESPUESTA_DIRECTA\" o \"DELEGAR_A_EJECUTOR\",\n\
                          \"contenido\": \"tu respuesta o instrucción para el ejecutor\"}}\n\n\
                         Petición: {}",
                        prompt
                    )
                }]
            }],
            "generationConfig": {
                "temperature": 0.7,
                "maxOutputTokens": 8192
            }
        });

        let response = self.client.post(&url).json(&payload).send().await?;

        let data: Value = response.json().await?;

        if let Some(text) = data["candidates"][0]["content"]["parts"][0]["text"].as_str() {
            Ok(text.to_string())
        } else {
            Ok("{\"accion\": \"RESPUESTA_DIRECTA\", \"contenido\": \"No pude procesar la solicitud\"}".to_string())
        }
    }

    // Respuesta directa de Gemini (sin delegar)
    pub async fn responder_directo(
        &self,
        prompt: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            self.model, self.api_key
        );

        let payload = json!({
            "contents": [{
                "parts": [{"text": prompt}]
            }],
            "generationConfig": {
                "temperature": 0.7,
                "maxOutputTokens": 8192
            }
        });

        let response = self.client.post(&url).json(&payload).send().await?;

        let data: Value = response.json().await?;

        Ok(data["candidates"][0]["content"]["parts"][0]["text"]
            .as_str()
            .unwrap_or("")
            .to_string())
    }
}
