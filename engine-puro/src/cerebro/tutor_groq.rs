// ============================================================================
// 🎓 TUTOR GROQ — LLM Tutor de Alta Inteligencia vía API Groq (OpenAI-compatible)
// ============================================================================
// Usa la API de Groq (api.groq.com) con modelos grandes y gratuitos como
// Llama 3.3 70B Versatile — razonamiento profundo, rápido y sin costo.
//
// Este tutor actúa como GUÍA y DESTILADOR en el puente de aprendizaje
// autónomo del cerebro: propone qué investigar y destila la web en lecciones.
//
// Implementación: HTTP síncrono con ureq (0 overhead async, misma lib que el
// tutor OpenRouter). Sin dependencias nuevas.
// ============================================================================

use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, Instant};

/// Endpoint compatible con OpenAI para chat (Groq)
const GROQ_URL: &str = "https://api.groq.com/openai/v1/chat/completions";

/// Modelo grande e inteligente por defecto (Llama 3.3 70B — gratuito en Groq)
const DEFAULT_GROQ_MODEL: &str = "llama-3.3-70b-versatile";

/// Límites de Groq (nivel gratuito): 30 RPM, 14400 RPD, 6000 tokens por minuto
const MAX_REQUESTS_PER_MINUTE: u32 = 30;
const MAX_REQUESTS_PER_DAY: u32 = 14_400;

// ─── Rate Limiter interno ─────────────────────────────────────────────────

struct RateLimiter {
    count_minute: AtomicU32,
    reset_minute: Mutex<Instant>,
    count_day: AtomicU32,
}

// Necesitamos Mutex de std
use std::sync::Mutex;

impl RateLimiter {
    fn new() -> Self {
        Self {
            count_minute: AtomicU32::new(0),
            reset_minute: Mutex::new(Instant::now()),
            count_day: AtomicU32::new(0),
        }
    }

    fn check(&self) -> Result<(), String> {
        // Ventana por minuto
        let mut last_reset = self.reset_minute.lock().map_err(|_| "lock poisoned")?;
        if last_reset.elapsed() >= Duration::from_secs(60) {
            self.count_minute.store(0, Ordering::Relaxed);
            *last_reset = Instant::now();
        }
        drop(last_reset);

        if self.count_minute.load(Ordering::Relaxed) >= MAX_REQUESTS_PER_MINUTE {
            return Err("⚠️ Groq: rate limit por minuto alcanzado. Espera 60s.".to_string());
        }
        if self.count_day.load(Ordering::Relaxed) >= MAX_REQUESTS_PER_DAY {
            return Err("⚠️ Groq: límite diario alcanzado (14400 req).".to_string());
        }
        Ok(())
    }

    fn increment(&self) {
        self.count_minute.fetch_add(1, Ordering::Relaxed);
        self.count_day.fetch_add(1, Ordering::Relaxed);
    }

    fn stats(&self) -> (u32, u32) {
        (
            self.count_minute.load(Ordering::Relaxed),
            self.count_day.load(Ordering::Relaxed),
        )
    }
}

// ─── Tutor Groq ────────────────────────────────────────────────────────────

pub struct TutorGroq {
    api_key: String,
    model: String,
    client: ureq::Agent,
    limiter: RateLimiter,
}

impl TutorGroq {
    /// Crea un nuevo tutor Groq con la API key (formato `gsk_...`).
    pub fn new(api_key: String) -> Self {
        Self {
            model: DEFAULT_GROQ_MODEL.to_string(),
            client: ureq::AgentBuilder::new()
                .timeout_read(Duration::from_secs(45))
                .timeout_write(Duration::from_secs(15))
                .timeout_connect(Duration::from_secs(10))
                .build(),
            limiter: RateLimiter::new(),
            api_key,
        }
    }

    /// Cambia el modelo activo (acepta cualquier id del catálogo Groq).
    pub fn set_model(&mut self, model: &str) {
        self.model = model.to_string();
    }

    /// Devuelve el identificador del modelo actual.
    pub fn model_name(&self) -> &str {
        &self.model
    }

    /// Consulta al tutor con un prompt de usuario y contexto.
    ///
    /// # Parámetros
    /// - `entrada_usuario`: contexto o material de entrada (síntesis web, etc.)
    /// - `instruccion`: la tarea/prompt del sistema (guiar, destilar, etc.)
    ///
    /// # Retorna
    /// - `Ok(texto)` — respuesta del modelo
    /// - `Err(msg)` — descripción del error (rate limit, red, parseo)
    pub fn consultar(&self, entrada_usuario: &str, instruccion: &str) -> Result<String, String> {
        // 1. Verificar límites
        self.limiter.check()?;

        // 2. Construir payload (OpenAI-compatible)
        let body = serde_json::json!({
            "model": self.model,
            "messages": [
                {"role": "system", "content": instruccion},
                {"role": "user", "content": entrada_usuario}
            ],
            "max_tokens": 700,
            "temperature": 0.7
        });

        // 3. Enviar request a Groq
        let response = self
            .client
            .post(GROQ_URL)
            .set("Authorization", &format!("Bearer {}", self.api_key))
            .set("Content-Type", "application/json")
            .send_json(&body)
            .map_err(|e| {
                let msg = e.to_string();
                if msg.contains("429") || msg.contains("status 429") {
                    "⚠️ Groq: rate limit (429). Espera 60s.".to_string()
                } else if msg.contains("401") || msg.contains("status 401") {
                    "⚠️ Groq: API key inválida (401).".to_string()
                } else if msg.contains("timeout") {
                    "⚠️ Groq: timeout de conexión.".to_string()
                } else {
                    format!("⚠️ Error de conexión con Groq: {msg}")
                }
            })?;

        // 4. Registrar la request
        self.limiter.increment();

        // 5. Parsear respuesta JSON
        let status = response.status();
        let json: serde_json::Value = response
            .into_json()
            .map_err(|e| format!("⚠️ Error parseando respuesta JSON de Groq: {e}"))?;

        if status != 200 {
            let error_msg = json["error"]["message"]
                .as_str()
                .unwrap_or("error desconocido");
            return Err(format!("⚠️ Groq error {status}: {error_msg}"));
        }

        // 6. Extraer contenido del mensaje
        json["choices"][0]["message"]["content"]
            .as_str()
            .map(|s| s.trim().to_string())
            .ok_or_else(|| "⚠️ Groq: respuesta sin contenido de texto.".to_string())
    }

    /// Muestra estadísticas de uso del tutor.
    pub fn stats(&self) -> String {
        let (min, day) = self.limiter.stats();
        format!(
            "📊 Tutor Groq ({}): {} req/minuto | {} req hoy",
            self.model, min, day
        )
    }
}

// ============================================================================
// TESTS
// ============================================================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tutor_groq_nuevo_modelo_default() {
        let t = TutorGroq::new("gsk_test".to_string());
        assert!(t.model_name().contains("llama"));
        assert!(t.model_name().contains("70b"));
    }

    #[test]
    fn test_set_model_cambia_modelo() {
        let mut t = TutorGroq::new("gsk_test".to_string());
        t.set_model("openai/gpt-oss-120b");
        assert_eq!(t.model_name(), "openai/gpt-oss-120b");
    }

    #[test]
    fn test_rate_limiter_inicial() {
        let l = RateLimiter::new();
        let (min, day) = l.stats();
        assert_eq!(min, 0);
        assert_eq!(day, 0);
    }

    #[test]
    fn test_rate_limiter_check_ok() {
        let l = RateLimiter::new();
        assert!(l.check().is_ok());
        l.increment();
        let (min, _) = l.stats();
        assert_eq!(min, 1);
    }

    #[test]
    fn test_stats_formato() {
        let t = TutorGroq::new("gsk_test".to_string());
        let s = t.stats();
        assert!(s.contains("Tutor Groq"));
        assert!(s.contains("req/minuto"));
    }
}
