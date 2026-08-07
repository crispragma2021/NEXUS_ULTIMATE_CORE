// ==========================================
// LÓBULO OCCIPITAL ESTÉTICO - Juicio Visual
// ==========================================
// Evalúa interfaces visuales con Gemini Vision
// No solo "existe", sino "es bonito/funcional"
// ==========================================

use reqwest::Client;
use serde_json::{json, Value};
use std::env;

pub struct LobuloOccipitalEstetico {
    client: Client,
    api_key: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EvaluacionEstetica {
    pub puntaje: f64,
    pub es_funcional: bool,
    pub es_bonito: bool,
    pub feedback: String,
    pub areas_mejora: Vec<String>,
}

impl Default for LobuloOccipitalEstetico {
    fn default() -> Self {
        Self::new()
    }
}

impl LobuloOccipitalEstetico {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            api_key: env::var("GEMINI_API_KEY").unwrap_or_default(),
        }
    }

    pub async fn evaluar_ui(&self, screenshot_base64: &str) -> EvaluacionEstetica {
        let prompt = r#"
        Evalúa esta interfaz de usuario en 4 aspectos:
        1. Funcionalidad: ¿Los elementos son claros y usables?
        2. Estética: ¿Es visualmente agradable?
        3. Coherencia: ¿Los colores/fuentes/espacios son consistentes?
        4. Claridad: ¿Se entiende qué hace cada cosa?

        Responde SOLO en JSON:
        {
            "puntaje": 0.0-1.0,
            "funcional": true/false,
            "bonito": true/false,
            "feedback": "texto breve",
            "mejoras": ["lista", "de", "sugerencias"]
        }
        "#;

        let body = json!({
            "contents": [{
                "parts": [
                    {"text": prompt},
                    {"inline_data": {"mime_type": "image/png", "data": screenshot_base64}}
                ]
            }]
        });

        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.0-flash:generateContent?key={}",
            self.api_key
        );

        match self.client.post(&url).json(&body).send().await {
            Ok(resp) => {
                match resp.json::<Value>().await {
                    Ok(json) => {
                        let text = json["candidates"][0]["content"]["parts"][0]["text"]
                            .as_str()
                            .unwrap_or("{}")
                            .to_string();
                        // Limpiar posibles marcadores ```json ... ```
                        let cleaned = text
                            .trim_start_matches("```json")
                            .trim_start_matches("```")
                            .trim_end_matches("```")
                            .trim();
                        serde_json::from_str(cleaned).unwrap_or(EvaluacionEstetica {
                            puntaje: 0.5,
                            es_funcional: false,
                            es_bonito: false,
                            feedback: "No se pudo parsear la respuesta".to_string(),
                            areas_mejora: vec![],
                        })
                    }
                    Err(e) => EvaluacionEstetica {
                        puntaje: 0.0,
                        es_funcional: false,
                        es_bonito: false,
                        feedback: format!("Error parseando JSON: {}", e),
                        areas_mejora: vec![],
                    },
                }
            }
            Err(e) => EvaluacionEstetica {
                puntaje: 0.0,
                es_funcional: false,
                es_bonito: false,
                feedback: format!("Error de conexión: {}", e),
                areas_mejora: vec![],
            },
        }
    }
}
