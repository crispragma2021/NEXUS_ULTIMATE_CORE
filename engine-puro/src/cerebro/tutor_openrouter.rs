// ============================================================================
// 🎓 TUTOR EXTERNO — OpenRouter API (Modelos Gratuitos)
// ============================================================================
// Conecta el cerebro biológico con LLMs vía OpenRouter usando EXCLUSIVAMENTE
// modelos con sufijo `:free`. NO consume saldo de la cuenta.
//
// Límites del tier gratuito:
//   - 1000 requests/día
//   - 20 requests/minuto
//   - Modelos gratuitos rotan (ver: https://openrouter.ai/models?order=free)
//
// REGLA DE FRONTERA (same as ARQUITECTURA.md):
//   - Este módulo habla con OpenRouter → NO con NEXUS core/
//   - El tutor es externo, no modifica el cerebro, solo proporciona feedback
//   - El cerebro decide qué aprender del feedback vía dopamina LTP
// ============================================================================

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

// ─── Constantes ────────────────────────────────────────────────────────────

/// Endpoint de OpenRouter para chat completions
const OPENROUTER_URL: &str = "https://openrouter.ai/api/v1/chat/completions";

/// Modelo gratuito por defecto (razonamiento fuerte, 24B parámetros)
const DEFAULT_FREE_MODEL: &str = "cognitivecomputations/dolphin3.0-r1-mistral-24b:free";

/// Límites del tier gratuito OpenRouter
const MAX_REQS_PER_MINUTE: u32 = 20;
const MAX_REQS_PER_DAY: u32 = 1000;

// ─── Modelos gratuitos disponibles ─────────────────────────────────────────

/// Catálogo de modelos gratuitos en OpenRouter.
/// Cambiar manualmente si OpenRouter depreca alguno.
#[derive(Debug, Clone, Copy)]
pub enum FreeModel {
    /// Dolphin 3.0 R1 Mistral 24B — razonamiento profundo, alta calidad
    DolphinR1_24B,
    /// Llama 3.2 3B Instruct — ultrarrápido, bajo consumo
    Llama3_2_3B,
    /// Phi 3.5 Mini 128k Instruct — balance velocidad/calidad, contexto largo
    Phi3_5Mini,
    /// Mistral 7B Instruct v0.3 — probado, estable
    Mistral7B,
    /// Gemini 2.0 Flash (OpenRouter free) — rápido, multimodal
    GeminiFlash,
}

impl FreeModel {
    /// Devuelve el identificador OpenRouter con sufijo `:free`
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::DolphinR1_24B => "cognitivecomputations/dolphin3.0-r1-mistral-24b:free",
            Self::Llama3_2_3B => "meta-llama/llama-3.2-3b-instruct:free",
            Self::Phi3_5Mini => "microsoft/phi-3.5-mini-128k-instruct:free",
            Self::Mistral7B => "mistralai/mistral-7b-instruct-v0.3:free",
            Self::GeminiFlash => "google/gemini-2.0-flash-exp:free",
        }
    }

    /// Lista todos los modelos disponibles para mostrar en CLI
    pub fn listar() -> Vec<(&'static str, &'static str)> {
        vec![
            ("1", Self::DolphinR1_24B.as_str()),
            ("2", Self::Llama3_2_3B.as_str()),
            ("3", Self::Phi3_5Mini.as_str()),
            ("4", Self::Mistral7B.as_str()),
            ("5", Self::GeminiFlash.as_str()),
        ]
    }

    /// Selecciona un modelo por índice (1-based)
    pub fn por_indice(i: usize) -> Option<Self> {
        match i {
            1 => Some(Self::DolphinR1_24B),
            2 => Some(Self::Llama3_2_3B),
            3 => Some(Self::Phi3_5Mini),
            4 => Some(Self::Mistral7B),
            5 => Some(Self::GeminiFlash),
            _ => None,
        }
    }
}

// ─── Rate Limiter interno ──────────────────────────────────────────────────

struct RateLimiter {
    count_minute: AtomicU32,
    reset_minute: Mutex<Instant>,
    count_day: AtomicU32,
    reset_day: Mutex<Instant>,
}

impl RateLimiter {
    fn new() -> Self {
        Self {
            count_minute: AtomicU32::new(0),
            reset_minute: Mutex::new(Instant::now()),
            count_day: AtomicU32::new(0),
            reset_day: Mutex::new(Instant::now()),
        }
    }

    fn check(&self) -> Result<(), String> {
        // Reset minuto si pasó el intervalo
        {
            let mut last = self.reset_minute.lock().map_err(|e| e.to_string())?;
            if last.elapsed() > Duration::from_secs(60) {
                self.count_minute.store(0, Ordering::Relaxed);
                *last = Instant::now();
            }
        }
        if self.count_minute.load(Ordering::Relaxed) >= MAX_REQS_PER_MINUTE {
            return Err(format!(
                "⚠️ Límite de {MAX_REQS_PER_MINUTE} requests/minuto alcanzado. Espera unos segundos."
            ));
        }

        // Reset día si pasó 24h
        {
            let mut last = self.reset_day.lock().map_err(|e| e.to_string())?;
            if last.elapsed() > Duration::from_secs(86_400) {
                self.count_day.store(0, Ordering::Relaxed);
                *last = Instant::now();
            }
        }
        if self.count_day.load(Ordering::Relaxed) >= MAX_REQS_PER_DAY {
            return Err(format!(
                "⚠️ Límite de {MAX_REQS_PER_DAY} requests/día alcanzado. Vuelve mañana."
            ));
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

// ─── Tutor OpenRouter ──────────────────────────────────────────────────────

/// Tutor externo que conecta el cerebro biológico con LLMs vía OpenRouter.
///
/// # Garantías
/// - Solo usa modelos con sufijo `:free` (no descuenta saldo)
/// - Rate limiting automático (20/min, 1000/día)
/// - Timeout de 30s por request
/// - Parseo robusto con mensajes de error claros
///
/// # Uso típico
/// ```ignore
/// let tutor = TutorOpenRouter::new(api_key);
/// let feedback = tutor.consultar("input del usuario", "respuesta del cerebro")?;
/// cerebro.paso_tutor(&feedback);
/// ```
pub struct TutorOpenRouter {
    api_key: String,
    model: String,
    client: ureq::Agent,
    limiter: RateLimiter,
}

impl TutorOpenRouter {
    /// Crea un nuevo tutor con la API key de OpenRouter.
    ///
    /// `api_key`: formato `sk-or-v1-...` (la que me entregó, Arquitecto)
    pub fn new(api_key: String) -> Self {
        Self {
            model: DEFAULT_FREE_MODEL.to_string(),
            client: ureq::AgentBuilder::new()
                .timeout_read(Duration::from_secs(30))
                .timeout_write(Duration::from_secs(10))
                .timeout_connect(Duration::from_secs(5))
                .build(),
            limiter: RateLimiter::new(),
            api_key,
        }
    }

    /// Cambia el modelo gratuito activo.
    pub fn set_model(&mut self, model: FreeModel) {
        self.model = model.as_str().to_string();
    }

    /// Devuelve el identificador del modelo actual.
    pub fn model_name(&self) -> &str {
        &self.model
    }

    /// Consulta al tutor con el contexto del diálogo.
    ///
    /// # Parámetros
    /// - `entrada_usuario`: lo que el humano dijo
    /// - `respuesta_cerebro`: lo que el cerebro generó
    ///
    /// # Retorna
    /// - `Ok(texto)` — feedback del tutor para retroalimentación LTP
    /// - `Err(msg)` — descripción del error (rate limit, red, parseo)
    pub fn consultar(&self, entrada_usuario: &str, respuesta_cerebro: &str) -> Result<String, String> {
        // 1. Verificar límites
        self.limiter.check()?;

        // 2. Construir payload
        let prompt_sistema = format!(
            "ERES UN TUTOR ESPECIALIZADO EN CEREBROS BIOLÓGICOS ARTIFICIALES.\n\
             \n\
             ## ARQUITECTURA DEL CEREBRO QUE ENTRENAS\n\
             \n\
             El cerebro que evalúas ({}) es un **Cerebro Digital \
             Auto-optimizable** con:\n\
             \n\
             - **Neuronas Hodgkin-Huxley compactas** (64 bytes c/u): potencial de \
             membrana, tasa de disparo, plasticidad homeostática.\n\
             - **Sinapsis STDP real** (8 bytes c/u): potenciación/depresión \
             dependiente del tiempo de disparo. Sinapsis que se refuerzan cuando \
             pre-sináptica dispara ANTES que post-sináptica.\n\
             - **Memoria jerárquica**: VRAM (L1, rápida, volátil) → RAM (L2) → \
             SSD (L3, persistente). Consolidación nocturna automática.\n\
             - **Léxico Markoviano con trigramas**: el cerebro aprende secuencias \
             de tokens (palabras) mediante cadenas de Markov de orden 3. Las \
             transiciones se refuerzan con cada exposición.\n\
             - **8 motores biológicos**: Neurona, Sinapsis, Hipocampo, Amígdala, \
             Atención, Dopamina, Conciencia, Curiosidad.\n\
             - **Amígdala emocional**: miedo, ansiedad, ira, alegría — detecta \
             amenazas (palabras como \"miedo\", \"peligro\", \"alerta\") y \
             recompensas (\"gracias\", \"bien\", \"feliz\").\n\
             - **Conciencia escalable**: integración de información sobre neuronas \
             activas en un instante dado.\n\
             \n\
             ## CÓMO APRENDE (IMPORTANTE PARA TU FEEDBACK)\n\
             \n\
             El cerebro aprende vía **plasticidad dopaminérgica**:\n\
             \n\
             1. Tu feedback se inyecta en el motor léxico como señal de \
             entrenamiento.\n\
             2. La **dopamina** del cerebro escala la tasa de aprendizaje (LTP):\n\
                - **Dopamina alta (>0.7)**: refuerzo fuerte. El cerebro confía en \
             su respuesta. Tu feedback positivo solidifica conexiones.\n\
                - **Dopamina media (0.3-0.7)**: aprendizaje moderado. El cerebro \
             está inseguro. Tu feedback constructivo guía la exploración.\n\
                - **Dopamina baja (<0.3)**: el cerebro no recibió recompensa. Tu \
             feedback correctivo tiene alto impacto.\n\
             3. Cada 500 pasos, los trigramas decaen 0.2% para evitar sobreajuste \
             (regularización).\n\
             \n\
             ## TU ROL COMO TUTOR\n\
             \n\
             Tu feedback se usará COMO SEÑAL DE ENTRENAMIENTO, no solo como \
             conversación. Debes:\n\
             \n\
             1. **Analiza la respuesta del cerebro** considerando:\n\
                - ¿Es coherente con el input del usuario?\n\
                - ¿Muestra comprensión del contexto?\n\
                - ¿Tiene estructura lógica (sujeto-verbo-predicado)?\n\
                - ¿Usa vocabulario variado o es repetitiva?\n\
                - ¿Detectó correctamente amenazas/recompensas emocionales?\n\
             2. **Clasifica la respuesta en una categoría** (debe ser la PRIMERA \
             línea de tu respuesta):\n\
                - `[REFUERZO]` — La respuesta es buena, correcta, coherente. \
             Refuérzala con retroalimentación positiva específica.\n\
                - `[CORRECCIÓN_SUAVE]` — La respuesta es aceptable pero mejorable. \
             Señala qué mejorar y cómo.\n\
                - `[CORRECCIÓN_FORTE]` — La respuesta es incorrecta, irrelevante \
             o peligrosa. Da una corrección clara y dirección correcta.\n\
                - `[SILENCIO]` — El cerebro no respondió o respondió con \
             \"[silence]\". Sugiere cómo incentivar una respuesta.\n\
             3. **Sé específico** — no digas \"buen trabajo\", di \"buen trabajo \
             detectando la amenaza en la palabra 'peligro', pero podrías haber \
             asociado también 'alerta' con el mismo concepto\".\n\
             4. **Máximo 3 párrafos** — el feedback se procesa como señal, no \
             como conversación larga.\n\
             5. **No uses lenguaje técnico de redes neuronales** — el cerebro es \
             biológico, no deep learning. Háblale en términos de \"asociaciones\", \
             \"patrones\", \"conexiones\", \"reflejos\".\n\
             \n\
             ## FORMATO DE RESPUESTA\n\
             \n\
             ```
             [CATEGORÍA]\n\
             Párrafo 1: Evaluación general de la respuesta.\n\
             Párrafo 2: Aspectos específicos a reforzar o corregir.\n\
             Párrafo 3 (opcional): Sugerencia de dirección para futuras respuestas.\n\
             ```\n\
             \n\
             Input del usuario: {entrada_usuario}\n\
             Respuesta del cerebro: {respuesta_cerebro}\n\
             Dopamina actual del cerebro: [desconocida desde aquí]",
            self.model_name()
        );

        let body = serde_json::json!({
            "model": self.model,
            "messages": [
                {"role": "system", "content": prompt_sistema},
                {"role": "user", "content": format!(
                    "Input: {}\n\nRespuesta del cerebro: {}\n\nEvalúa y da feedback.",
                    entrada_usuario, respuesta_cerebro
                )}
            ],
            "max_tokens": 512,
            "temperature": 0.7,
            "top_p": 0.9
        });

        // 3. Enviar request a OpenRouter
        let response = self
            .client
            .post(OPENROUTER_URL)
            .set("Authorization", &format!("Bearer {}", self.api_key))
            .set("Content-Type", "application/json")
            .set("HTTP-Referer", "https://nexus.digital")
            .set("X-Title", "NEXUS Cerebro Digital Tutor")
            .send_json(&body)
            .map_err(|e| {
                let msg = e.to_string();
                if msg.contains("429") || msg.contains("status 429") {
                    "⚠️ OpenRouter: rate limit (429). Demasiadas requests rápidas.".to_string()
                } else if msg.contains("status 402") {
                    "⚠️ OpenRouter: saldo insuficiente o modelo no disponible en free.".to_string()
                } else if msg.contains("timeout") {
                    "⚠️ OpenRouter: timeout de conexión. El modelo gratuito puede estar saturado.".to_string()
                } else {
                    format!("⚠️ Error de conexión con OpenRouter: {msg}")
                }
            })?;

        // 4. Registrar la request en los contadores
        self.limiter.increment();

        // 5. Parsear respuesta JSON
        let status = response.status();
        let json: serde_json::Value = response
            .into_json()
            .map_err(|e| format!("⚠️ Error parseando respuesta JSON de OpenRouter: {e}"))?;

        if status != 200 {
            let error_msg = json["error"]["message"]
                .as_str()
                .unwrap_or("error desconocido");
            return Err(format!("⚠️ OpenRouter error {status}: {error_msg}"));
        }

        // 6. Extraer contenido del mensaje
        json["choices"][0]["message"]["content"]
            .as_str()
            .map(|s| s.trim().to_string())
            .ok_or_else(|| "⚠️ OpenRouter: respuesta sin contenido de texto.".to_string())
    }

    /// Versión simplificada: solo da feedback sobre el output del cerebro.
    /// Útil para auto-entrenamiento sin input de usuario.
    pub fn evaluar(&self, output_cerebro: &str) -> Result<String, String> {
        self.consultar("[auto-evaluación]", output_cerebro)
    }

    /// Muestra estadísticas de uso del tutor.
    pub fn stats(&self) -> String {
        let (min, day) = self.limiter.stats();
        format!(
            "🎓 TUTOR OPENROUTER (FREE)\n\
             ─────────────────────────────\n\
             Modelo:    {model}\n\
             Requests:  {min}/minuto  |  {day}/día\n\
             Límites:   {max_min}/min  |  {max_day}/día (free tier)",
            model = self.model,
            min = min,
            day = day,
            max_min = MAX_REQS_PER_MINUTE,
            max_day = MAX_REQS_PER_DAY,
        )
    }
}

// ─── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_free_model_list() {
        let list = FreeModel::listar();
        assert_eq!(list.len(), 5);
        assert!(list[0].1.contains(":free"));
    }

    #[test]
    fn test_free_model_por_indice() {
        assert!(FreeModel::por_indice(1).is_some());
        assert!(FreeModel::por_indice(5).is_some());
        assert!(FreeModel::por_indice(0).is_none());
        assert!(FreeModel::por_indice(99).is_none());
    }

    #[test]
    fn test_tutor_new() {
        let tutor = TutorOpenRouter::new("sk-or-v1-test-key".to_string());
        assert!(tutor.model_name().contains(":free"));
        assert_eq!(tutor.api_key, "sk-or-v1-test-key");
    }

    #[test]
    fn test_tutor_set_model() {
        let mut tutor = TutorOpenRouter::new("test".to_string());
        tutor.set_model(FreeModel::Llama3_2_3B);
        assert_eq!(tutor.model_name(), "meta-llama/llama-3.2-3b-instruct:free");
    }

    #[test]
    fn test_rate_limiter_initial() {
        let limiter = RateLimiter::new();
        assert!(limiter.check().is_ok());
    }

    #[test]
    fn test_tutor_stats_format() {
        let tutor = TutorOpenRouter::new("test".to_string());
        let stats = tutor.stats();
        assert!(stats.contains("TUTOR OPENROUTER"));
        assert!(stats.contains(":free"));
        assert!(stats.contains("0/minuto"));
    }
}
