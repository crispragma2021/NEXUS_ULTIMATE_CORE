// ==========================================
// 👁️ OCR_VISION — El Lóbulo Occipital de NEXUS
// ==========================================
// Da OJOS al SLM local y a la nube según el uso que cada uno necesita:
//
//   🧠 MODO LOCAL (SLM con visión nativa vía Ollama /api/chat):
//       - Modelo elegible (default qwen2.5vl:7b, ya instalado)
//       - Describe/transcribe imágenes EN VIVO (pantalla, archivos)
//       - Sin nube, sin llaves, soberano al 100%
//
//   ☁️ MODO NUBE (OpenRouter multimodal, primario):
//       - Usa OPENROUTER_API_KEY del entorno (NUNCA hardcodeada)
//       - Sin dependencia de cuentas Google AI Studio (lección crispragmatico)
//       - Modelo multimodal vía OpenRouter; Gemini AI Studio queda como ULTIMO respaldo
//
//   🧠 MODO DEEPSEEK (text-only → necesita front-end OCR):
//       - Tesseract local (spa+eng): base rápida, sin GPU, ya instalado
//       - PaddleOCR v4 / PP-Structure: rápido, 80+ idiomas, CPU (CLI opcional)
//       - GOT-OCR 2.0: estructura Markdown para documentos/PDFs/tablas (CLI opcional)
//       - Marker / Nougat: libros, papers y PDFs complejos → RAG (CLI opcional)
//       - El texto extraído se envía a DeepSeek (API oficial con DEEPSEEK_API_KEY)
//
//   🤖 MODO AUTO: si hay internet → Nube (OpenRouter/Gemini); si no → Local (SLM).
//
// ⚠️ DETALLE CLAVE: la NUBE (OpenRouter/Gemini) NO necesita OCR.
//    Los modelos multimodales analizan la imagen DIRECTAMENTE (visión nativa).
//    El OCR (tesseract/paddleocr/got-ocr) SOLO se usa en el modo DEEPSEEK,
//    porque DeepSeek es texto puro y necesita un front-end que extraiga el texto.
//
// 🔒 SIN llaves hardcodeadas: todas las credenciales salen del entorno.
// ==========================================

use base64::{engine::general_purpose::STANDARD, Engine as _};
use serde_json::{json, Value};
use std::process::{Command, Stdio};
use std::time::Instant;
use tracing::{info, warn};

/// Modelo de visión local por defecto (verificado con `ollama list`).
pub const MODELO_VISION_LOCAL_DEFAULT: &str = "qwen2.5vl:7b";
/// Cadena de modelos de visión en la nube vía OpenRouter, por disponibilidad:
/// 1. Gemini 2.5 Flash (primario, el mejor)
/// 2. Mimo v2.5 (gratis vía OpenCode Zen) si Gemini falla
/// 3. Hy3 (gratis vía OpenCode Zen) como último respaldo
pub const CADENA_VISION_NUBE: &[&str] = &[
    "google/gemini-2.5-flash",
    "xiaomi/mimo-v2.5",
    "tencent/hy3",
];
/// Modelo de visión en la nube vía OpenRouter (primario, sin Google AI Studio).
pub const MODELO_VISION_NUBE: &str = "google/gemini-2.5-flash";
/// Modelo de visión en la nube vía OpenRouter para video (multi-frame).
pub const MODELO_VISION_NUBE_VIDEO: &str = "google/gemini-2.5-flash";
/// Modelo de visión de respaldo (Gemini AI Studio directo — solo si OpenRouter falla).
pub const MODELO_VISION_GEMINI_DIRECTO: &str = "gemini-2.5-flash";

/// Motores de visión disponibles.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MotorVision {
    /// SLM local con visión nativa (Ollama /api/chat + images base64).
    LocalSlm,
    /// Nube multimodal con visión NATIVA (SIN OCR): OpenRouter primario,
    /// Gemini AI Studio directo como último respaldo.
    Nube,
    /// DeepSeek (text-only) con front-end OCR (Tesseract/PaddleOCR/GOT-OCR).
    DeepSeek,
    /// Auto: si hay internet → Nube (Gemini); si no → Local (SLM).
    Auto,
}

/// Modo de análisis visual.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModoVision {
    /// Solo extraer el texto (OCR puro).
    Transcribir,
    /// Describir qué hay en la imagen (escena, objetos, contexto).
    Describir,
    /// Extraer estructura (tablas/Markdown) — mejor con GOT-OCR o Gemini.
    Estructura,
}

impl ModoVision {
    pub fn parsear(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "describir" | "describe" => Self::Describir,
            "estructura" | "markdown" | "tabla" | "tablas" => Self::Estructura,
            _ => Self::Transcribir,
        }
    }
}

/// Resultado de un análisis visual.
#[derive(Debug, Clone)]
pub struct ResultadoVision {
    pub texto: String,
    pub motor: String,
    pub origen: String,
    pub latencia_ms: u64,
}

/// URL base de Ollama (configurable por entorno).
fn ollama_base() -> String {
    std::env::var("OLLAMA_API_URL").unwrap_or_else(|_| "http://localhost:11434".to_string())
}

/// URL de la API oficial de DeepSeek (compatible con OpenAI).
const DEEPSEEK_API_URL: &str = "https://api.deepseek.com/v1/chat/completions";

// ═══════════════════════════════════════════════════════════════════
// 1️⃣  MODO LOCAL — SLM con visión nativa vía Ollama (modelo elegible)
// ═══════════════════════════════════════════════════════════════════

/// Envía una imagen (bytes PNG) al modelo de visión local elegido vía Ollama.
/// Reutiliza el patrón /api/chat de NexusClawPro pero con `images` (base64).
pub async fn analizar_con_slm_local(
    image_bytes: &[u8],
    pregunta: &str,
    modelo: &str,
) -> Result<String, String> {
    let b64 = STANDARD.encode(image_bytes);
    let url = format!("{}/api/chat", ollama_base());

    let payload = json!({
        "model": modelo,
        "messages": [
            {
                "role": "user",
                "content": pregunta,
                "images": [b64]
            }
        ],
        "stream": false,
        "options": {
            "temperature": 0.3,
            "num_gpu": 99
        }
    });

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(180))
        .build()
        .map_err(|e| format!("❌ Error creando cliente HTTP: {}", e))?;

    info!(
        "👁️ [OCR_VISION] Enviando imagen a {} (SLM local)...",
        modelo
    );
    let resp = client
        .post(&url)
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await
        .map_err(|e| format!("❌ Error de conexión con Ollama: {}", e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!(
            "⚠️ Ollama HTTP {}: {}",
            status,
            &body[..body.len().min(300)]
        ));
    }

    let data: Value = resp
        .json()
        .await
        .map_err(|e| format!("❌ Fallo parseando respuesta de Ollama: {}", e))?;

    data["message"]["content"]
        .as_str()
        .map(|s| s.trim().to_string())
        .ok_or_else(|| "⚠️ Ollama no devolvió contenido en la respuesta.".to_string())
}

/// Detecta si el modelo de visión local está disponible en Ollama.
pub async fn slm_local_disponible(modelo: &str) -> bool {
    let url = format!("{}/api/tags", ollama_base());
    let client = match reqwest::Client::new().get(&url).send().await {
        Ok(r) if r.status().is_success() => r,
        _ => return false,
    };
    let data: Value = match client.json().await {
        Ok(v) => v,
        Err(_) => return false,
    };
    data["models"]
        .as_array()
        .map(|models| {
            models.iter().any(|m| {
                m["name"]
                    .as_str()
                    .map(|n| n.starts_with(modelo))
                    .unwrap_or(false)
            })
        })
        .unwrap_or(false)
}

/// Lista los modelos con capacidad de visión disponibles en Ollama.
pub async fn listar_modelos_vision() -> Vec<String> {
    let url = format!("{}/api/tags", ollama_base());
    let Ok(resp) = reqwest::Client::new().get(&url).send().await else {
        return Vec::new();
    };
    if !resp.status().is_success() {
        return Vec::new();
    }
    let Ok(data) = resp.json::<Value>().await else {
        return Vec::new();
    };
    data["models"]
        .as_array()
        .map(|models| {
            models
                .iter()
                .filter(|m| {
                    m["capabilities"]
                        .as_array()
                        .map(|caps| {
                            caps.iter()
                                .any(|c| c.as_str().map(|s| s == "vision").unwrap_or(false))
                        })
                        .unwrap_or(false)
                })
                .filter_map(|m| m["name"].as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default()
}

// ═══════════════════════════════════════════════════════════════════
// 2️⃣  MODO NUBE — Gemini multimodal (visión nativa en la nube)
// ═══════════════════════════════════════════════════════════════════

/// Envía una imagen a Gemini (multimodal) usando la llave del entorno.
/// Patrón idéntico a ZenithPool::analizar_imagen pero SIN ZENITH_ACCOUNTS:
/// lee GEMINI_API_KEY o GOOGLE_API_KEY directamente (lección GitGuardian).
pub async fn analizar_con_gemini(
    image_bytes: &[u8],
    mime_type: &str,
    prompt: &str,
) -> Result<String, String> {
    let api_key = std::env::var("GEMINI_API_KEY")
        .or_else(|_| std::env::var("GOOGLE_API_KEY"))
        .map_err(|_| {
            "❌ Falta GEMINI_API_KEY/GOOGLE_API_KEY en el entorno (exportala en .env).".to_string()
        })?;

    let b64 = STANDARD.encode(image_bytes);
    // Gemini AI Studio DIRECTO usa el id corto del modelo (ej: gemini-2.5-flash),
    // NO el id con prefijo de OpenRouter (google/gemini-2.5-flash).
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
        MODELO_VISION_GEMINI_DIRECTO, api_key
    );

    let payload = json!({
        "contents": [{
            "parts": [
                {"inline_data": {"mime_type": mime_type, "data": b64}},
                {"text": prompt}
            ]
        }]
    });

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .map_err(|e| format!("❌ Error creando cliente HTTP: {}", e))?;

    info!(
        "☁️ [OCR_VISION] Respaldo: enviando imagen a Gemini AI Studio directo ({})...",
        MODELO_VISION_GEMINI_DIRECTO
    );
    let resp = client
        .post(&url)
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await
        .map_err(|e| format!("❌ Error de conexión con Gemini: {}", e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!(
            "⚠️ Gemini HTTP {}: {}",
            status,
            &body[..body.len().min(300)]
        ));
    }

    let data: Value = resp
        .json()
        .await
        .map_err(|e| format!("❌ Fallo parseando respuesta de Gemini: {}", e))?;

    data["candidates"][0]["content"]["parts"]
        .as_array()
        .and_then(|parts| {
            parts
                .iter()
                .filter_map(|p| p["text"].as_str())
                .collect::<Vec<_>>()
                .join("\n")
                .into()
        })
        .map(|s: String| s.trim().to_string())
        .ok_or_else(|| "⚠️ Gemini no devolvió contenido en la respuesta.".to_string())
}

// ─── Visión Nube vía OpenRouter (PRIMARIO) ──────────────────────────────────
// OpenRouter NO depende de cuentas de Google AI Studio: una sola llave da acceso
// a cientos de modelos multimodales (incluidos Gemini/Claude/GPT).
// Patrón OpenAI-compatible: content con image_url (data URL base64).

/// Envía una imagen a un modelo multimodal vía OpenRouter (primario).
/// Prueba la cadena completa por disponibilidad: Gemini → Mimo → Hy3.
/// Si OPENROUTER_API_KEY no está, o toda la cadena falla, devuelve Err
/// para que el llamador pruebe el respaldo Gemini AI Studio directo.
pub async fn analizar_con_openrouter(
    image_bytes: &[u8],
    mime_type: &str,
    prompt: &str,
) -> Result<String, String> {
    let api_key = std::env::var("OPENROUTER_API_KEY").map_err(|_| {
        "❌ Falta OPENROUTER_API_KEY en el entorno (exportala en .env).".to_string()
    })?;
    if api_key.is_empty() {
        return Err("❌ OPENROUTER_API_KEY está vacía.".to_string());
    }

    let b64 = STANDARD.encode(image_bytes);
    let data_url = format!("data:{};base64,{}", mime_type, b64);

    let mut ultimo_error = String::new();
    for modelo in CADENA_VISION_NUBE {
        let payload = json!({
            "model": modelo,
            "messages": [{
                "role": "user",
                "content": [
                    {"type": "image_url", "image_url": {"url": data_url}},
                    {"type": "text", "text": prompt}
                ]
            }],
            "max_tokens": 4096
        });
        match enviar_openrouter(&api_key, &payload).await {
            Ok(t) => return Ok(t),
            Err(e) => {
                warn!("⚠️ [OCR_VISION] Modelo {} falló ({}). Probando siguiente...", modelo, e);
                ultimo_error = e;
            }
        }
    }
    Err(format!("❌ Toda la cadena de visión nube falló: {}", ultimo_error))
}

/// Ejecuta una llamada HTTP a OpenRouter con un payload dado.
async fn enviar_openrouter(api_key: &str, payload: &Value) -> Result<String, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .map_err(|e| format!("❌ Error creando cliente HTTP: {}", e))?;

    let modelo = payload["model"].as_str().unwrap_or("?");
    info!("☁️ [OCR_VISION] Enviando imagen a OpenRouter ({})...", modelo);
    let resp = client
        .post("https://openrouter.ai/api/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .header("HTTP-Referer", "https://nexus-omega.app")
        .header("X-Title", "NEXUS Omega")
        .json(payload)
        .send()
        .await
        .map_err(|e| format!("❌ Error de conexión con OpenRouter: {}", e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!(
            "⚠️ OpenRouter HTTP {}: {}",
            status,
            &body[..body.len().min(300)]
        ));
    }

    let data: Value = resp
        .json()
        .await
        .map_err(|e| format!("❌ Fallo parseando respuesta de OpenRouter: {}", e))?;

    let msg = &data["choices"][0]["message"];
    // Modelos como Mimo/Hy3 devuelven la respuesta en `reasoning` (thinking)
    // con `content` null. Intentamos content, luego reasoning.
    msg["content"]
        .as_str()
        .map(|s: &str| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .or_else(|| {
            msg["reasoning"]
                .as_str()
                .map(|s: &str| s.trim().to_string())
                .filter(|s| !s.is_empty())
        })
        .ok_or_else(|| "⚠️ OpenRouter no devolvió contenido.".to_string())
}

/// Envía múltiples frames (video) a un modelo multimodal vía OpenRouter.
/// Los modelos de Gemini vía OpenRouter aceptan varias imágenes en un mensaje.
pub async fn analizar_video_con_openrouter(
    frames: &[Vec<u8>],
    mime_type: &str,
    prompt: &str,
) -> Result<String, String> {
    if frames.is_empty() {
        return Err("❌ Sin frames para analizar.".to_string());
    }
    let api_key = std::env::var("OPENROUTER_API_KEY").map_err(|_| {
        "❌ Falta OPENROUTER_API_KEY en el entorno (exportala en .env).".to_string()
    })?;
    if api_key.is_empty() {
        return Err("❌ OPENROUTER_API_KEY está vacía.".to_string());
    }

    let mut content: Vec<Value> = Vec::new();
    for f in frames {
        let data_url = format!("data:{};base64,{}", mime_type, STANDARD.encode(f));
        content.push(json!({
            "type": "image_url",
            "image_url": {"url": data_url}
        }));
    }
    content.push(json!({"type": "text", "text": prompt}));

    let payload = json!({
        "model": MODELO_VISION_NUBE_VIDEO,
        "messages": [{"role": "user", "content": content}],
        "max_tokens": 4096
    });

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .map_err(|e| format!("❌ Error creando cliente HTTP: {}", e))?;

    info!(
        "☁️ [OCR_VISION] Enviando {} frames a OpenRouter ({})...",
        frames.len(),
        MODELO_VISION_NUBE_VIDEO
    );
    let resp = client
        .post("https://openrouter.ai/api/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .header("HTTP-Referer", "https://nexus-omega.app")
        .header("X-Title", "NEXUS Omega")
        .json(&payload)
        .send()
        .await
        .map_err(|e| format!("❌ Error de conexión con OpenRouter: {}", e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!(
            "⚠️ OpenRouter HTTP {}: {}",
            status,
            &body[..body.len().min(300)]
        ));
    }

    let data: Value = resp
        .json()
        .await
        .map_err(|e| format!("❌ Fallo parseando respuesta de OpenRouter: {}", e))?;

    let msg = &data["choices"][0]["message"];
    // Modelos como Mimo/Hy3 devuelven la respuesta en `reasoning` (thinking)
    // con `content` null. Intentamos content, luego reasoning.
    msg["content"]
        .as_str()
        .map(|s: &str| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .or_else(|| {
            msg["reasoning"]
                .as_str()
                .map(|s: &str| s.trim().to_string())
                .filter(|s| !s.is_empty())
        })
        .ok_or_else(|| "⚠️ OpenRouter no devolvió contenido.".to_string())
}

// ═══════════════════════════════════════════════════════════════════
// 3️⃣  OCR FRONT-END — Tesseract local (base) + motores externos
// ═══════════════════════════════════════════════════════════════════

/// OCR local base con Tesseract (spa+eng). Ya es usado por OmnipresentVision.
fn ocr_tesseract(ruta: &str) -> Option<String> {
    let output = Command::new("tesseract")
        .arg(ruta)
        .arg("stdout")
        .arg("-l")
        .arg("spa+eng")
        .output()
        .ok()?;

    if output.status.success() {
        let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !text.is_empty() {
            return Some(text);
        }
    }
    warn!("⚠️ [OCR_VISION] Tesseract no devolvió texto o no está instalado.");
    None
}

/// Detecta motores OCR externos del ranking (PaddleOCR, GOT-OCR, Marker).
/// Devuelve un mapa comando → disponible.
pub fn detectar_motores_externos() -> Vec<(String, bool)> {
    let candidatos = [
        (
            "paddleocr",
            "PaddleOCR v4 / PP-Structure (rápido, 80+ idiomas, CPU)",
        ),
        (
            "got-ocr",
            "GOT-OCR 2.0 (estructura Markdown, docs/PDFs/tablas)",
        ),
        (
            "marker_single",
            "Marker (libros/papers/PDFs complejos → RAG)",
        ),
        ("nougat", "Nougat (papers académicos → RAG)"),
    ];

    candidatos
        .iter()
        .map(|(cmd, desc)| {
            let found = Command::new("which")
                .arg(cmd)
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);
            (format!("{} — {}", cmd, desc), found)
        })
        .collect()
}

/// Ejecuta un motor OCR externo vía CLI si está instalado.
/// `args` son los argumentos posicionales (ej: ruta del PDF).
fn ocr_externo(motor: &str, ruta: &str) -> Option<String> {
    let mut cmd = Command::new(motor);
    // Diferentes motores tienen diferentes interfaces; todos aceptan la ruta.
    cmd.arg(ruta);
    let output = cmd.output().ok()?;
    if output.status.success() {
        let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !text.is_empty() {
            return Some(text);
        }
    }
    None
}

// ═══════════════════════════════════════════════════════════════════
// 4️⃣  DEEPSEEK — razonamiento sobre el texto extraído por OCR
// ═══════════════════════════════════════════════════════════════════

/// Envía texto (extraído por OCR) a DeepSeek para razonar/estructurar.
/// Lee DEEPSEEK_API_KEY del entorno — NUNCA hardcodeada (incidente GitGuardian).
pub async fn razonar_con_deepseek(texto_ocr: &str, instruccion: &str) -> Result<String, String> {
    let api_key = std::env::var("DEEPSEEK_API_KEY").unwrap_or_default();
    if api_key.is_empty() {
        return Err("❌ Falta DEEPSEEK_API_KEY en el entorno (exportala en .env).".to_string());
    }

    let payload = json!({
        "model": "deepseek-chat",
        "messages": [
            {
                "role": "system",
                "content": "Eres NEXUS. Recibes texto extraído por OCR de una imagen o documento. "
                    .to_string() + instruccion
            },
            {"role": "user", "content": texto_ocr}
        ],
        "temperature": 0.3,
        "max_tokens": 2048
    });

    let client = reqwest::Client::new();
    info!("🧠 [OCR_VISION] Enviando texto OCR a DeepSeek para razonamiento...");
    let resp = client
        .post(DEEPSEEK_API_URL)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&payload)
        .send()
        .await
        .map_err(|e| format!("❌ Error de conexión con DeepSeek: {}", e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!(
            "⚠️ DeepSeek HTTP {}: {}",
            status,
            &body[..body.len().min(300)]
        ));
    }

    let data: Value = resp
        .json()
        .await
        .map_err(|e| format!("❌ Fallo parseando respuesta de DeepSeek: {}", e))?;

    data["choices"][0]["message"]["content"]
        .as_str()
        .map(|s| s.trim().to_string())
        .ok_or_else(|| "⚠️ DeepSeek no devolvió contenido.".to_string())
}

// ═══════════════════════════════════════════════════════════════════
// 5️⃣  ORQUESTACIÓN — punto de entrada unificado
// ═══════════════════════════════════════════════════════════════════

/// Lee bytes de una imagen desde disco o desde la captura de pantalla.
async fn obtener_bytes(origen: &str) -> Result<(Vec<u8>, String), String> {
    if origen.eq_ignore_ascii_case("pantalla") || origen.eq_ignore_ascii_case("screen") {
        // Captura de pantalla normalizada para modelos de visión locales
        // (xcap + Lanczos3 → PNG en RAM, patrón de OmnipresentVision).
        let (w, h) = (768u32, 432u32);
        let bytes =
            crate::sentidos::omnipresent_vision::OmnipresentVision::capturar_para_modelo_local(
                w, h,
            )
            .await
            .ok_or_else(|| "❌ No se pudo capturar la pantalla (xcap).".to_string())?;
        Ok((bytes, "pantalla".to_string()))
    } else {
        let ruta = origen.trim();
        if ruta.is_empty() {
            return Err("❌ Debes indicar 'pantalla' o una ruta de archivo de imagen.".to_string());
        }
        let bytes =
            std::fs::read(ruta).map_err(|e| format!("❌ No se pudo leer '{}': {}", ruta, e))?;
        Ok((bytes, ruta.to_string()))
    }
}

/// Construye la pregunta según el modo de análisis.
fn pregunta_para_modo(modo: ModoVision) -> String {
    match modo {
        ModoVision::Transcribir => {
            "Transcribe TODO el texto visible en esta imagen, palabra por palabra, \
             respetando el idioma. Devuelve solo el texto."
                .to_string()
        }
        ModoVision::Describir => {
            "Describe detalladamente qué hay en esta imagen: objetos, personas, texto, \
             colores, contexto. En español."
                .to_string()
        }
        ModoVision::Estructura => {
            "Extrae la estructura de esta imagen en formato Markdown: tablas, listas, \
             encabezados. Devuelve el Markdown."
                .to_string()
        }
    }
}

/// Analiza una imagen (archivo o pantalla) con el motor elegido.
/// - `motor`: local (SLM con `modelo_local` elegible) |
///            nube (OpenRouter multimodal PRIMARIO, visión nativa SIN OCR;
///                 Gemini AI Studio directo solo como último respaldo) |
///            deepseek (OCR front-end + DeepSeek — único modo que usa OCR) |
///            auto (internet→nube, si no→local)
/// - `modo`: transcribir | describir | estructura
/// - `modelo_local`: modelo Ollama para el modo local (default qwen2.5vl:7b)
pub async fn analizar_imagen(
    origen: &str,
    motor: MotorVision,
    modo: ModoVision,
    modelo_local: &str,
) -> Result<ResultadoVision, String> {
    let start = Instant::now();
    let (bytes, origen_str) = obtener_bytes(origen).await?;
    let pregunta = pregunta_para_modo(modo);

    // Persistir temporalmente para OCRs que necesitan ruta en disco
    let temp_path = "/tmp/nexus_ocr_input.png";
    let _ = std::fs::write(temp_path, &bytes);

    // ═══ MODO LOCAL — SLM con visión nativa vía Ollama ═══
    if motor == MotorVision::LocalSlm || (motor == MotorVision::Auto && !hay_internet().await) {
        if !slm_local_disponible(modelo_local).await {
            return Err(format!(
                "❌ El modelo de visión local '{}' no está disponible en Ollama. Instálalo con: ollama pull {}",
                modelo_local, modelo_local
            ));
        }
        let texto = analizar_con_slm_local(&bytes, &pregunta, modelo_local).await?;
        return Ok(ResultadoVision {
            texto,
            motor: format!("{} (SLM local)", modelo_local),
            origen: origen_str,
            latencia_ms: start.elapsed().as_millis() as u64,
        });
    }

    // ═══ MODO NUBE — OpenRouter multimodal (primario), Gemini directo como respaldo ═══
    if motor == MotorVision::Nube {
        let mime = "image/png";
        let texto = match analizar_con_openrouter(&bytes, mime, &pregunta).await {
            Ok(t) => t,
            Err(e_openrouter) => {
                warn!(
                    "⚠️ [OCR_VISION] OpenRouter falló ({}). Probando Gemini directo como respaldo...",
                    e_openrouter
                );
                analizar_con_gemini(&bytes, mime, &pregunta).await?
            }
        };
        return Ok(ResultadoVision {
            texto,
            motor: format!("{} (nube OpenRouter)", MODELO_VISION_NUBE),
            origen: origen_str,
            latencia_ms: start.elapsed().as_millis() as u64,
        });
    }

    // ═══ MODO DEEPSEEK — OCR front-end + razonamiento ═══
    let texto_ocr = match modo {
        ModoVision::Estructura => {
            // GOT-OCR 2.0 si está instalado (mejor para estructura Markdown),
            // sino tesseract base.
            ocr_externo("got-ocr", temp_path)
                .or_else(|| ocr_tesseract(temp_path))
                .unwrap_or_default()
        }
        ModoVision::Describir => {
            // Describir no es el fuerte de un OCR puro: usa tesseract + DeepSeek
            // para interpretar la escena desde el texto extraído.
            ocr_tesseract(temp_path)
                .or_else(|| ocr_externo("paddleocr", temp_path))
                .unwrap_or_default()
        }
        ModoVision::Transcribir => ocr_tesseract(temp_path)
            .or_else(|| ocr_externo("paddleocr", temp_path))
            .or_else(|| ocr_externo("marker_single", temp_path))
            .unwrap_or_default(),
    };

    if texto_ocr.trim().is_empty() {
        return Err(
            "❌ El OCR no extrajo texto. Verifica que: 1) tesseract esté instalado \
             (sudo apt install tesseract-ocr tesseract-ocr-spa), 2) la imagen tenga texto \
             legible, 3) o instala PaddleOCR/GOT-OCR para mejor precisión."
                .to_string(),
        );
    }

    let instruccion = match modo {
        ModoVision::Estructura => {
            "Estructura el contenido en Markdown limpio: detecta tablas, listas, \
             encabezados y corrige errores obvios de OCR. Devuelve solo el Markdown."
                .to_string()
        }
        ModoVision::Describir => {
            "Con el texto visible extraído por OCR, describe qué tipo de documento/imagen es, \
             su propósito y resume su contenido. En español."
                .to_string()
        }
        ModoVision::Transcribir => {
            "Corrige los errores evidentes de OCR y devuelve el texto limpio y fiel al original."
                .to_string()
        }
    };

    let texto = razonar_con_deepseek(&texto_ocr, &instruccion).await?;

    Ok(ResultadoVision {
        texto,
        motor: "deepseek-chat (OCR: tesseract/paddleocr/got-ocr)".to_string(),
        origen: origen_str,
        latencia_ms: start.elapsed().as_millis() as u64,
    })
}

// ═══════════════════════════════════════════════════════════════════
// 6️⃣  VIDEO STREAMING — ojos que ven movimiento (frames múltiples)
// ═══════════════════════════════════════════════════════════════════
// Los modelos multimodales (qwen2.5vl vía Ollama, OpenRouter y Gemini)
// aceptan SECUENCIAS de imágenes: cada frame es una "foto" del video.
// Esto da a NEXUS percepción temporal: puede ver qué CAMBIÓ entre frames.
//
// Fuentes:
//   - Archivo de video (mp4/webm/mkv/avi) → ffmpeg extrae frames
//   - Streaming en vivo desde pantalla → captura xcap a intervalos
//
// Motores:
//   - local   → múltiples `images[]` en un solo mensaje a Ollama
//   - nube    → OpenRouter multimodal PRIMARIO (sin OCR, visión nativa);
//               Gemini AI Studio directo solo como último respaldo
//   - deepseek→ OCR de cada frame + DeepSeek resume la secuencia
//               (DeepSeek es texto puro: SOLO este modo necesita OCR)

/// Detecta si un origen es un archivo de video por su extensión.
pub fn es_archivo_video(ruta: &str) -> bool {
    let ext = ruta.rsplit('.').next().unwrap_or("").to_lowercase();
    matches!(
        ext.as_str(),
        "mp4" | "webm" | "mkv" | "avi" | "mov" | "m4v" | "flv" | "wmv"
    )
}

/// Extrae frames de un archivo de video con ffmpeg (si está instalado).
/// Devuelve los frames como PNG en RAM. Requiere `ffmpeg` en PATH.
pub async fn extraer_frames_video(
    ruta: &str,
    fps: u32,
    max_frames: usize,
) -> Result<Vec<Vec<u8>>, String> {
    let dir = std::env::temp_dir().join(format!("nexus_video_frames_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir)
        .map_err(|e| format!("❌ No se pudo crear dir temporal de frames: {}", e))?;

    let pattern = dir.join("frame_%03d.png");
    let output = Command::new("ffmpeg")
        .arg("-y")
        .arg("-i")
        .arg(ruta)
        .arg("-vf")
        .arg(format!("fps={}", fps))
        .arg("-frames:v")
        .arg(max_frames.to_string())
        .arg(pattern.to_str().unwrap_or(""))
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| format!("❌ ffmpeg no está instalado o falló al ejecutarse: {}", e))?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        let _ = std::fs::remove_dir_all(&dir);
        return Err(format!("⚠️ ffmpeg falló: {}", &err[..err.len().min(300)]));
    }

    let mut frames = Vec::new();
    let mut n = 1;
    loop {
        let p = dir.join(format!("frame_{:03}.png", n));
        if !p.exists() {
            break;
        }
        match std::fs::read(&p) {
            Ok(bytes) => frames.push(bytes),
            Err(_) => break,
        }
        n += 1;
        if n > max_frames {
            break;
        }
    }

    let _ = std::fs::remove_dir_all(&dir);

    if frames.is_empty() {
        return Err(
            "❌ No se extrajeron frames del video (¿ruta inválida o sin track de video?)."
                .to_string(),
        );
    }
    Ok(frames)
}

/// Captura un stream en vivo desde la pantalla: toma `fps` capturas por
/// segundo durante `duracion_seg`. Devuelve los frames como PNG en RAM.
pub async fn capturar_stream_pantalla(fps: u32, duracion_seg: u64) -> Vec<Vec<u8>> {
    use crate::sentidos::omnipresent_vision::OmnipresentVision;

    let fps_real = fps.max(1) as u64;
    let intervalo = std::time::Duration::from_millis(1000 / fps_real);
    let total = fps_real.saturating_mul(duracion_seg.max(1)).max(1);

    let mut frames = Vec::new();
    for _ in 0..total {
        if let Some(b) = OmnipresentVision::capturar_para_modelo_local(768, 432).await {
            frames.push(b);
        }
        tokio::time::sleep(intervalo).await;
    }
    frames
}

/// Envía una secuencia de frames al SLM local de visión (Ollama).
/// Ollama acepta varias imágenes en `images[]` del mismo mensaje.
pub async fn analizar_video_con_slm_local(
    frames: &[Vec<u8>],
    pregunta: &str,
    modelo: &str,
) -> Result<String, String> {
    if frames.is_empty() {
        return Err("❌ Sin frames para analizar.".to_string());
    }
    let b64s: Vec<String> = frames.iter().map(|f| STANDARD.encode(f)).collect();
    let url = format!("{}/api/chat", ollama_base());

    let payload = json!({
        "model": modelo,
        "messages": [
            {
                "role": "user",
                "content": pregunta,
                "images": b64s
            }
        ],
        "stream": false,
        "options": {"temperature": 0.3, "num_gpu": 99}
    });

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(300))
        .build()
        .map_err(|e| format!("❌ Error creando cliente HTTP: {}", e))?;

    info!(
        "👁️ [OCR_VISION] Enviando {} frames a {} (SLM local)...",
        frames.len(),
        modelo
    );
    let resp = client
        .post(&url)
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await
        .map_err(|e| format!("❌ Error de conexión con Ollama: {}", e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!(
            "⚠️ Ollama HTTP {}: {}",
            status,
            &body[..body.len().min(300)]
        ));
    }

    let data: Value = resp
        .json()
        .await
        .map_err(|e| format!("❌ Fallo parseando respuesta de Ollama: {}", e))?;

    data["message"]["content"]
        .as_str()
        .map(|s| s.trim().to_string())
        .ok_or_else(|| "⚠️ Ollama no devolvió contenido.".to_string())
}

/// Envía una secuencia de frames a Gemini (multimodal nativo).
/// Cada frame es un `inline_data` en `parts[]` — así se le da video a Gemini.
pub async fn analizar_video_con_gemini(
    frames: &[Vec<u8>],
    mime_type: &str,
    prompt: &str,
) -> Result<String, String> {
    if frames.is_empty() {
        return Err("❌ Sin frames para analizar.".to_string());
    }
    let api_key = std::env::var("GEMINI_API_KEY")
        .or_else(|_| std::env::var("GOOGLE_API_KEY"))
        .map_err(|_| "❌ Falta GEMINI_API_KEY/GOOGLE_API_KEY en el entorno.".to_string())?;

    let mut parts: Vec<Value> = Vec::new();
    for f in frames {
        parts.push(json!({
            "inline_data": {"mime_type": mime_type, "data": STANDARD.encode(f)}
        }));
    }
    parts.push(json!({"text": prompt}));

    // Gemini AI Studio DIRECTO usa el id corto del modelo (ej: gemini-2.5-flash),
    // NO el id con prefijo de OpenRouter (google/gemini-2.5-flash).
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
        MODELO_VISION_GEMINI_DIRECTO, api_key
    );
    let payload = json!({"contents": [{"parts": parts}]});

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .map_err(|e| format!("❌ Error creando cliente HTTP: {}", e))?;

    info!(
        "☁️ [OCR_VISION] Respaldo: enviando {} frames a Gemini AI Studio directo ({})...",
        frames.len(),
        MODELO_VISION_GEMINI_DIRECTO
    );
    let resp = client
        .post(&url)
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await
        .map_err(|e| format!("❌ Error de conexión con Gemini: {}", e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!(
            "⚠️ Gemini HTTP {}: {}",
            status,
            &body[..body.len().min(300)]
        ));
    }

    let data: Value = resp
        .json()
        .await
        .map_err(|e| format!("❌ Fallo parseando respuesta de Gemini: {}", e))?;

    data["candidates"][0]["content"]["parts"]
        .as_array()
        .and_then(|ps| {
            ps.iter()
                .filter_map(|p| p["text"].as_str())
                .collect::<Vec<_>>()
                .join("\n")
                .into()
        })
        .map(|s: String| s.trim().to_string())
        .ok_or_else(|| "⚠️ Gemini no devolvió contenido.".to_string())
}

/// Analiza una secuencia de video (archivo con ffmpeg o stream en vivo).
/// - origen: ruta de video (mp4/webm/mkv/...) o 'stream'/'live'/'pantalla'
/// - motor: local | nube | deepseek | auto
/// - fps: frames por segundo a extraer/capturar (default 2)
/// - duracion_seg: solo para stream en vivo (default 5)
pub async fn analizar_video(
    origen: &str,
    motor: MotorVision,
    modo: ModoVision,
    modelo_local: &str,
    fps: u32,
    duracion_seg: u64,
) -> Result<ResultadoVision, String> {
    let start = Instant::now();
    let es_stream = origen.eq_ignore_ascii_case("stream")
        || origen.eq_ignore_ascii_case("live")
        || origen.eq_ignore_ascii_case("pantalla")
        || origen.eq_ignore_ascii_case("screen");

    let (frames, fuente) = if es_stream {
        let frames = capturar_stream_pantalla(fps, duracion_seg).await;
        if frames.is_empty() {
            return Err(
                "❌ No se pudo capturar el stream (¿xcap disponible y monitor activo?)."
                    .to_string(),
            );
        }
        (
            frames,
            format!("stream en vivo ({}s @ {}fps)", duracion_seg, fps),
        )
    } else {
        let frames = extraer_frames_video(origen, fps, 30).await?;
        (frames, format!("video: {}", origen))
    };

    let pregunta = pregunta_para_modo(modo);

    // ═══ MODO LOCAL — SLM con visión nativa ═══
    if motor == MotorVision::LocalSlm || (motor == MotorVision::Auto && !hay_internet().await) {
        if !slm_local_disponible(modelo_local).await {
            return Err(format!(
                "❌ El modelo de visión local '{}' no está disponible en Ollama. Instálalo con: ollama pull {}",
                modelo_local, modelo_local
            ));
        }
        let texto = analizar_video_con_slm_local(&frames, &pregunta, modelo_local).await?;
        return Ok(ResultadoVision {
            texto,
            motor: format!("{} (SLM local, {} frames)", modelo_local, frames.len()),
            origen: fuente,
            latencia_ms: start.elapsed().as_millis() as u64,
        });
    }

    // ═══ MODO NUBE — OpenRouter multimodal (primario), Gemini directo como respaldo ═══
    if motor == MotorVision::Nube {
        let texto = match analizar_video_con_openrouter(&frames, "image/png", &pregunta).await {
            Ok(t) => t,
            Err(e_openrouter) => {
                warn!(
                    "⚠️ [OCR_VISION] OpenRouter video falló ({}). Probando Gemini directo...",
                    e_openrouter
                );
                analizar_video_con_gemini(&frames, "image/png", &pregunta).await?
            }
        };
        return Ok(ResultadoVision {
            texto,
            motor: format!(
                "{} (nube OpenRouter, {} frames)",
                MODELO_VISION_NUBE_VIDEO,
                frames.len()
            ),
            origen: fuente,
            latencia_ms: start.elapsed().as_millis() as u64,
        });
    }

    // ═══ MODO DEEPSEEK — OCR de cada frame + DeepSeek resume ═══
    let mut textos: Vec<String> = Vec::new();
    for (i, frame) in frames.iter().enumerate() {
        let temp = format!("/tmp/nexus_video_frame_{}.png", i);
        let _ = std::fs::write(&temp, frame);
        if let Some(t) = ocr_tesseract(&temp) {
            textos.push(t);
        }
        let _ = std::fs::remove_file(&temp);
    }
    if textos.is_empty() {
        return Err("❌ El OCR no extrajo texto de ningún frame.".to_string());
    }
    let crudo = textos.join("\n--- frame ---\n");
    let instruccion =
        "Recibes el texto OCR de varios frames de un video (separados por '--- frame ---'). "
            .to_string()
            + "Identifica qué está ocurriendo en la secuencia, qué cambia entre frames, "
            + "y devuelve un resumen cronológico en español.";
    let texto = razonar_con_deepseek(&crudo, &instruccion).await?;

    Ok(ResultadoVision {
        texto,
        motor: format!("deepseek-chat (OCR por frame, {} frames)", frames.len()),
        origen: fuente,
        latencia_ms: start.elapsed().as_millis() as u64,
    })
}

/// Verifica conectividad a internet vía TCP (patrón OMEGA de NexusClawPro).
async fn hay_internet() -> bool {
    use std::net::SocketAddr;
    use std::time::Duration;
    match tokio::time::timeout(
        Duration::from_secs(2),
        tokio::net::TcpStream::connect(SocketAddr::from(([8, 8, 8, 8], 443))),
    )
    .await
    {
        Ok(Ok(_)) => true,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parsea_modo_vision() {
        assert_eq!(ModoVision::parsear("transcribir"), ModoVision::Transcribir);
        assert_eq!(ModoVision::parsear("describir"), ModoVision::Describir);
        assert_eq!(ModoVision::parsear("markdown"), ModoVision::Estructura);
        assert_eq!(ModoVision::parsear("tablas"), ModoVision::Estructura);
        assert_eq!(ModoVision::parsear(""), ModoVision::Transcribir);
    }

    #[test]
    fn motores_externos_no_panica() {
        // Debe devolver la lista de candidatos sin panico, estén o no instalados.
        let motores = detectar_motores_externos();
        assert!(!motores.is_empty());
        assert!(motores.iter().any(|(n, _)| n.contains("paddleocr")));
    }

    #[test]
    fn detecta_archivos_de_video() {
        assert!(es_archivo_video("clase.mp4"));
        assert!(es_archivo_video("/tmp/video.WEBM"));
        assert!(es_archivo_video("pelicula.mkv"));
        assert!(!es_archivo_video("foto.png"));
        assert!(!es_archivo_video("doc.pdf"));
        assert!(!es_archivo_video(""));
    }
}
