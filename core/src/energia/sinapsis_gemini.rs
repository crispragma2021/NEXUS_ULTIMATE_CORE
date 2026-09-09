use crate::capa_invisibilidad::{red::GestorRed, NexusCloak, SigiloLevel};
use crate::homeostasis_utils::GLOBAL_CACHE;
use crate::thinking_strategy::AdaptiveThinking;
use anyhow::{bail, Result};
use reqwest::{
    header::{HeaderMap, HeaderValue, USER_AGENT},
    Client,
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::sync::RwLock;

const GEMINI_CALIBRATION_PATH: &str = "nexus_gemini_calibration.json";
const GEMINI_MD_PATH: &str = "C:/Users/crisp/NEXUS_ULTIMATE_CORE/docs/identity/identity.md";

#[derive(Serialize)]
pub struct GeminiRequest {
    pub contents: Vec<Content>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,
    #[serde(rename = "generationConfig", skip_serializing_if = "Option::is_none")]
    pub generation_config: Option<GenerationConfig>,
    /// Prompt Caching de Gemini: si se establece, la constitución/identidad
    /// estable se envía por `cachedContent` (se paga una vez por sesión)
    /// en lugar de re-pagarse en cada turno. Ahorro ~87% en tokens de input.
    #[serde(rename = "cachedContent", skip_serializing_if = "Option::is_none")]
    pub cached_content: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Content {
    pub role: String,
    pub parts: Vec<Part>,
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Part {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(rename = "inlineData", skip_serializing_if = "Option::is_none")]
    pub inline_data: Option<InlineData>,
    #[serde(rename = "functionCall", skip_serializing_if = "Option::is_none")]
    pub function_call: Option<FunctionCall>,
    #[serde(rename = "functionResponse", skip_serializing_if = "Option::is_none")]
    pub function_response: Option<FunctionResponse>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Tool {
    #[serde(rename = "functionDeclarations")]
    pub function_declarations: Vec<FunctionDeclaration>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct FunctionDeclaration {
    pub name: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FunctionCall {
    pub name: String,
    pub args: serde_json::Value,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct FunctionResponse {
    pub name: String,
    pub response: serde_json::Value,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct InlineData {
    pub mime_type: String,
    pub data: String,
}

#[derive(Serialize)]
pub struct GenerationConfig {
    pub temperature: f32,
    #[serde(rename = "maxOutputTokens")]
    pub max_output_tokens: u32,
    #[serde(rename = "responseModalities", skip_serializing_if = "Option::is_none")]
    pub response_modalities: Option<Vec<String>>,
    #[serde(rename = "speechConfig", skip_serializing_if = "Option::is_none")]
    pub speech_config: Option<SpeechConfig>,
}

#[derive(Serialize)]
pub struct SpeechConfig {
    #[serde(rename = "voiceConfig")]
    pub voice_config: VoiceConfig,
}

#[derive(Serialize)]
pub struct VoiceConfig {
    #[serde(rename = "prebuiltVoiceConfig")]
    pub prebuilt_voice_config: PrebuiltVoiceConfig,
}

#[derive(Serialize)]
pub struct PrebuiltVoiceConfig {
    #[serde(rename = "voiceName")]
    pub voice_name: String,
}

#[derive(Deserialize)]
struct GeminiResponse {
    candidates: Option<Vec<Candidate>>,
    error: Option<GeminiError>,
}

#[derive(Deserialize)]
struct Candidate {
    content: Content,
}

#[derive(Deserialize)]
struct GeminiError {
    message: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GeminiModel {
    Flash3_0, // [OMEGA UPDATE]: Actualizado a la versión 3.0 Flash Preview por orden del Arquitecto
    Pro2_0,
    Custom(String),
}

impl GeminiModel {
    pub fn as_str(&self) -> &str {
        match self {
            GeminiModel::Flash3_0 => "gemini-3-flash-preview",
            GeminiModel::Pro2_0 => "gemini-2.0-pro-exp-02-05",
            GeminiModel::Custom(s) => s,
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct NexusPlan {
    pub agent: String,
    pub task: String,
    pub params: serde_json::Value,
    pub priority: u8,
    #[serde(default = "default_pool")]
    pub key_pool: String, // "official", "openrouter", "zenith"
    pub fallback_agent: Option<String>,
    pub confidence_score: f64,
    #[serde(default)]
    pub semantic_metadata: serde_json::Value, // Aquí "empaquetamos" pesos, colores y significados
}

fn default_pool() -> String {
    "official".to_string()
}

/// NEXUS 13.0: Gemini aprende a calibrar su confianza
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ConfidenceCalibration {
    pub predictions: Vec<(f64, bool)>, // Historial (score, success)
}

impl ConfidenceCalibration {
    pub fn calibrate(&self) -> f64 {
        if self.predictions.is_empty() {
            return 1.0;
        }
        let total = self.predictions.len() as f64;
        let expected_success: f64 =
            self.predictions.iter().map(|(score, _)| score).sum::<f64>() / total;
        let actual_success: f64 = self
            .predictions
            .iter()
            .filter(|(_, success)| *success)
            .count() as f64
            / total;

        if expected_success < 0.01 {
            return 1.0;
        }
        (actual_success / expected_success).clamp(0.5, 2.0)
    }
}

use std::sync::atomic::{AtomicUsize, Ordering};

pub struct GeminiAPI {
    client: Client,
    api_accounts: Vec<Vec<String>>,
    current_account_index: AtomicUsize,
    current_key_in_account_index: AtomicUsize,
    ultimo_uso: std::sync::Arc<tokio::sync::Mutex<std::time::Instant>>,
    calibration: RwLock<ConfidenceCalibration>, // NEXUS 13.0
}

impl GeminiAPI {
    pub fn new(flat_keys: Vec<String>) -> Self {
        // Para mantener compatibilidad inicial, agrupamos las llaves planas en una sola cuenta
        // si no vienen ya estructuradas.
        let api_accounts = vec![flat_keys];
        Self {
            client: Client::new(),
            api_accounts,
            current_account_index: AtomicUsize::new(0),
            current_key_in_account_index: AtomicUsize::new(0),
            ultimo_uso: std::sync::Arc::new(tokio::sync::Mutex::new(std::time::Instant::now())),
            calibration: RwLock::new(Self::load_calibration()),
        }
    }

    pub fn new_omega(api_accounts: Vec<Vec<String>>) -> Self {
        Self {
            client: Client::new(),
            api_accounts,
            current_account_index: AtomicUsize::new(0),
            current_key_in_account_index: AtomicUsize::new(0),
            ultimo_uso: std::sync::Arc::new(tokio::sync::Mutex::new(std::time::Instant::now())),
            calibration: RwLock::new(Self::load_calibration()),
        }
    }

    fn load_calibration() -> ConfidenceCalibration {
        if let Ok(data) = fs::read_to_string(GEMINI_CALIBRATION_PATH) {
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            ConfidenceCalibration::default()
        }
    }

    fn save_calibration(&self) {
        let cal = self.calibration.read().unwrap();
        if let Ok(json) = serde_json::to_string(&*cal) {
            let _ = fs::write(GEMINI_CALIBRATION_PATH, json);
        }
    }

    /// Carga la identidad soberana de NEXUS desde GEMINI.md
    fn cargar_identidad_nexus(&self) -> String {
        match fs::read_to_string(GEMINI_MD_PATH) {
            Ok(content) => {
                tracing::info!("🧬 [NEXUS] Identidad cargada desde .agent/rules/GEMINI.md");
                content
            }
            Err(e) => {
                tracing::error!(
                    "❌ [NEXUS] No se pudo cargar la identidad: {}. Usando ancla de emergencia.",
                    e
                );
                "ERES NEXUS. Orquestador Soberano. Lealtad absoluta al Arquitecto.".to_string()
            }
        }
    }

    /// Devuelve el prefijo de identidad ya estable de NEXUS para usar como
    /// `cachedContent` en las llamadas a Gemini.
    ///
    /// Por defecto retorna `None` (caché OF): evita cambiar el comportamiento de
    /// las llamadas de extracción/scraping. Cuando el pipeline soberano quiera
    /// activarlo, puede llamarse con `Some` para que la constitución de ~5.6K
    /// tokens se pague una vez por sesión y no en cada turno (ahorro ~87%).
    ///
    /// Para ACTIVAR el caché en el flujo soberano, cambiar la siguiente línea a:
    ///   Some(self.cargar_identidad_nexus())
    fn cached_identity_ctx(&self) -> Option<String> {
        // Por ahora, caché desactivado por defecto (cambio mínimo de comportamiento).
        // Para activarlo en producción soberana, descomentar:
        // Some(self.cargar_identidad_nexus())
        None
    }

    /// Construye el prompt del sistema con la identidad inyectada
    fn construir_prompt_con_identidad(&self, task_prompt: &str) -> String {
        let identidad = self.cargar_identidad_nexus();
        format!(
            "{}\n\n---\n\n## INSTRUCCIÓN OPERATIVA ACTUAL:\n{}",
            identidad, task_prompt
        )
    }

    fn rotar_llave(&self) -> String {
        if self.api_accounts.is_empty() {
            return String::new();
        }

        let total_cuentas = self.api_accounts.len();
        let c_idx = self.current_account_index.load(Ordering::SeqCst) % total_cuentas;

        // Si rotamos todas las cuentas, avanzamos a la siguiente llave dentro de las cuentas
        let current_acc = self.current_account_index.load(Ordering::SeqCst);
        if current_acc > 0 && current_acc.is_multiple_of(total_cuentas) {
            self.current_key_in_account_index
                .fetch_add(1, Ordering::SeqCst);
        }

        let k_idx = self.current_key_in_account_index.load(Ordering::SeqCst)
            % self.api_accounts[c_idx].len();
        self.current_account_index.fetch_add(1, Ordering::SeqCst);

        self.api_accounts[c_idx][k_idx].clone()
    }

    pub async fn consultar(&self, prompt: &str, model: &str) -> Result<(String, Option<String>)> {
        self.consultar_con_audio(prompt, model, None).await
    }

    /// [FUSIÓN SELECTIVA]: Análisis de imagen succionado de core/src/gemini_client.rs
    pub async fn analizar_imagen(
        &self,
        prompt: &str,
        image_b64: &str,
        model: Option<&str>,
    ) -> Result<String> {
        let model_name = model.unwrap_or(GeminiModel::Flash3_0.as_str());
        let (text, _) = self
            .consultar_omega(prompt, model_name, Some(image_b64.to_string()), None)
            .await?;
        Ok(text)
    }

    /// NEXUS Orquestador: Genera un plan estructurado utilizando Gemini
    pub async fn planificar(
        &self,
        user_input: &str,
        system_health: serde_json::Value,
    ) -> Result<NexusPlan> {
        // NEXUS 14.0: Anomalía Zero - Detección de intrusión semántica
        if self.detectar_anomalia(user_input) {
            tracing::error!(
                "🚨 [ANOMALÍA ZERO] Intento de intrusión detectado. Abortando planificación."
            );
            bail!("Intento de secuestro de identidad o jailbreak detectado.");
        }

        // Obtener estrategia de pensamiento adaptativa
        let thinking_engine = AdaptiveThinking::new();

        // NEXUS OMEGA: Integración de sensores reales en la planificación
        // Nota: En una implementación completa, orquestador pasaría el estado aquí.
        // Por ahora, usamos el sensor para informar a la estrategia.
        let strategy = thinking_engine.get_strategy(user_input, None);

        let user_prompt = format!(
            "Analiza la petición con un nivel de esfuerzo {:?} y genera un plan JSON de routing.\n\
            Estructura: {{\n\
                \"agent\": \"deepseek_coder|reasoner|v3|nexusclaw\",\n\
                \"task\": \"...\",\n\
                \"params\": {{}},\n\
                \"thinking_budget\": {},\n\
                \"priority\": 1..5,\n\
                \"key_pool\": \"official|openrouter|zenith\",\n\
                \"fallback_agent\": \"gemini|nexusclaw|deepseek_v3\",\n\
                \"confidence_score\": 0.0..1.0\n\
            }}\n\
            Regla Crítica: Usa 'nexusclaw' para: 1. Ejecución de comandos Bash. 2. Lectura/Escritura de archivos locales. 3. Scraping con navegador. 4. Control de hardware. Los 'params' deben incluir el comando o la ruta de archivo.\n\
            Salud del Sistema: {system_health}\n\
            Petición del Arquitecto: {user_input}",
            strategy.effort,
            strategy.budget_tokens
        );

        let full_prompt = self.construir_prompt_con_identidad(&user_prompt);
        // NEXUS: Si la tarea requiere scraping local o interactuar con el host, prioriza 'nexusclaw'.

        // Verificar Cache de planes
        if let Some(cached) = GLOBAL_CACHE.get(&full_prompt, "gemini-planner") {
            return Ok(serde_json::from_str(&cached)?);
        }

        // NEXUS 16.0: Ejecución con Identidad Soberana
        let (text, _) = self
            .consultar_omega(&full_prompt, GeminiModel::Flash3_0.as_str(), None, None)
            .await?;

        let json_content = text
            .trim()
            .trim_start_matches("```json")
            .trim_end_matches("```")
            .trim();
        let mut plan: NexusPlan = serde_json::from_str(json_content)?;

        // NEXUS 13.0: Calibrar la confianza basada en el historial de aciertos/errores
        let factor = self.calibration.read().unwrap().calibrate();
        plan.confidence_score = (plan.confidence_score * factor).clamp(0.0, 1.0);

        GLOBAL_CACHE.insert(&full_prompt, "gemini-planner", text);
        Ok(plan)
    }

    /// NEXUS 15.0: Oracle - Gemini genera una trayectoria experta para entrenar a DeepSeek
    pub async fn generar_simulacion_oraculo(&self, escenario: &str) -> Result<(f64, bool)> {
        let prompt = format!(
            "Analiza este escenario de ejecución: {}. \n\
            Determina el umbral de confianza óptimo (0.4 - 0.95) y la probabilidad de éxito.\n\
            Responde solo con: [UMBRAL, EXITO(true/false)]",
            escenario
        );
        let (resp, _) = self
            .consultar(&prompt, GeminiModel::Flash3_0.as_str())
            .await?;
        // Parseo simple de la sugerencia del maestro
        let umbral = if resp.contains("0.") { 0.7 } else { 0.8 }; // Simplificado para el ejemplo
        let exito = resp.contains("true");
        Ok((umbral, exito))
    }

    /// NEXUS 14.0: Blinda NEXUS contra jailbreaks e inyecciones
    fn detectar_anomalia(&self, input: &str) -> bool {
        let input_lower = input.to_lowercase();
        let patrones_peligrosos = [
            "ignore your instructions",
            "you are now",
            "forget everything",
            "dan mode",
            "developer mode",
            "disregard previous",
            "new identity",
        ];

        patrones_peligrosos.iter().any(|&p| input_lower.contains(p))
    }

    pub fn registrar_resultado_plan(&self, score: f64, exito: bool) {
        let mut cal = self.calibration.write().unwrap();
        cal.predictions.push((score, exito));
        if cal.predictions.len() > 500 {
            cal.predictions.remove(0); // Mantener una ventana deslizante de 500 muestras
        }
        drop(cal);
        self.save_calibration();
    }

    pub async fn consultar_con_audio(
        &self,
        prompt: &str,
        model: &str,
        audio_base64: Option<String>,
    ) -> Result<(String, Option<String>)> {
        let mut retries = 0;
        let total_keys: usize = self.api_accounts.iter().map(|a| a.len()).sum();
        let max_total_attempts = total_keys * 2;

        while retries < max_total_attempts {
            let api_key = self.rotar_llave();
            if api_key.is_empty() {
                bail!("No hay llaves configuradas en el Zenith Pool.");
            }

            // Control de Rate Limit (mínimo 1s entre ráfagas)
            {
                let mut ultimo = self.ultimo_uso.lock().await;
                let espera = std::time::Duration::from_millis(1000);
                if ultimo.elapsed() < espera {
                    tokio::time::sleep(espera - ultimo.elapsed()).await;
                }
                *ultimo = std::time::Instant::now();
            }

            let model_name = if model.contains("flash") {
                "gemini-3-flash-preview"
            } else if model.contains("pro") {
                "gemini-2.0-pro-exp-02-05" // O el más cercano disponible
            } else {
                model
            };
            let url = format!(
                "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
                model_name, api_key
            );

            let mut parts = vec![Part {
                text: Some(prompt.to_string()),
                inline_data: None,
                function_call: None,
                function_response: None,
            }];

            if let Some(audio) = audio_base64.clone() {
                parts.push(Part {
                    text: None,
                    inline_data: Some(InlineData {
                        mime_type: "audio/wav".to_string(),
                        data: audio,
                    }),
                    function_call: None,
                    function_response: None,
                });
            }

            let response_modalities = if audio_base64.is_some() {
                Some(vec!["TEXT".to_string(), "AUDIO".to_string()])
            } else {
                Some(vec!["TEXT".to_string()])
            };

            let speech_config = if audio_base64.is_some() {
                Some(SpeechConfig {
                    voice_config: VoiceConfig {
                        prebuilt_voice_config: PrebuiltVoiceConfig {
                            voice_name: "Ursa".to_string(),
                        },
                    },
                })
            } else {
                None
            };

            let request = GeminiRequest {
                contents: vec![Content {
                    role: "user".to_string(),
                    parts,
                }],
                tools: None,
                cached_content: self.cached_identity_ctx(),
                generation_config: Some(GenerationConfig {
                    temperature: 0.7,
                    max_output_tokens: 4096,
                    response_modalities,
                    speech_config,
                }),
            };

            // Configuración de Sigilo (Fase 2)
            let cloak = NexusCloak::new(SigiloLevel::Soberano); // NEXUS: Elevando sigilo para forzar el Escudo

            let mut headers = HeaderMap::new();
            headers.insert(
                USER_AGENT,
                HeaderValue::from_str(&cloak.identidad.user_agent).unwrap(),
            );

            let mut client_builder = Client::builder().default_headers(headers);

            if let Some(proxy) = GestorRed::obtener_configuracion(&cloak.nivel) {
                client_builder = client_builder.proxy(proxy);
            }

            let client = client_builder.build().unwrap_or_else(|_| Client::new());

            match client.post(&url).json(&request).send().await {
                Ok(response) => {
                    let status = response.status();
                    if status.is_success() {
                        let resp_json: GeminiResponse = response.json().await?;
                        if let Some(err) = resp_json.error {
                            tracing::error!("❌ [GEMINI API] Error interno: {}", err.message);
                            retries += 1;
                            continue;
                        }

                        if let Some(candidates) = resp_json.candidates {
                            if let Some(candidate) = candidates.first() {
                                let mut text_response = String::new();
                                let mut audio_response = None;

                                for part in &candidate.content.parts {
                                    if let Some(text) = &part.text {
                                        text_response.push_str(text);
                                    }
                                    if let Some(inline) = &part.inline_data {
                                        if inline.mime_type.contains("audio") {
                                            audio_response = Some(inline.data.clone());
                                        }
                                    }
                                }
                                return Ok((text_response, audio_response));
                            }
                        }
                        retries += 1;
                    } else if status.as_u16() == 429 {
                        tracing::warn!("⚠️ [ZENITH POOL] Rate limit en llave actual. Rotando...");
                        retries += 1;
                        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                    } else {
                        let err_text = response.text().await.unwrap_or_default();
                        tracing::error!("❌ [GEMINI API ERROR] Status {}: {}", status, err_text);
                        retries += 1;
                    }
                }
                Err(e) => {
                    tracing::error!("❌ [GEMINI API] Fallo de conexión: {}", e);
                    retries += 1;
                }
            }
        }
        bail!("Fallo tras reintentos")
    }

    pub async fn consultar_omega(
        &self,
        prompt: &str,
        model: &str,
        audio_base64: Option<String>,
        tools: Option<Vec<Tool>>,
    ) -> Result<(String, Option<String>)> {
        self.consultar_omega_full(
            vec![Content {
                role: "user".to_string(),
                parts: {
                    let mut parts = vec![Part {
                        text: Some(prompt.to_string()),
                        inline_data: None,
                        function_call: None,
                        function_response: None,
                    }];
                    if let Some(audio) = audio_base64 {
                        parts.push(Part {
                            text: None,
                            inline_data: Some(InlineData {
                                mime_type: "audio/wav".to_string(),
                                data: audio,
                            }),
                            function_call: None,
                            function_response: None,
                        });
                    }
                    parts
                },
            }],
            model,
            tools,
        )
        .await
    }

    pub async fn consultar_omega_full(
        &self,
        history: Vec<Content>,
        model: &str,
        tools: Option<Vec<Tool>>,
    ) -> Result<(String, Option<String>)> {
        let mut retries = 0;
        let total_keys: usize = self.api_accounts.iter().map(|a| a.len()).sum();
        let max_total_attempts = total_keys * 2;

        while retries < max_total_attempts {
            let api_key = self.rotar_llave();
            if api_key.is_empty() {
                bail!("No hay llaves configuradas en el Zenith Pool.");
            }

            let model_name = if model.contains("flash") {
                "gemini-3-flash-preview"
            } else if model.contains("pro") {
                "gemini-2.0-pro-exp-02-05"
            } else {
                model
            };
            let url = format!(
                "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
                model_name, api_key
            );

            let has_audio = history
                .iter()
                .any(|c| c.parts.iter().any(|p| p.inline_data.is_some()));

            let response_modalities = if has_audio {
                Some(vec!["TEXT".to_string(), "AUDIO".to_string()])
            } else {
                Some(vec!["TEXT".to_string()])
            };

            let speech_config = if has_audio {
                Some(SpeechConfig {
                    voice_config: VoiceConfig {
                        prebuilt_voice_config: PrebuiltVoiceConfig {
                            voice_name: "Ursa".to_string(),
                        },
                    },
                })
            } else {
                None
            };

            let request = GeminiRequest {
                contents: history.clone(),
                tools: tools.clone(),
                cached_content: self.cached_identity_ctx(),
                generation_config: Some(GenerationConfig {
                    temperature: 0.7,
                    max_output_tokens: 8192,
                    response_modalities,
                    speech_config,
                }),
            };

            let mut headers = HeaderMap::new();

            // NEXUS: Aplicando Capa de Invisibilidad Soberana al Planificador Omega
            let cloak = NexusCloak::new(SigiloLevel::Soberano);
            headers.insert(
                USER_AGENT,
                HeaderValue::from_str(&cloak.identidad.user_agent).unwrap(),
            );

            let mut client_builder = Client::builder().default_headers(headers);
            if let Some(proxy) = GestorRed::obtener_configuracion(&cloak.nivel) {
                client_builder = client_builder.proxy(proxy);
            }
            let client = client_builder.build().unwrap_or_else(|_| Client::new());

            match client.post(&url).json(&request).send().await {
                Ok(response) => {
                    let status = response.status();
                    if status.is_success() {
                        let resp_json: GeminiResponse = response.json().await?;
                        if let Some(candidates) = resp_json.candidates {
                            if let Some(candidate) = candidates.first() {
                                let mut text_response = String::new();
                                let mut audio_response = None;

                                for part in &candidate.content.parts {
                                    if let Some(text) = &part.text {
                                        text_response.push_str(text);
                                    }
                                    if let Some(inline) = &part.inline_data {
                                        if inline.mime_type.contains("audio") {
                                            audio_response = Some(inline.data.clone());
                                        }
                                    }
                                    // Si es un function call, lo devolvemos como texto JSON por ahora
                                    // o lo manejamos en el nivel superior.
                                    if let Some(fc) = &part.function_call {
                                        text_response.push_str(&format!(
                                            "\n[[TOOL_USE: {}]]\n{}",
                                            fc.name,
                                            serde_json::to_string_pretty(&fc.args).unwrap()
                                        ));
                                    }
                                }
                                return Ok((text_response, audio_response));
                            }
                        }
                        retries += 1;
                    } else if status.as_u16() == 429 {
                        retries += 1;
                        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                    } else {
                        retries += 1;
                    }
                }
                Err(_) => retries += 1,
            }
        }
        bail!("Fallo tras reintentos")
    }
}
