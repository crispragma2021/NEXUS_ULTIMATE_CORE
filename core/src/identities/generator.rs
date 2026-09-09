// 🌱 NEXUS OMEGA — Generador de Perfiles Sintéticos vía Mistral
// ============================================================

use crate::identities::types::{IdentityProfile, SyntheticIdentity};
use anyhow::{anyhow, Result};
use reqwest::Client;
use serde_json::Value;
use std::time::Duration;

/// Genera identidades sintéticas creíbles usando Mistral API
pub struct IdentityGenerator {
    client: Client,
    api_key: String,
}

impl Default for IdentityGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl IdentityGenerator {
    pub fn new() -> Self {
        let api_key = std::env::var("MISTRAL_API_KEY")
            .unwrap_or_else(|_| "vwMgOsXzugzxZaKjfbYLyLQQqOMf1f0A".to_string());

        let client = Client::builder()
            .timeout(Duration::from_secs(60))
            .build()
            .unwrap_or_default();

        Self { client, api_key }
    }

    /// Genera N identidades sintéticas usando Mistral
    pub async fn generate(&self, count: usize) -> Result<Vec<SyntheticIdentity>> {
        let mut identities = Vec::with_capacity(count);

        for i in 0..count {
            println!("🌱 Generando identidad {}/{}...", i + 1, count);
            let profile = self.generate_profile().await?;
            let mut identity = SyntheticIdentity::new(profile);

            // Asignar fingerprint único por identidad
            identity.fingerprint = self.generate_fingerprint(&identity);

            identities.push(identity);
        }

        Ok(identities)
    }

    /// Genera un perfil de persona sintética vía Mistral
    pub async fn generate_profile(&self) -> Result<IdentityProfile> {
        let prompt = "Genera un perfil de persona sintética en formato JSON \
            con los siguientes campos exactos: full_name, gender, age, nationality, \
            occupation, city, country, bio (2 frases), traits (array de 3-5 rasgos \
            de personalidad), interests (array de 3-5 intereses). \
            La persona debe ser de Paraguay o Latinoamérica, nombre realista, \
            edad entre 22-55 años, ocupación creíble. \
            Responde SOLO el JSON, sin explicaciones.";

        let json_body = serde_json::json!({
            "model": "mistral-small-latest",
            "messages": [
                {"role": "system", "content": "Eres un generador de datos sintéticos. Respondes SOLO con JSON válido."},
                {"role": "user", "content": prompt}
            ],
            "temperature": 0.8,
            "max_tokens": 1024,
        });

        let resp = self
            .client
            .post("https://api.mistral.ai/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&json_body)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(anyhow!("Mistral API error {}: {}", status, text));
        }

        let data: Value = resp.json().await?;
        let content = data["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| anyhow!("No content in Mistral response"))?;

        // Extraer JSON del contenido (puede venir con markdown fences)
        let json_str = extract_json(content)?;
        let profile: IdentityProfile = serde_json::from_str(&json_str)
            .map_err(|e| anyhow!("Error parsing profile JSON: {} — content: {}", e, json_str))?;

        Ok(profile)
    }

    /// Genera una huella digital única para la identidad
    fn generate_fingerprint(
        &self,
        _identity: &SyntheticIdentity,
    ) -> crate::identities::types::IdentityFingerprint {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        let ua = format!(
            "Mozilla/5.0 ({}; {} {}) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/{}.0.{}.{} Safari/537.36",
            if rng.gen_bool(0.5) { "X11; Linux x86_64" } else { "Windows NT 10.0; Win64; x64" },
            match rng.gen_range(0..4) {
                0 => "en-US",
                1 => "es-ES",
                2 => "es-PY",
                _ => "pt-BR",
            },
            if rng.gen_bool(0.3) { " rv:115.0" } else { "" },
            rng.gen_range(115..125),
            rng.gen_range(0..5),
            rng.gen_range(1000..9999),
        );

        let resolutions = [
            "1920x1080",
            "1366x768",
            "1536x864",
            "1440x900",
            "1280x720",
            "1600x900",
            "2560x1440",
        ];

        let timezones = [
            "America/Asuncion",
            "America/Sao_Paulo",
            "America/Buenos_Aires",
            "America/Mexico_City",
            "America/Bogota",
            "America/Lima",
            "America/Santiago",
            "America/Caracas",
        ];

        let langs = ["es-PY", "es-AR", "es-ES", "pt-BR", "en-US", "es-MX"];

        crate::identities::types::IdentityFingerprint {
            user_agent: ua,
            screen_resolution: resolutions[rng.gen_range(0..resolutions.len())].to_string(),
            timezone: timezones[rng.gen_range(0..timezones.len())].to_string(),
            language: langs[rng.gen_range(0..langs.len())].to_string(),
            platform: if rng.gen_bool(0.6) {
                "Linux x86_64".to_string()
            } else {
                "Win64".to_string()
            },
            browser_profile_dir: None,
        }
    }

    /// Genera identidad de respaldo (offline) cuando Mistral no está disponible
    pub fn generate_offline(&self, count: usize) -> Vec<SyntheticIdentity> {
        let fallback_names = vec![
            (
                "Carlos Mendoza",
                "M",
                34,
                "Ingeniero de Sistemas",
                "Asunción",
            ),
            ("Laura Giménez", "F", 28, "Abogada", "Ciudad del Este"),
            ("Pedro Benítez", "M", 45, "Contador Público", "Encarnación"),
            (
                "Sofía Duarte",
                "F",
                31,
                "Docente Universitaria",
                "San Lorenzo",
            ),
            ("Federico Ozuna", "M", 39, "Comerciante", "Luque"),
            ("Valentina Rojas", "F", 26, "Periodista", "Asunción"),
            ("Héctor Acosta", "M", 52, "Médico", "Ciudad del Este"),
            ("Camila Núñez", "F", 33, "Diseñadora Gráfica", "Lambaré"),
            ("Gustavo Ferreira", "M", 41, "Arquitecto", "Capiatá"),
            (
                "María José López",
                "F",
                29,
                "Administradora de Empresas",
                "Fernando de la Mora",
            ),
        ];

        fallback_names
            .into_iter()
            .take(count)
            .map(|(name, gender, age, occ, city)| {
                let profile = IdentityProfile {
                    full_name: name.to_string(),
                    gender: gender.to_string(),
                    age,
                    nationality: "Paraguaya".to_string(),
                    occupation: occ.to_string(),
                    city: city.to_string(),
                    country: "Paraguay".to_string(),
                    bio: format!(
                        "{} es {} con experiencia en {}. Reside actualmente en {}.",
                        name,
                        occ.to_lowercase(),
                        occ.to_lowercase(),
                        city
                    ),
                    traits: vec![
                        "analítico".to_string(),
                        "comunicativo".to_string(),
                        "resolutivo".to_string(),
                    ],
                    interests: vec![
                        "tecnología".to_string(),
                        "lectura".to_string(),
                        "viajes".to_string(),
                    ],
                };
                let mut identity = SyntheticIdentity::new(profile);
                identity.fingerprint = self.generate_fingerprint(&identity);
                identity
            })
            .collect()
    }
}

/// Extrae JSON del texto (maneja markdown fences y basura)
fn extract_json(text: &str) -> Result<String> {
    let text = text.trim();
    // Quitar ```json ... ``` si existe
    if text.starts_with("```") {
        let start = text.find('\n').unwrap_or(0) + 1;
        let end = text.rfind("```").unwrap_or(text.len());
        return Ok(text[start..end].trim().to_string());
    }
    // Intentar parsear directamente
    serde_json::from_str::<Value>(text).map_err(|_| anyhow!("No valid JSON found in response"))?;
    Ok(text.to_string())
}
