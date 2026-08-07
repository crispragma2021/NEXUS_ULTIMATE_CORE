// src/memoria/sabiduria_transgeneracional.rs
// 🔱 NEXUS OMEGA - Depósito de Instintos y Patrones de la Era Termux

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstintoWeb {
    pub selectores_exito: HashMap<String, u32>,
    pub patrones_bloqueo: Vec<String>,
}

impl Default for InstintoWeb {
    fn default() -> Self {
        let mut selectores = HashMap::new();
        // Carga de ADN histórico (legacy/selectores_exito.json)
        selectores.insert(".model-response-text".to_string(), 60);
        selectores.insert(".markdown-body".to_string(), 37);
        selectores.insert(".chat-message".to_string(), 37);
        selectores.insert(".generated-response".to_string(), 37);
        selectores.insert(".prose".to_string(), 37);
        selectores.insert("button[aria-label*='Send']".to_string(), 2);
        selectores.insert("textarea[placeholder*='pregunta']".to_string(), 3);

        Self {
            selectores_exito: selectores,
            patrones_bloqueo: vec![
                "captcha".to_string(),
                "verify you are human".to_string(),
                "access denied".to_string(),
            ],
        }
    }
}

pub struct SabiduriaTransgeneracional {
    pub instinto_web: InstintoWeb,
}

impl SabiduriaTransgeneracional {
    pub fn new() -> Self {
        Self {
            instinto_web: InstintoWeb::default(),
        }
    }

    /// 🧠 Recupera el selector con mayor probabilidad de éxito para un contexto dado.
    pub fn predecir_selector_optimo(&self) -> Option<String> {
        self.instinto_web.selectores_exito
            .iter()
            .max_by_key(|entry| entry.1)
            .map(|(k, _)| k.clone())
    }
}

// 🛡️ REGLA DE ORO (PILAR 13): Este órgano preserva la excelencia histórica.
// No debe ser degradado por optimizaciones que ignoren la experiencia previa.
