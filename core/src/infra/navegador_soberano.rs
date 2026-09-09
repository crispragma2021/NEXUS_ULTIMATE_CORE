// ============================================================================
// 🔱 NAVEGADOR SOBERANO — Gestor Unificado de Navegación, Red e Interacción Local
// ============================================================================
// Single Source of Truth para navegación headless, automatización e inyección
// de cookies. Consolida browser_native.rs y webclaw_extractor.rs.
// ============================================================================

use anyhow::{anyhow, Result};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use chromiumoxide::browser::BrowserConfig;
use chromiumoxide::handler::viewport::Viewport;
use chromiumoxide::{Browser, Page};
use futures::StreamExt;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::OnceLock;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tokio::time::sleep;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

// ── User-Agent pool realista ────────────────────────────────────────

const DESKTOP_USER_AGENTS: &[&str] = &[
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/132.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/132.0.0.0 Safari/537.36",
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:133.0) Gecko/20100101 Firefox/133.0",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 14.7; rv:133.0) Gecko/20100101 Firefox/133.0",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 14_7_2) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.4.1 Safari/605.1.15",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/132.0.0.0 Safari/537.36 Edg/132.0.0.0",
];

/// Elige un User-Agent desktop realista al azar.
pub fn random_user_agent() -> &'static str {
    let mut rng = rand::thread_rng();
    DESKTOP_USER_AGENTS
        .choose(&mut rng)
        .copied()
        .unwrap_or(DESKTOP_USER_AGENTS[0])
}

// ── Detección de navegador ──────────────────────────────────────────

/// Busca un binario de Brave/Chrome/Chromium en el sistema.
pub fn find_chrome_executable() -> Option<String> {
    // 1. Env override
    if let Ok(p) = std::env::var("CHROME_EXECUTABLE") {
        if Path::new(&p).exists() {
            return Some(p);
        }
    }

    // 2. PATH scan
    if let Ok(path_var) = std::env::var("PATH") {
        let candidates = [
            "brave-browser",
            "brave",
            "chromium-browser",
            "chromium",
            "chrome",
            "google-chrome",
        ];
        for dir in std::env::split_paths(&path_var) {
            for exe in &candidates {
                let full = dir.join(exe);
                if full.exists() {
                    return Some(full.to_string_lossy().to_string());
                }
            }
        }
    }

    // 3. Rutas específicas de Linux
    #[cfg(target_os = "linux")]
    {
        let candidates = [
            "/usr/bin/brave-browser",
            "/usr/bin/brave",
            "/usr/bin/chromium-browser",
            "/usr/bin/google-chrome",
            "/snap/bin/brave",
            "/snap/bin/chromium",
        ];
        for c in &candidates {
            if Path::new(c).exists() {
                return Some(c.to_string());
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        let candidates = [
            "/Applications/Brave Browser.app/Contents/MacOS/Brave Browser",
            "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
        ];
        for c in &candidates {
            if Path::new(c).exists() {
                return Some(c.to_string());
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        let candidates = [
            r"C:\Program Files\BraveSoftware\Brave-Browser\Application\brave.exe",
            r"C:\Program Files\Google\Chrome\Application\chrome.exe",
        ];
        for c in &candidates {
            if Path::new(c).exists() {
                return Some(c.to_string());
            }
        }
    }

    None
}

/// ¿Hay un navegador nativo instalado?
pub fn native_browser_available() -> bool {
    find_chrome_executable().is_some()
}

static BROWSER_LAUNCH_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

fn browser_launch_lock() -> &'static Mutex<()> {
    BROWSER_LAUNCH_LOCK.get_or_init(|| Mutex::new(()))
}

fn cleanup_chromiumoxide_runner_profile() {
    let runner_dir = std::env::temp_dir().join("chromiumoxide-runner");
    if !runner_dir.exists() {
        return;
    }
    let _ = std::fs::remove_file(runner_dir.join("SingletonLock"));
    let _ = std::fs::remove_file(runner_dir.join("SingletonSocket"));
    let _ = std::fs::remove_file(runner_dir.join("SingletonCookie"));
}

pub fn is_benign_cdp_disconnect(message: &str) -> bool {
    [
        "ResetWithoutClosingHandshake",
        "Connection reset by peer",
        "Broken pipe",
        "connection closed",
        "WebSocket protocol error: Connection reset without closing handshake",
    ]
    .iter()
    .any(|needle| message.contains(needle))
}

pub fn log_cdp_handler_error(context: &str, message: &str) {
    if is_benign_cdp_disconnect(message) {
        info!(
            "{}: browser connection closed during shutdown: {}",
            context, message
        );
    } else {
        warn!("{}: {}", context, message);
    }
}

pub async fn launch_browser_serialized(
    config: BrowserConfig,
    context: &str,
) -> Result<(Browser, chromiumoxide::Handler)> {
    let _guard = browser_launch_lock().lock().await;
    cleanup_chromiumoxide_runner_profile();
    match Browser::launch(config).await {
        Ok(result) => Ok(result),
        Err(e) => {
            cleanup_chromiumoxide_runner_profile();
            Err(anyhow!("{}: {}", context, e))
        }
    }
}

pub async fn shutdown_browser_session(
    browser: &mut Browser,
    handler_task: JoinHandle<()>,
    data_dir: PathBuf,
    context: &str,
) {
    match tokio::time::timeout(Duration::from_secs(3), browser.close()).await {
        Ok(Ok(_)) => {}
        Ok(Err(e)) => {
            let message = e.to_string();
            if !is_benign_cdp_disconnect(&message) {
                warn!("{}: browser close error (non-fatal): {}", context, message);
            }
        }
        Err(_) => {
            warn!("{}: timed out waiting for browser.close()", context);
        }
    }

    match tokio::time::timeout(Duration::from_secs(3), handler_task).await {
        Ok(Ok(())) => {}
        Ok(Err(join_err)) => {
            if !join_err.is_cancelled() {
                warn!("{}: handler task join error: {}", context, join_err);
            }
        }
        Err(_) => {
            warn!("{}: timed out waiting for handler task shutdown", context);
        }
    }

    let _ = tokio::fs::remove_dir_all(&data_dir).await;
}

pub fn build_headless_config(
    exe: &str,
    proxy_url: Option<&str>,
    width: u32,
    height: u32,
) -> Result<(BrowserConfig, PathBuf)> {
    let ua = random_user_agent();
    let data_dir = std::env::temp_dir().join(format!("nexus-browser-{}", Uuid::new_v4()));

    let mut builder = BrowserConfig::builder()
        .chrome_executable(exe)
        .viewport(Viewport {
            width,
            height,
            device_scale_factor: Some(1.0),
            emulating_mobile: false,
            is_landscape: true,
            has_touch: false,
        })
        .window_size(width, height)
        .arg("--disable-gpu")
        .arg("--no-sandbox")
        .arg("--disable-setuid-sandbox")
        .arg("--disable-dev-shm-usage")
        .arg("--disable-extensions")
        .arg("--disable-background-networking")
        .arg("--disable-sync")
        .arg("--disable-translate")
        .arg("--disable-crash-reporter")
        .arg("--disable-breakpad")
        .arg("--no-first-run")
        .arg("--no-default-browser-check")
        .arg("--hide-scrollbars")
        .arg("--mute-audio")
        .arg("--disable-blink-features=AutomationControlled")
        .arg("--password-store=basic")
        .arg("--use-mock-keyring")
        .arg(format!("--user-data-dir={}", data_dir.display()))
        .arg(format!("--user-agent={}", ua));

    if let Some(proxy) = proxy_url {
        builder = builder.arg(format!("--proxy-server={}", proxy));
    }

    let config = builder
        .build()
        .map_err(|e| anyhow!("Failed to build browser config: {}", e))?;
    Ok((config, data_dir))
}

// ── Browser Pool persistente ────────────────────────────────────────

pub struct BrowserPool {
    exe: String,
    inner: Mutex<Option<(Browser, PathBuf)>>,
}

impl BrowserPool {
    pub fn new(exe: impl Into<String>) -> Arc<Self> {
        Arc::new(Self {
            exe: exe.into(),
            inner: Mutex::new(None),
        })
    }

    pub fn new_auto() -> Option<Arc<Self>> {
        find_chrome_executable().map(Self::new)
    }

    pub async fn acquire(&self, proxy_url: Option<&str>) -> Result<Page> {
        let mut guard = self.inner.lock().await;

        let alive = match guard.as_mut() {
            Some((b, _)) => b.new_page("about:blank").await.is_ok(),
            None => false,
        };

        if !alive {
            if guard.is_some() {
                warn!("🔄 Browser pool: instance dead, restarting...");
                if let Some((mut old, old_dir)) = guard.take() {
                    let _ = old.close().await;
                    let _ = tokio::fs::remove_dir_all(&old_dir).await;
                }
            }
            info!("🚀 Browser pool: launching new instance ({})", self.exe);
            let (config, data_dir) = build_headless_config(&self.exe, proxy_url, 1920, 1080)?;
            let (new_browser, mut handler) = launch_browser_serialized(
                config,
                &format!("Pool: failed to launch ({})", self.exe),
            )
            .await?;
            tokio::spawn(async move {
                while let Some(event) = handler.next().await {
                    if let Err(e) = event {
                        log_cdp_handler_error("Pool CDP handler error", &e.to_string());
                    }
                }
            });
            *guard = Some((new_browser, data_dir));
        }

        let (b, _) = guard.as_mut().expect("browser present after init");
        b.new_page("about:blank")
            .await
            .map_err(|e| anyhow!("Pool: failed to open tab: {}", e))
    }

    pub async fn shutdown(&self) {
        let mut guard = self.inner.lock().await;
        if let Some((mut b, data_dir)) = guard.take() {
            let _ = b.close().await;
            let _ = tokio::fs::remove_dir_all(&data_dir).await;
            info!("🛑 Browser pool shut down");
        }
    }
}

impl Drop for BrowserPool {
    fn drop(&mut self) {
        let Ok(handle) = tokio::runtime::Handle::try_current() else {
            return;
        };
        if let Ok(mut guard) = self.inner.try_lock() {
            if let Some((mut browser, data_dir)) = guard.take() {
                handle.spawn(async move {
                    let _ = browser.close().await;
                    let _ = tokio::fs::remove_dir_all(&data_dir).await;
                });
            }
        }
    }
}

// ── Ad-block patterns ───────────────────────────────────────────────

const AD_BLOCK_PATTERNS: &[&str] = &[
    "doubleclick.net",
    "googlesyndication.com",
    "googletagmanager.com",
    "googletagservices.com",
    "adservice.google.",
    "amazon-adsystem.com",
    "ads.twitter.com",
    "ads.linkedin.com",
    "advertising.com",
    "criteo.com",
    "taboola.com",
    "outbrain.com",
    "moatads.com",
    "adnxs.com",
    "google-analytics.com",
    "analytics.google.com",
    "segment.com/v1/t",
    "segment.io/v1",
    "mixpanel.com/track",
    "hotjar.com",
    "mouseflow.com",
    "fullstory.com",
    "newrelic.com/",
    "nr-data.net",
    "connect.facebook.net",
    "platform.twitter.com/widgets",
    "cookielaw.org",
    "cookiebot.com",
    "onetrust.com",
];

pub fn should_block_url(url: &str, block_images: bool) -> bool {
    let lower = url.to_lowercase();
    if AD_BLOCK_PATTERNS.iter().any(|pat| lower.contains(pat)) {
        return true;
    }
    if block_images {
        let lower = url.to_lowercase();
        for ext in &[
            ".jpg", ".jpeg", ".png", ".gif", ".webp", ".svg", ".ico", ".mp4", ".webm", ".ogg",
            ".mp3", ".woff", ".woff2", ".ttf",
        ] {
            if lower.contains(ext) {
                return true;
            }
        }
    }
    return false;
}

pub fn should_block_resource_type(resource_type: &str) -> bool {
    ["media", "font"]
        .iter()
        .any(|t| resource_type.eq_ignore_ascii_case(t))
}

pub async fn wait_until_stable(page: &Page, quiet_ms: u64, timeout_ms: u64) -> Result<()> {
    let poll_ms = 250u64;
    let start = std::time::Instant::now();
    let mut last_count: u64 = 0;
    let mut stable_since = std::time::Instant::now();

    loop {
        if start.elapsed().as_millis() as u64 >= timeout_ms {
            debug!("wait_until_stable: timeout after {}ms", timeout_ms);
            break;
        }

        let count: u64 = page
            .evaluate("performance.getEntriesByType('resource').length")
            .await
            .ok()
            .and_then(|v| v.into_value::<serde_json::Value>().ok())
            .and_then(|j| j.as_u64())
            .unwrap_or(0);

        let ready_complete: bool = page
            .evaluate("document.readyState")
            .await
            .ok()
            .and_then(|v| v.into_value::<serde_json::Value>().ok())
            .and_then(|j| j.as_str().map(|s| s == "complete"))
            .unwrap_or(false);

        if !ready_complete {
            stable_since = std::time::Instant::now();
            last_count = count;
        } else if count != last_count {
            last_count = count;
            stable_since = std::time::Instant::now();
        } else if stable_since.elapsed().as_millis() as u64 >= quiet_ms {
            debug!(
                "wait_until_stable: idle after {}ms ({} resources)",
                start.elapsed().as_millis(),
                count
            );
            break;
        }

        tokio::time::sleep(Duration::from_millis(poll_ms)).await;
    }
    Ok(())
}

pub async fn auto_scroll(page: &Page) -> Result<()> {
    let height: u64 = page
        .evaluate(
            "() => Math.max(document.body.scrollHeight, document.documentElement.scrollHeight)",
        )
        .await
        .ok()
        .and_then(|v| v.into_value::<serde_json::Value>().ok())
        .and_then(|j| j.as_u64())
        .unwrap_or(3000);

    let step = 600u64;
    let steps = (height / step).min(20);
    for i in 0..=steps {
        let y = i * step;
        if let Err(e) = page
            .evaluate(format!(
                "window.scrollTo({{top: {y}, behavior: 'smooth'}});"
            ))
            .await
        {
            warn!("auto_scroll: step {} error: {}", i, e);
        }
        tokio::time::sleep(Duration::from_millis(250)).await;
    }
    Ok(())
}

pub async fn fetch_html_native(url: &str, wait_ms: Option<u32>) -> Result<(u16, String)> {
    let exe = find_chrome_executable()
        .ok_or_else(|| anyhow!("No browser found. Install Brave, Chrome, or Chromium."))?;

    info!("🌐 Native headless fetch: {} (browser: {})", url, exe);
    let wait_time = wait_ms.unwrap_or(2000) as u64;

    let (config, data_dir) = build_headless_config(&exe, None, 1280, 900)?;
    let (mut browser, mut handler) =
        launch_browser_serialized(config, &format!("Failed to launch browser ({})", exe)).await?;

    let _handle = tokio::spawn(async move {
        while let Some(event) = handler.next().await {
            if let Err(e) = event {
                log_cdp_handler_error("CDP handler error", &e.to_string());
            }
        }
    });

    let result: Result<(u16, String)> = async {
        let page = browser
            .new_page(url)
            .await
            .map_err(|e| anyhow!("Failed to open page: {}", e))?;

        tokio::time::sleep(Duration::from_millis(wait_time)).await;

        let html = page
            .content()
            .await
            .map_err(|e| anyhow!("Failed to get page content: {}", e))?;

        info!("✅ Native fetch: {} chars ({}ms)", html.len(), wait_time);
        Ok((200u16, html))
    }
    .await;

    shutdown_browser_session(&mut browser, _handle, data_dir, "fetch_html_native").await;
    result
}

pub async fn fetch_html_native_mobile(url: &str, wait_ms: Option<u32>) -> Result<(u16, String)> {
    let exe = find_chrome_executable()
        .ok_or_else(|| anyhow!("No browser found for mobile fetch fallback"))?;

    let wait_time = wait_ms.unwrap_or(2500) as u64;
    let mobile_data_dir = std::env::temp_dir().join(format!("nexus-mobile-{}", Uuid::new_v4()));

    let config = BrowserConfig::builder()
        .chrome_executable(&exe)
        .viewport(Viewport {
            width: 390,
            height: 844,
            device_scale_factor: Some(3.0),
            emulating_mobile: true,
            is_landscape: false,
            has_touch: true,
        })
        .window_size(390, 844)
        .arg("--disable-gpu")
        .arg("--no-sandbox")
        .arg("--disable-dev-shm-usage")
        .arg("--no-first-run")
        .arg("--disable-blink-features=AutomationControlled")
        .arg("--password-store=basic")
        .arg("--use-mock-keyring")
        .arg(format!("--user-data-dir={}", mobile_data_dir.display()))
        .arg("--user-agent=Mozilla/5.0 (iPhone; CPU iPhone OS 17_4 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.4 Mobile/15E148 Safari/604.1")
        .build()
        .map_err(|e| anyhow!("Mobile browser config error: {}", e))?;

    let (mut browser, mut handler) =
        launch_browser_serialized(config, "Failed to launch mobile browser").await?;

    let _handle = tokio::spawn(async move {
        while let Some(event) = handler.next().await {
            if let Err(e) = event {
                log_cdp_handler_error("CDP mobile handler error", &e.to_string());
            }
        }
    });

    let result: Result<(u16, String)> = async {
        let page = browser
            .new_page(url)
            .await
            .map_err(|e| anyhow!("Mobile page navigation failed: {}", e))?;

        wait_until_stable(&page, wait_time.min(3000), wait_time + 5000).await?;

        let html = page
            .content()
            .await
            .map_err(|e| anyhow!("Failed to get mobile page content: {}", e))?;

        Ok((200u16, html))
    }
    .await;

    shutdown_browser_session(
        &mut browser,
        _handle,
        mobile_data_dir,
        "fetch_html_native_mobile",
    )
    .await;
    result
}

// ── WebClawExtractor unificado ──────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GeminiCookies {
    pub cookies_json: String,
    pub fecha_extraccion: String,
}

pub struct WebClawExtractor {
    cookies_path: PathBuf,
    pool: Arc<BrowserPool>,
}

impl WebClawExtractor {
    pub fn new() -> Result<Self> {
        let cookies_path = crate::infra::paths::resolve_path("data/gemini_cookies.json");
        let pool = BrowserPool::new_auto()
            .ok_or_else(|| anyhow!("No hay navegador nativo instalado (Brave/Chrome/Chromium)."))?;
        info!("🦾 [NAVEGADOR-SOBERANO] Extractor unificado listo.");
        Ok(Self { cookies_path, pool })
    }

    fn cargar_cookies(&self) -> Option<serde_json::Value> {
        if !self.cookies_path.exists() {
            warn!("🍪 No se encontraron cookies.");
            return None;
        }
        match fs::read_to_string(&self.cookies_path) {
            Ok(json_str) => match serde_json::from_str::<GeminiCookies>(&json_str) {
                Ok(gc) => match serde_json::from_str::<serde_json::Value>(&gc.cookies_json) {
                    Ok(cookies) => {
                        info!("🍪 Cookies cargadas.");
                        Some(cookies)
                    }
                    Err(e) => {
                        warn!("🍪 Error parseando: {}", e);
                        None
                    }
                },
                Err(e) => {
                    warn!("🍪 Error parseando archivo: {}", e);
                    None
                }
            },
            Err(e) => {
                warn!("🍪 Error leyendo archivo: {}", e);
                None
            }
        }
    }

    async fn inject_gemini_session(
        page: &chromiumoxide::Page,
        cookies: &serde_json::Value,
    ) -> Result<()> {
        use chromiumoxide::cdp::browser_protocol::network::{CookieParam, SetCookiesParams};

        let mut raw_cookies = Vec::new();
        if let Some(obj) = cookies.as_object() {
            for (nombre, valor) in obj {
                if let Some(v) = valor.as_str() {
                    raw_cookies.push(serde_json::json!({
                        "name": nombre,
                        "value": v,
                        "domain": ".google.com",
                        "path": "/",
                        "secure": true,
                        "httpOnly": true
                    }));
                }
            }
        }

        let cookie_params: Vec<CookieParam> = raw_cookies
            .iter()
            .filter_map(|v| serde_json::from_value::<CookieParam>(v.clone()).ok())
            .collect();

        if cookie_params.is_empty() {
            return Err(anyhow!("No valid CookieParams constructed"));
        }

        let count = cookie_params.len();
        page.execute(SetCookiesParams::new(cookie_params)).await?;
        info!("🍪 Inyectadas {} cookies Gemini vía CDP.", count);

        sleep(Duration::from_millis(500)).await;
        Ok(())
    }

    async fn type_human_like(element: &chromiumoxide::Element, text: &str) -> Result<()> {
        for c in text.chars() {
            let delay = rand::random::<u64>() % 40 + 15;
            element.type_str(c.to_string()).await?;
            sleep(Duration::from_millis(delay)).await;
        }
        Ok(())
    }

    pub async fn extraer_respuesta(&self, prompt: &str) -> Result<String> {
        let cookies = self
            .cargar_cookies()
            .ok_or_else(|| anyhow!("No hay cookies."))?;

        let page = self.pool.acquire(None).await?;
        let mut texto = String::new();

        let extraction_result = async {
            page.evaluate_on_new_document(crate::defensa::camuflaje_omega::STEALTH_PAYLOAD).await?;
            Self::inject_gemini_session(&page, &cookies).await?;
            page.goto("https://gemini.google.com/app").await?;
            sleep(Duration::from_secs(4)).await;

            let current_url = page.url().await?.unwrap_or_default();
            let title: String = page.evaluate("document.title").await?.into_value().unwrap_or_default();
            info!("🌐 [DIAG] URL actual: {}", current_url);
            info!("🌐 [DIAG] Título: {}", title);

            if current_url.contains("signin") || current_url.contains("accounts.google") {
                warn!("⚠️ [DIAG] Redireccionado a login. Cookies expiradas.");
            }

            info!("⌨️ Escribiendo prompt como humano...");
            if let Ok(el) = page.find_element("rich-textarea").await {
                el.click().await?;
                sleep(Duration::from_millis(300)).await;
                Self::type_human_like(&el, prompt).await?;
            } else {
                warn!("⚠️ No se encontró 'rich-textarea'.");
                return Err(anyhow!("No se encontró 'rich-textarea'"));
            }

            sleep(Duration::from_millis(300)).await;

            if let Ok(btn) = page.find_element("button.send-button").await {
                info!("🖱️ Clic en send-button...");
                btn.click().await?;
            } else if let Ok(btn) = page.find_element("rich-textarea button").await {
                info!("🖱️ Clic en textarea button...");
                btn.click().await?;
            } else {
                info!("⌨️ Enviando vía Enter...");
                let _ = page.evaluate(
                    "document.activeElement.dispatchEvent(new KeyboardEvent('keydown', {key: 'Enter', code: 'Enter', keyCode: 13, bubbles: true}));",
                ).await?;
            }

            info!("⏳ Esperando respuesta...");
            sleep(Duration::from_secs(5)).await;

            for _ in 0..10 {
                if let Ok(el) = page.find_element("message-content").await {
                    if let Ok(Some(t)) = el.inner_text().await {
                        if t.len() > 20 && !t.contains("...") {
                            texto = t;
                            break;
                        }
                    }
                }
                sleep(Duration::from_millis(500)).await;
            }

            if texto.is_empty() {
                let final_url = page.url().await?.unwrap_or_default();
                let final_title: String = page.evaluate("document.title").await?.into_value().unwrap_or_default();
                error!("❌ [DIAG] Extracción fallida. URL: {}, Título: {}", final_url, final_title);
                if let Ok(screenshot_data) = page.screenshot(
                    chromiumoxide::cdp::browser_protocol::page::CaptureScreenshotParams::default(),
                ).await {
                    let ss_path = crate::infra::paths::resolve_path("data/silent_extractor_failed.png");
                    if fs::write(&ss_path, screenshot_data).is_ok() {
                        warn!("📸 Screenshot guardada en: {:?}", ss_path);
                    }
                }
                return Err(anyhow!("No se pudo extraer respuesta."));
            }

            let _ = page.close().await;
            Ok(texto)
        }.await;

        match &extraction_result {
            Ok(ref resp) => debug!("✅ Respuesta extraída ({} chars).", resp.len()),
            Err(ref e) => error!("❌ Error en extracción: {}", e),
        }

        extraction_result
    }
}
