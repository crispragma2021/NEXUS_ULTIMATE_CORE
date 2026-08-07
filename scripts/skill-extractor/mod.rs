use chromiumoxide::cdp::browser_protocol::network::BlockPattern;
use chromiumoxide::layout::Point;
use std::time::{Duration, Instant};
use tokio::time::sleep;
use tokio::sync::Mutex;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, AtomicU64, Ordering};
use anyhow::Result;
use crate::elite::BrowserManager;
use crate::selectores_adaptativos::{ConfigProveedor, SelectorDB, encontrar_elemento};
use crate::deepseek_api::DeepSeekAPI;
use crate::gemini_api::GeminiAPI;
use crate::utils::HTTP_CLIENT;
use serde_json::json;
use rand::Rng;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExtractionMode {
    Sovereign, // CDP / Web / Stealth
    Elite,     // Direct API
    Hybrid,    // Try API, then Web
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OmegaModel {
    GeminiFlash,
    GeminiPro,
    GeminiCoder,
    DeepSeekChat,
    DeepSeekReasoner,
    DeepSeekCoder,
    OpenRouter(String),
}

impl OmegaModel {
    pub fn id(&self) -> String {
        match self {
            OmegaModel::GeminiFlash => "gemini-flash-latest".to_string(),
            OmegaModel::GeminiPro => "gemini-pro-latest".to_string(),
            OmegaModel::GeminiCoder => "gemini-2.0-flash-exp".to_string(), // Explicit 2.0 Coder
            OmegaModel::DeepSeekChat => "deepseek-chat".to_string(),
            OmegaModel::DeepSeekReasoner => "deepseek/deepseek-r1".to_string(),
            OmegaModel::DeepSeekCoder => "deepseek-coder".to_string(),
            OmegaModel::OpenRouter(id) => id.clone(),
        }
    }

    pub fn config(&self) -> ConfigProveedor {
        if self.id().contains("gemini") {
            ConfigProveedor::gemini()
        } else {
            ConfigProveedor::deepseek()
        }
    }
}

pub struct ExtractorOmega {
    pub manager: Arc<BrowserManager>,
    pub db: Arc<Mutex<SelectorDB>>,
    pub deepseek_api: Arc<DeepSeekAPI>,
    pub gemini_api: Arc<GeminiAPI>,
    openrouter_api_keys: Vec<String>,
    openrouter_current_index: AtomicUsize,
    gemini_session_token: String,
    pub active_tasks: AtomicUsize,
    pub throttle_ms: AtomicU64,
}

impl ExtractorOmega {
    pub fn new(manager: Arc<BrowserManager>, deepseek_api: Arc<DeepSeekAPI>, gemini_api: Arc<GeminiAPI>, openrouter_keys: Vec<String>, gemini_token: &str) -> Self {
        Self {
            manager,
            db: Arc::new(Mutex::new(SelectorDB::new("config/selectores.json"))),
            deepseek_api,
            gemini_api,
            openrouter_api_keys: openrouter_keys,
            openrouter_current_index: AtomicUsize::new(0),
            gemini_session_token: gemini_token.to_string(),
            active_tasks: AtomicUsize::new(0),
            throttle_ms: AtomicU64::new(0),
        }
    }

    pub fn set_throttle(&self, ms: u64) {
        self.throttle_ms.store(ms, Ordering::SeqCst);
    }

    pub async fn consultar(&self, prompt: &str, model: OmegaModel, mode: ExtractionMode) -> Result<String> {
        self.active_tasks.fetch_add(1, Ordering::SeqCst);
        let throttle = self.throttle_ms.load(Ordering::SeqCst);
        if throttle > 0 { sleep(Duration::from_millis(throttle)).await; }

        let result = match mode {
            ExtractionMode::Elite => self.consultar_api(prompt, &model).await,
            ExtractionMode::Sovereign => self.consultar_web(prompt, &model).await,
            ExtractionMode::Hybrid => {
                match self.consultar_api(prompt, &model).await {
                    Ok(res) => Ok(res),
                    Err(_) => {
                        tracing::warn!("⚠️ [EXTRACTOR OMEGA] Elite falló, intentando Sovereign...");
                        self.consultar_web(prompt, &model).await
                    }
                }
            }
        };

        self.active_tasks.fetch_sub(1, Ordering::SeqCst);
        result
    }

    async fn consultar_api(&self, prompt: &str, model: &OmegaModel) -> Result<String> {
        match model {
            OmegaModel::OpenRouter(id) => self.consultar_openrouter(prompt, id).await,
            OmegaModel::GeminiFlash | OmegaModel::GeminiPro | OmegaModel::GeminiCoder => {
                self.gemini_api.consultar(prompt, &model.id()).await
            },
            _ => {
                let ds_model = match model {
                    OmegaModel::DeepSeekReasoner => crate::deepseek_api::DeepSeekModel::Reasoner,
                    OmegaModel::DeepSeekCoder => crate::deepseek_api::DeepSeekModel::Coder,
                    _ => crate::deepseek_api::DeepSeekModel::Chat,
                };
                let res = self.deepseek_api.consultar_con_modelo(prompt, &ds_model, None).await?;
                Ok(res.content)
            }
        }
    }

    async fn consultar_web(&self, prompt: &str, model: &OmegaModel) -> Result<String> {
        let config = model.config();
        let profile = format!("omega_{}", config.nombre.to_lowercase());
        let guard = self.manager.acquire(&profile).await?;
        let page = guard.new_page().await?;

        // Inyectar sesión si es Gemini
        if config.nombre == "Gemini" && !self.gemini_session_token.is_empty() {
             let script = format!(
                "document.cookie = '__Secure-1PSID={}; domain=.google.com; path=/; secure; HttpOnly; SameSite=Transparent';",
                self.gemini_session_token
            );
            let _ = page.evaluate(script).await?;
        }

        page.goto(config.url).await?;
        
        // Bloqueo de recursos para velocidad
        page.execute(chromiumoxide::cdp::browser_protocol::network::EnableParams::default()).await?;
        let patterns: Vec<BlockPattern> = vec!["png", "jpg", "jpeg", "gif", "svg", "webp", "css", "analytics"].into_iter().map(|p| BlockPattern::new(p, true)).collect();
        let _ = page.execute(chromiumoxide::cdp::browser_protocol::network::SetBlockedUrLsParams::builder().url_patterns(patterns).build()).await?;

        sleep(Duration::from_secs(3)).await;

        let mut db_lock = self.db.lock().await;
        let textarea = encontrar_elemento(&page, &config.selectores.textarea, &mut *db_lock, config.nombre, "textarea").await?;

        let (x, y) = {
            let mut rng = rand::thread_rng();
            (rng.gen_range(100.0..500.0), rng.gen_range(100.0..500.0))
        };
        let _ = page.move_mouse(Point { x, y }).await;
        
        let wait_ms = {
            let mut rng = rand::thread_rng();
            rng.gen_range(100..300)
        };
        sleep(Duration::from_millis(wait_ms)).await;
        
        textarea.click().await?;
        sleep(Duration::from_millis(200)).await;

        // Zenith Typing: Tipeo ráfaga adaptativa
        for c in prompt.chars() {
            let delay = {
                let mut rng = rand::thread_rng();
                rng.gen_range(15..45)
            };
            textarea.type_str(c.to_string()).await?;
            sleep(Duration::from_millis(delay)).await;
        }

        // Envío via Tecla Enter o Selector de Envío
        let wait_final = {
            let mut rng = rand::thread_rng();
            rng.gen_range(300..600)
        };
        sleep(Duration::from_millis(wait_final)).await;
        if let Ok(btn) = encontrar_elemento(&page, &config.selectores.send, &mut *db_lock, config.nombre, "send").await {
            btn.click().await?;
        } else {
            page.evaluate("document.activeElement.dispatchEvent(new KeyboardEvent('keydown', {key: 'Enter', code: 'Enter', keyCode: 13, bubbles: true}));").await?;
        }
        
        // Captura de respuesta con timeout
        let mut texto = String::new();
        let start = Instant::now();
        while start.elapsed() < Duration::from_secs(25) {
            if let Ok(res_elem) = encontrar_elemento(&page, &config.selectores.respuesta, &mut *db_lock, config.nombre, "respuesta").await {
                if let Ok(Some(t)) = res_elem.inner_text().await {
                    if !t.is_empty() && t.trim().len() > 2 {
                        texto = t;
                        break;
                    }
                }
            }
            sleep(Duration::from_millis(1000)).await;
        }

        page.close().await?;
        if texto.is_empty() { anyhow::bail!("Extractor Sovereign devolvió respuesta vacía"); }
        Ok(texto)
    }

    async fn consultar_openrouter(&self, prompt: &str, model_id: &str) -> Result<String> {
        let pool_size = self.openrouter_api_keys.len();
        let mut retries = 0;
        let max_total_attempts = pool_size * 2;

        while retries < max_total_attempts {
            let idx = self.openrouter_current_index.load(Ordering::SeqCst) % pool_size;
            let api_key = &self.openrouter_api_keys[idx];

            let response = HTTP_CLIENT.post("https://openrouter.ai/api/v1/chat/completions")
                .header("Authorization", format!("Bearer {}", api_key))
                .header("HTTP-Referer", "http://localhost:43211")
                .header("X-Title", "NexusOrquestador OMEGA")
                .json(&json!({
                    "model": model_id,
                    "messages": [{"role": "user", "content": prompt}]
                })).send().await?;
            
            let status = response.status();
            
            if status.is_success() {
                let data: serde_json::Value = response.json().await?;
                return Ok(data["choices"][0]["message"]["content"].as_str().unwrap_or_default().to_string());
            } else if status.as_u16() == 429 {
                self.openrouter_current_index.fetch_add(1, Ordering::SeqCst);
                tracing::warn!("⚠️ [429 OPENROUTER] Clave {} agotada. Rotando...", idx);
                tokio::time::sleep(Duration::from_millis(1000)).await;
                retries += 1;
            } else {
                let data: serde_json::Value = response.json().await?;
                let err = data["error"]["message"].as_str().unwrap_or("Error desconocido");
                anyhow::bail!("OpenRouter Error ({}): {}", status, err);
            }
        }
        anyhow::bail!("OpenRouter Zenith Pool exhausto");
    }
}
