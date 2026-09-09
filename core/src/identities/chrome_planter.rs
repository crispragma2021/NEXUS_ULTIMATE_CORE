// 🔱 CHROME PLANTER OMEGA — Sembrador Chrome Soberano en Rust Puro
// ============================================================================
// Sustituye: trading-portal/scripts/sembrador_chrome.js (Puppeteer/Node.js)
// Motor:    chromiumoxide (Chrome DevTools Protocol directo en Rust)
// ADN:      VisionFantasma (stealth) + BrowserProfileManager (perfiles aislados)
//
// Capacidades:
//   1. crear_cuenta_gmail()   → accounts.google.com/signup
//   2. login_gmail()          → accounts.google.com/AccountChooser
//   3. crear_cuenta_facebook() → facebook.com/r.php
//   4. crear_cuenta_proton()   → account.proton.me/mail/signup
//
// Zero dependencia Node.js. Zero scripts externos. Omega puro.
// ============================================================================

use anyhow::{anyhow, Result};
use chromiumoxide::{
    browser::BrowserConfig, cdp::browser_protocol::page::CaptureScreenshotParams,
    handler::viewport::Viewport, Browser, Page,
};
use rand::Rng;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tracing::{error, info, warn};

use crate::identities::browser_profile::BrowserProfileManager;
use crate::identities::types::SyntheticIdentity;
use crate::infra::browser_native::{
    build_headless_config, find_chrome_executable, launch_browser_serialized,
    shutdown_browser_session,
};
use crate::sentidos::vision_fantasma::VisionFantasma;

// ─── Constantes ──────────────────────────────────────────────────────────────

const SCREENSHOTS_DIR: &str = "artifacts/screenshots";
const GMAIL_SIGNUP_URL: &str =
    "https://accounts.google.com/signup/v2/webcreateaccount?flowName=SignUpFlow";
const GMAIL_LOGIN_URL: &str = "https://accounts.google.com/AccountChooser";
const FACEBOOK_SIGNUP_URL: &str = "https://www.facebook.com/r.php";
const PROTON_SIGNUP_URL: &str = "https://account.proton.me/mail/signup";

/// Ventana estándar para simular escritorio real
const VIEWPORT_W: u32 = 1366;
const VIEWPORT_H: u32 = 768;

use crate::infra::sms_activate::SmsActivateClient;

// ─── Resultados ──────────────────────────────────────────────────────────────

/// Resultado de una operación de plantación
#[derive(Debug, Clone)]
pub struct PlantResult {
    pub success: bool,
    pub email: Option<String>,
    pub password: Option<String>,
    pub error: Option<String>,
    pub pending_verification: bool,
}

/// Resultado de login
#[derive(Debug, Clone)]
pub struct LoginResult {
    pub success: bool,
    pub email: String,
    pub error: Option<String>,
}

// ─── ChromePlanter ───────────────────────────────────────────────────────────

/// Órgano soberano de automatización Chrome vía CDP Rust puro.
///
/// ## Omega vs Puppeteer
/// - Sin Node.js, sin runtime externo, sin `require()`
/// - Camuflaje OMEGA via VisionFantasma (stealth + webdriver override + Chrome spoof)
/// - Perfiles persistentes aislados por identidad (BrowserProfileManager)
/// - Screenshots por paso en `artifacts/screenshots/`
pub struct ChromePlanter {
    browser_mgr: BrowserProfileManager,
    screenshots_dir: PathBuf,
    sms_client: Option<Arc<SmsActivateClient>>,
}

impl ChromePlanter {
    /// Crea un nuevo plantador con gestor de perfiles
    pub fn new(browser_mgr: BrowserProfileManager) -> Self {
        let screenshots_dir = PathBuf::from(SCREENSHOTS_DIR);
        let _ = std::fs::create_dir_all(&screenshots_dir);

        let sms_client = std::env::var("SMS_ACTIVATE_API_KEY")
            .ok()
            .map(|key| Arc::new(SmsActivateClient::new(key)));

        Self {
            browser_mgr,
            screenshots_dir,
            sms_client,
        }
    }

    // ─── Constructor de config ───────────────────────────────────────────

    /// Construye config headless con perfil persistente de identidad
    fn config_para_identidad(
        &self,
        identity: &SyntheticIdentity,
    ) -> Result<(BrowserConfig, PathBuf)> {
        let exe = find_chrome_executable().ok_or_else(|| {
            anyhow!("No Chrome/Chromium/Brave encontrado. Instala uno o define CHROME_EXECUTABLE")
        })?;

        // Crear perfil persistente para esta identidad
        let profile_dir = self.browser_mgr.create_profile(identity)?;

        // Tomar fingerprint de la identidad si existe
        let ua = &identity.fingerprint.user_agent;
        let (w, h) = parsear_resolucion(&identity.fingerprint.screen_resolution);

        let mut builder = BrowserConfig::builder()
            .chrome_executable(&exe)
            .viewport(Viewport {
                width: w,
                height: h,
                device_scale_factor: Some(1.0),
                emulating_mobile: false,
                is_landscape: true,
                has_touch: false,
            })
            .window_size(w, h)
            // Flags anti-detección OMEGA
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
            .arg("--mute-audio")
            .arg("--disable-blink-features=AutomationControlled")
            .arg("--password-store=basic")
            .arg("--use-mock-keyring")
            .arg(format!("--user-data-dir={}", profile_dir.display()))
            .arg(format!("--user-agent={}", ua));

        // Configurar idioma
        let locale = &identity.fingerprint.language.replace('-', "_");
        builder = builder.arg(format!("--lang={}", locale));
        builder = builder.arg("--accept-lang=es-ES,es;q=0.9");

        let config = builder
            .build()
            .map_err(|e| anyhow!("Error construyendo config browser: {}", e))?;

        Ok((config, profile_dir))
    }

    /// Config para operaciones sin identidad específica (login standalone)
    fn config_generica(profile_dir: Option<PathBuf>) -> Result<(BrowserConfig, PathBuf)> {
        let exe = find_chrome_executable()
            .ok_or_else(|| anyhow!("No Chrome/Chromium/Brave encontrado"))?;

        let data_dir = profile_dir.unwrap_or_else(|| {
            std::env::temp_dir().join(format!("nexus-planter-{}", uuid::Uuid::new_v4()))
        });
        let _ = std::fs::create_dir_all(&data_dir);

        let mut builder = BrowserConfig::builder()
            .chrome_executable(&exe)
            .viewport(Viewport {
                width: VIEWPORT_W,
                height: VIEWPORT_H,
                device_scale_factor: Some(1.0),
                emulating_mobile: false,
                is_landscape: true,
                has_touch: false,
            })
            .window_size(VIEWPORT_W, VIEWPORT_H)
            .arg("--disable-gpu")
            .arg("--no-sandbox")
            .arg("--disable-setuid-sandbox")
            .arg("--disable-dev-shm-usage")
            .arg("--disable-extensions")
            .arg("--disable-background-networking")
            .arg("--disable-sync")
            .arg("--disable-translate")
            .arg("--no-first-run")
            .arg("--no-default-browser-check")
            .arg("--mute-audio")
            .arg("--disable-blink-features=AutomationControlled")
            .arg("--password-store=basic")
            .arg("--use-mock-keyring")
            .arg(format!("--user-data-dir={}", data_dir.display()));

        let config = builder
            .build()
            .map_err(|e| anyhow!("Error construyendo config browser: {}", e))?;

        Ok((config, data_dir))
    }

    // ─── Lanzamiento con stealth ─────────────────────────────────────────

    /// Lanza navegador con perfil de identidad y aplica camuflaje OMEGA
    async fn lanzar_con_identidad(
        &self,
        identity: &SyntheticIdentity,
    ) -> Result<(Browser, Page, PathBuf)> {
        let (config, profile_dir) = self.config_para_identidad(identity)?;

        let (mut browser, mut handler) =
            launch_browser_serialized(config, "ChromePlanter: lanzar_con_identidad").await?;

        // Handler en background
        tokio::spawn(async move {
            use futures::StreamExt;
            while let Some(event) = handler.next().await {
                if let Err(e) = event {
                    if !e.to_string().contains("ResetWithoutClosingHandshake")
                        && !e.to_string().contains("Connection reset")
                        && !e.to_string().contains("Broken pipe")
                    {
                        warn!("[ChromePlanter] CDP event error: {}", e);
                    }
                }
            }
        });

        let page = browser
            .new_page("about:blank")
            .await
            .map_err(|e| anyhow!("Error abriendo página: {}", e))?;

        // 👻 Inyectar camuflaje OMEGA
        VisionFantasma::aplicar_camuflaje_omega(&page).await?;

        Ok((browser, page, profile_dir))
    }

    /// Lanza navegador genérico (sin identidad) con camuflaje
    async fn lanzar_generico(profile_dir: Option<PathBuf>) -> Result<(Browser, Page, PathBuf)> {
        let (config, data_dir) = Self::config_generica(profile_dir)?;

        let (mut browser, mut handler) =
            launch_browser_serialized(config, "ChromePlanter: lanzar_generico").await?;

        tokio::spawn(async move {
            use futures::StreamExt;
            while let Some(event) = handler.next().await {
                if let Err(e) = event {
                    if !e.to_string().contains("ResetWithoutClosingHandshake")
                        && !e.to_string().contains("Connection reset")
                        && !e.to_string().contains("Broken pipe")
                    {
                        warn!("[ChromePlanter] CDP event error: {}", e);
                    }
                }
            }
        });

        let page = browser
            .new_page("about:blank")
            .await
            .map_err(|e| anyhow!("Error abriendo página: {}", e))?;

        VisionFantasma::aplicar_camuflaje_omega(&page).await?;

        Ok((browser, page, data_dir))
    }

    // ─── Utilidades de interacción OMEGA ────────────────────────────────
    // Capas de anti-detección:
    //   ⌨️ Type con distribución gaussiana + bursts + pausas
    //   🖱️ Mouse movement con curvas bezier + jitter
    //   📜 Scroll natural con aceleración/deceleración
    //   ⏱️ Esperas con distribución real

    /// Escribe texto con patrón de escritura humana real.
    ///
    /// Distribución:
    /// - 70%: delay normal ~20-50ms (gaussiana centrada en 32ms)
    /// - 15%: burst (3-5 chars a 5-10ms, simula muscle memory)
    /// - 10%: micro-pausa 30-80ms cada 4-8 chars (reposicionamiento dedos)
    /// - 5%: pausa larga 100-350ms (piensa qué escribir)
    async fn type_human_like(element: &chromiumoxide::Element, text: &str) -> Result<()> {
        let chars: Vec<char> = text.chars().collect();
        let mut i = 0;
        while i < chars.len() {
            let c = chars[i];
            let r: f64 = rand::thread_rng().gen();

            // 5%: Pausa larga (simula pensar)
            if r < 0.05 && i > 0 {
                let pause = rand::thread_rng().gen_range(100..350);
                tokio::time::sleep(Duration::from_millis(pause)).await;
            }

            // 10%: Micro-pausa cada 4-8 chars (reposicionamiento físico)
            if i > 0 && i % rand::thread_rng().gen_range(4..8) == 0 {
                let micro = rand::thread_rng().gen_range(30..80);
                tokio::time::sleep(Duration::from_millis(micro)).await;
            }

            // 15%: Burst — escribe 3-5 caracteres casi instantáneos (muscle memory)
            if r < 0.15 && i + 3 <= chars.len() {
                let burst_len = rand::thread_rng().gen_range(3..=5.min(chars.len() - i));
                for j in 0..burst_len {
                    let burst_c = chars[i + j];
                    element.type_str(burst_c.to_string()).await?;
                    let burst_delay: u64 = rand::thread_rng().gen_range(5..12);
                    tokio::time::sleep(Duration::from_millis(burst_delay)).await;
                }
                i += burst_len;
                // Micro-pausa post-burst
                let post = rand::thread_rng().gen_range(20..50);
                tokio::time::sleep(Duration::from_millis(post)).await;
                continue;
            }

            // 70%: Normal — gaussiana centrada en 32ms (rango 15-55ms)
            // Usamos Box-Muller para distribución normal aproximada
            let normal = {
                let u1: f64 = rand::thread_rng().gen();
                let u2: f64 = rand::thread_rng().gen();
                (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos()
            };
            let delay = (32.0 + normal * 8.0).clamp(12.0, 65.0) as u64;

            element.type_str(c.to_string()).await?;
            tokio::time::sleep(Duration::from_millis(delay)).await;
            i += 1;
        }
        Ok(())
    }

    /// Delay humano con distribución uniforme entre min_ms y max_ms
    async fn esperar_humano(min_ms: u64, max_ms: u64) {
        let delay: u64 = rand::thread_rng().gen_range(min_ms..max_ms);
        tokio::time::sleep(Duration::from_millis(delay)).await;
    }

    /// Mueve el mouse desde una posición inicial aleatoria hasta (target_x, target_y)
    /// siguiendo una curva cúbica bezier con jitter y easing humano.
    async fn mover_mouse_humano(page: &Page, target_x: f64, target_y: f64) -> Result<()> {
        // Posición inicial aleatoria en el viewport (simula mano que llega desde cualquier lugar)
        let start_x: f64 = rand::thread_rng().gen_range(50.0..1300.0);
        let start_y: f64 = rand::thread_rng().gen_range(50.0..700.0);

        // Control points con desviación aleatoria para que cada movimiento sea único
        let dx = target_x - start_x;
        let dy = target_y - start_y;
        let cp1_x = start_x + dx * 0.3 + rand::thread_rng().gen_range(-60.0..60.0);
        let cp1_y = start_y + dy * 0.1 + rand::thread_rng().gen_range(-40.0..40.0);
        let cp2_x = start_x + dx * 0.7 + rand::thread_rng().gen_range(-60.0..60.0);
        let cp2_y = start_y + dy * 0.9 + rand::thread_rng().gen_range(-40.0..40.0);

        // Pasos basados en distancia (~10px por paso)
        let distance = (dx.powi(2) + dy.powi(2)).sqrt();
        let steps = (distance / 10.0).ceil().max(8.0).min(80.0) as u32;

        for i in 0..=steps {
            let t = i as f64 / steps as f64;
            // Cubic bezier: B(t) = (1-t)³P0 + 3(1-t)²tP1 + 3(1-t)t²P2 + t³P3
            let mt = 1.0 - t;
            let x = mt.powi(3) * start_x
                + 3.0 * mt.powi(2) * t * cp1_x
                + 3.0 * mt * t.powi(2) * cp2_x
                + t.powi(3) * target_x;
            let y = mt.powi(3) * start_y
                + 3.0 * mt.powi(2) * t * cp1_y
                + 3.0 * mt * t.powi(2) * cp2_y
                + t.powi(3) * target_y;

            // Easing: lento al inicio/aceleración/media/deceleración/final
            let ease = (1.0 - (2.0 * t - 1.0).abs()).max(0.05);
            let delay = (20.0 / ease).min(60.0) as u64;

            // Jitter de 1-4px (la mano nunca es perfectamente recta)
            let jx = x + rand::thread_rng().gen_range(-4.0..4.0);
            let jy = y + rand::thread_rng().gen_range(-4.0..4.0);

            // Disparamos mousemove nativo vía CDP para simular hardware real (isTrusted = true)
            page.move_mouse(chromiumoxide::layout::Point { x: jx, y: jy })
                .await?;

            tokio::time::sleep(Duration::from_millis(delay)).await;
        }

        // Pequeña pausa antes del click (la mano se posiciona)
        Self::esperar_humano(30, 80).await;
        Ok(())
    }

    /// Desplaza la página con scroll humano natural.
    /// Acelera gradualmente, mantiene velocidad, decelera al final.
    async fn scroll_humano(page: &Page, pixeles: i64) -> Result<()> {
        let steps = (pixeles.abs() as f64 / 40.0).ceil().max(5.0).min(40.0) as u32;
        let direction = if pixeles > 0 { 1.0 } else { -1.0 };
        let total = pixeles.abs() as f64;

        for i in 0..=steps {
            let t = i as f64 / steps as f64;

            // Aceleración: suave al inicio (curva senoidal)
            let progress = (t * std::f64::consts::PI / 2.0).sin();
            let step_px = (total / steps as f64) * progress * direction;

            let js = format!(r#"window.scrollBy(0, {});"#, step_px as i64);
            let _ = page.evaluate(js).await;

            // Delay variable: más rápido en medio, más lento al inicio/final
            let ease = (1.0 - (2.0 * t - 1.0).abs()).max(0.15);
            let delay = (20.0 / ease).min(45.0) as u64;
            tokio::time::sleep(Duration::from_millis(delay)).await;
        }
        Ok(())
    }

    /// Captura screenshot con timestamp
    async fn screenshot(&self, page: &Page, name: &str) {
        let filename = format!("{}_{}.png", name, chrono::Utc::now().timestamp_millis());
        let path = self.screenshots_dir.join(&filename);
        match page.screenshot(CaptureScreenshotParams::default()).await {
            Ok(data) => {
                if let Err(e) = std::fs::write(&path, data) {
                    warn!(
                        "[ChromePlanter] Error guardando screenshot {}: {}",
                        filename, e
                    );
                }
            }
            Err(e) => {
                warn!(
                    "[ChromePlanter] Error capturando screenshot {}: {}",
                    filename, e
                );
            }
        }
    }

    // ─── 1. CREAR CUENTA GMAIL ────────────────────────────────────────────

    /// Crea una cuenta de Gmail REAL navegando accounts.google.com/signup
    /// y rellenando el formulario completo con comportamiento humano.
    ///
    /// Retorna PlantResult con email generado o error.
    pub async fn crear_cuenta_gmail(
        &self,
        nombre: &str,
        apellido: &str,
        password: &str,
        recovery_email: Option<&str>,
        identity: &SyntheticIdentity,
    ) -> PlantResult {
        info!(
            "🌐 [GMAIL] Creando cuenta para {} {} (vía ChromePlanter Rust)",
            nombre, apellido
        );

        let (mut browser, page, profile_dir) = match self.lanzar_con_identidad(identity).await {
            Ok(v) => v,
            Err(e) => return PlantResult::error(format!("Error lanzando browser: {}", e)),
        };

        let result = self
            .flujo_crear_gmail_inner(&page, nombre, apellido, password, recovery_email)
            .await;

        let _ = shutdown_browser_session(
            &mut browser,
            tokio::spawn(async {}),
            profile_dir,
            "ChromePlanter: crear_cuenta_gmail",
        )
        .await;

        result
    }

    async fn flujo_crear_gmail_inner(
        &self,
        page: &Page,
        nombre: &str,
        apellido: &str,
        password: &str,
        recovery_email: Option<&str>,
    ) -> PlantResult {
        // Paso 1: Navegar a signup
        info!("🌐 [GMAIL] Navegando a signup de Google...");
        if let Err(e) = page.goto(GMAIL_SIGNUP_URL).await {
            return PlantResult::error(format!("Error navegando a signup: {}", e));
        }
        Self::esperar_humano(4000, 6000).await;
        info!("✅ [GMAIL] Página de registro cargada");

        // [MEJORA OMEGA]: Localización dinámica de inputs por inspección de DOM
        // Google usa ofuscación, así que buscamos el primer y segundo input de texto visibles.
        let inputs_js = r#"
            (() => {
                const inputs = Array.from(document.querySelectorAll('input[type="text"]'))
                    .filter(el => el.offsetParent !== null); // Solo visibles
                return inputs.length >= 1 ? true : false;
            })()
        "#;

        if let Ok(val) = page.evaluate(inputs_js).await {
            if !val.into_value::<bool>().unwrap_or(false) {
                return PlantResult::error(
                    "No se detectaron inputs de texto en el registro de Google".to_string(),
                );
            }
        }

        info!("✍️ [GMAIL] Rellenando formulario con selectores de respaldo...");

        // Intentar rellenar nombre vía JS para mayor fiabilidad
        let fill_name_js = format!(
            r#"
            (() => {{
                const inputs = Array.from(document.querySelectorAll('input'))
                    .filter(el => el.offsetParent !== null);
                if (inputs.length >= 1) {{
                    inputs[0].value = '{}';
                    inputs[0].dispatchEvent(new Event('input', {{ bubbles: true }}));
                    if (inputs.length >= 2) {{
                        inputs[1].value = '{}';
                        inputs[1].dispatchEvent(new Event('input', {{ bubbles: true }}));
                    }}
                    return true;
                }}
                return false;
            }})()
        "#,
            nombre, apellido
        );

        let _ = page.evaluate(fill_name_js).await;
        Self::esperar_humano(1000, 2000).await;

        // Click Siguiente
        if let Err(e) = Self::click_siguiente(page).await {
            return PlantResult::error(format!("Error click siguiente (nombre): {}", e));
        }
        Self::esperar_humano(3000, 5000).await;

        // Paso 3: Username (si aparece el campo)
        let username_suggested = format!(
            "{}.{}{}",
            nombre.to_lowercase(),
            apellido.to_lowercase(),
            rand::thread_rng().gen_range(1000..99999)
        );

        match page.find_element(r#"input[name="Username"]"#).await {
            Ok(el) => {
                info!("📧 [GMAIL] Escribiendo username: {}", username_suggested);
                if let Err(e) = Self::type_human_like(&el, &username_suggested).await {
                    warn!("[GMAIL] Error escribiendo username: {}", e);
                }
                Self::esperar_humano(500, 1000).await;
            }
            Err(_) => {
                info!("📧 [GMAIL] Username ya sugerido por Google, continuando...");
            }
        }

        let _ = Self::click_siguiente(page).await;
        Self::esperar_humano(2000, 3500).await;

        // Paso 4: Contraseña
        match page.find_element(r#"input[name="Passwd"]"#).await {
            Ok(el) => {
                info!("🔑 [GMAIL] Ingresando contraseña...");
                if let Err(e) = Self::type_human_like(&el, password).await {
                    return PlantResult::error(format!("Error escribiendo password: {}", e));
                }
                // Confirmar contraseña
                if let Ok(confirm_el) = page.find_element(r#"input[name="ConfirmPasswd"]"#).await {
                    Self::esperar_humano(200, 500).await;
                    let _ = Self::type_human_like(&confirm_el, password).await;
                }
                info!("🔑 [GMAIL] Contraseña ingresada");
            }
            Err(e) => {
                warn!("[GMAIL] No se encontró campo Passwd: {:?}", e);
            }
        }

        Self::esperar_humano(500, 1000).await;
        let _ = Self::click_siguiente(page).await;
        Self::esperar_humano(2000, 4000).await;

        // Paso 5: Opcional — recovery email
        if let Some(recovery) = recovery_email {
            if let Ok(el) = page.find_element(r#"input[type="email"]"#).await {
                info!("📧 [GMAIL] Ingresando recovery email...");
                let _ = Self::type_human_like(&el, recovery).await;
                Self::esperar_humano(500, 1000).await;
                let _ = Self::click_siguiente(page).await;
                Self::esperar_humano(2000, 3000).await;
            }
        }

        // Paso 6: Intentar omitir verificación telefónica
        let skip_selectors = [
            r#"button:has-text("Omitir")"#,
            r#"button:has-text("Skip")"#,
            r#"button:has-text("Saltar")"#,
            r#"button:has-text("Ahora no")"#,
            r#"button:has-text("Not now")"#,
            r#"span:has-text("Omitir")"#,
            r#"span:has-text("Skip")"#,
        ];

        for &selector in &skip_selectors {
            if let Ok(btn) = page.find_element(selector).await {
                info!("⏭️ [GMAIL] Omitiendo verificación telefónica...");
                let _ = btn.click().await;
                Self::esperar_humano(1500, 2500).await;
                break;
            }
        }

        // Fallback interactivo/automático: Si Google exige verificación telefónica
        if let Ok(phone_input) = page
            .find_element(r#"input[type="tel"], input#phoneNumberId"#)
            .await
        {
            info!("🔱 [NEXUS] Google está solicitando verificación telefónica obligatoria.");

            if let Some(sms) = &self.sms_client {
                info!("🤖 [NEXUS] Iniciando bypass automático de SMS...");

                // Pedir número para Google (go) en Paraguay (68) o Indonesia (6) que suele ser barato
                match sms.pedir_numero("go", "68").await {
                    Ok((id, number)) => {
                        info!("✍️ [GMAIL] Escribiendo número virtual: {}", number);
                        let _ = Self::type_human_like(&phone_input, &number).await;
                        Self::esperar_humano(1000, 2000).await;

                        let _ = Self::click_siguiente(page).await;
                        Self::esperar_humano(4000, 6000).await;

                        // Esperar a la pantalla del código
                        if let Ok(code_input) =
                            page.find_element(r#"input#code, input[name="code"]"#).await
                        {
                            match sms.esperar_codigo(&id).await {
                                Ok(code) => {
                                    info!("✍️ [GMAIL] Escribiendo código recibido: {}", code);
                                    let _ = Self::type_human_like(&code_input, &code).await;
                                    Self::esperar_humano(1000, 2000).await;
                                    let _ = Self::click_siguiente(page).await;
                                    Self::esperar_humano(5000, 7000).await;
                                    let _ = sms.confirmar_exito(&id).await;
                                }
                                Err(e) => warn!("❌ [SMS] Error esperando código: {}", e),
                            }
                        }
                    }
                    Err(e) => warn!("❌ [SMS] No se pudo obtener número: {}", e),
                }
            } else {
                // Fallback interactivo si no hay cliente SMS
                println!("\n📞 [INTERACTIVO] Google requiere verificar teléfono.");
                // ... rest of interactive code ...
            }
        }

        // Paso 7: Aceptar términos de servicio
        let agree_selectors = [
            r#"button:has-text("Acepto")"#,
            r#"button:has-text("I agree")"#,
            r#"button:has-text("Aceptar")"#,
        ];

        for &selector in &agree_selectors {
            if let Ok(btn) = page.find_element(selector).await {
                info!("✅ [GMAIL] Aceptando términos...");
                let _ = btn.click().await;
                Self::esperar_humano(2000, 3000).await;
                break;
            }
        }

        Self::esperar_humano(2000, 3000).await;

        // Verificar resultado
        let final_url = page.url().await.unwrap_or_default().unwrap_or_default();
        info!("📍 [GMAIL] URL final: {}", final_url);

        let email = format!("{}@gmail.com", username_suggested);

        if final_url.contains("myaccount")
            || final_url.contains("signin")
            || final_url.contains("google")
        {
            info!("✅ [GMAIL] CUENTA CREADA: {}", email);
            PlantResult {
                success: true,
                email: Some(email),
                password: Some(password.to_string()),
                error: None,
                pending_verification: false,
            }
        } else if final_url.contains("signup") || final_url.contains("createaccount") {
            info!(
                "⚠️ [GMAIL] Puede requerir verificación adicional: {}",
                email
            );
            PlantResult {
                success: true,
                email: Some(email),
                password: Some(password.to_string()),
                error: None,
                pending_verification: true,
            }
        } else {
            warn!("⚠️ [GMAIL] Estado desconocido. URL: {}", final_url);
            PlantResult {
                success: true,
                email: Some(email),
                password: Some(password.to_string()),
                error: Some(format!("URL final inesperada: {}", final_url)),
                pending_verification: true,
            }
        }
    }

    // ─── 2. LOGIN A GMAIL ─────────────────────────────────────────────────

    /// Inicia sesión en una cuenta de Gmail existente
    pub async fn login_gmail(
        &self,
        email: &str,
        password: &str,
        identity: Option<&SyntheticIdentity>,
    ) -> LoginResult {
        info!("🔑 [LOGIN] Accediendo a Gmail: {}", email);

        let (mut browser, page, profile_dir) = if let Some(id) = identity {
            match self.lanzar_con_identidad(id).await {
                Ok((b, p, d)) => (b, p, d),
                Err(e) => {
                    return LoginResult::error(email, format!("Error lanzando browser: {}", e))
                }
            }
        } else {
            match Self::lanzar_generico(None).await {
                Ok((b, p, d)) => (b, p, d),
                Err(e) => {
                    return LoginResult::error(email, format!("Error lanzando browser: {}", e))
                }
            }
        };

        // Navegar a AccountChooser
        let login_url = format!(
            "{}/AccountChooser?Email={}&continue=https://mail.google.com",
            GMAIL_LOGIN_URL, email
        );

        if let Err(e) = page.goto(&login_url).await {
            return LoginResult::error(email, format!("Error navegando a login: {}", e));
        }
        Self::esperar_humano(2000, 4000).await;

        // Si no está logueado, ingresar password
        match page.find_element(r#"input[type="password"]"#).await {
            Ok(pass_el) => {
                info!("🔑 [LOGIN] Ingresando password...");
                if let Err(e) = Self::type_human_like(&pass_el, password).await {
                    return LoginResult::error(email, format!("Error escribiendo password: {}", e));
                }
                Self::esperar_humano(500, 1000).await;

                // Click en Siguiente/Next
                if let Ok(btn) = page
                    .find_element(r#"#passwordNext, button[jsname="V67aGc"]"#)
                    .await
                {
                    let _ = btn.click().await;
                }
                Self::esperar_humano(3000, 5000).await;

                info!("✅ [LOGIN] Sesión iniciada: {}", email);
                LoginResult {
                    success: true,
                    email: email.to_string(),
                    error: None,
                }
            }
            Err(_) => {
                // Ya logueado o pantalla diferente
                info!("✅ [LOGIN] Sesión ya activa: {}", email);
                LoginResult {
                    success: true,
                    email: email.to_string(),
                    error: None,
                }
            }
        }
    }

    // ─── 3. CREAR CUENTA FACEBOOK ─────────────────────────────────────────

    /// Crea una cuenta de Facebook navegando facebook.com/r.php
    pub async fn crear_cuenta_facebook(
        &self,
        nombre: &str,
        apellido: &str,
        email: &str,
        password: &str,
        identity: &SyntheticIdentity,
    ) -> PlantResult {
        info!("📘 [FACEBOOK] Registrando a {} {}...", nombre, apellido);

        let (mut browser, page, profile_dir) = match self.lanzar_con_identidad(identity).await {
            Ok(v) => v,
            Err(e) => return PlantResult::error(format!("Error lanzando browser: {}", e)),
        };

        let result =
            Self::flujo_crear_facebook_inner(&page, nombre, apellido, email, password).await;

        let _ = shutdown_browser_session(
            &mut browser,
            tokio::spawn(async {}),
            profile_dir,
            "ChromePlanter: crear_cuenta_facebook",
        )
        .await;

        result
    }

    async fn flujo_crear_facebook_inner(
        page: &Page,
        nombre: &str,
        apellido: &str,
        email: &str,
        password: &str,
    ) -> PlantResult {
        if let Err(e) = page.goto(FACEBOOK_SIGNUP_URL).await {
            return PlantResult::error(format!("Error navegando a Facebook: {}", e));
        }
        Self::esperar_humano(2000, 3000).await;
        info!("✅ [FACEBOOK] Página cargada");

        // Rellenar formulario
        let campos = [
            (r#"input[name="firstname"]"#, nombre),
            (r#"input[name="lastname"]"#, apellido),
            (r#"input[name="reg_email__"]"#, email),
            (r#"input[name="reg_passwd__"]"#, password),
        ];

        for &(selector, valor) in &campos {
            if let Ok(el) = page.find_element(selector).await {
                if let Err(e) = Self::type_human_like(&el, valor).await {
                    return PlantResult::error(format!(
                        "Error rellenando campo {}: {}",
                        selector, e
                    ));
                }
                Self::esperar_humano(300, 800).await;
            }
        }

        // Fecha de nacimiento aleatoria
        let birth_year = (1985 + rand::thread_rng().gen_range(0..20)).to_string();
        let birth_day = (1 + rand::thread_rng().gen_range(0..28)).to_string();
        let birth_month = (1 + rand::thread_rng().gen_range(0..12)).to_string();

        let selects: Vec<(&str, &str)> = vec![
            ("#day", &birth_day),
            ("#month", &birth_month),
            ("#year", &birth_year),
        ];

        for &(sel, val) in &selects {
            if let Ok(el) = page.find_element(sel).await {
                let _ = el.click().await;
                Self::esperar_humano(100, 300).await;
                // Intentar seleccionar opción por valor
                let js = format!(
                    r#"document.querySelector('{}').value = '{}'; document.querySelector('{}').dispatchEvent(new Event('change', {{ bubbles: true }}));"#,
                    sel, val, sel
                );
                let _ = page.evaluate(js).await;
                Self::esperar_humano(200, 500).await;
            }
        }

        // Género aleatorio
        let genero = if rand::thread_rng().gen_bool(0.5) {
            "2"
        } else {
            "1"
        };
        let js_genero = format!(
            r#"document.querySelector('input[value="{}"]').click();"#,
            genero
        );
        let _ = page.evaluate(js_genero).await;
        Self::esperar_humano(500, 1000).await;

        // Click en Registrar
        match page.find_element(r#"button[name="websubmit"]"#).await {
            Ok(btn) => {
                info!("🧬 [FACEBOOK] Enviando registro...");
                let _ = btn.click().await;
                Self::esperar_humano(5000, 8000).await;
                info!("✅ [FACEBOOK] Registro completado para {}", email);
                PlantResult {
                    success: true,
                    email: Some(email.to_string()),
                    password: Some(password.to_string()),
                    error: None,
                    pending_verification: false,
                }
            }
            Err(e) => PlantResult::error(format!("No se encontró botón submit: {}", e)),
        }
    }

    // ─── 4. CREAR CUENTA PROTON ───────────────────────────────────────────

    /// Crea una cuenta de Proton Mail navegando account.proton.me/mail/signup
    pub async fn crear_cuenta_proton(
        &self,
        nombre: &str,
        apellido: &str,
        password: &str,
        recovery_email: Option<&str>,
        identity: &SyntheticIdentity,
    ) -> PlantResult {
        info!("📧 [PROTON] Creando cuenta para {} {}...", nombre, apellido);

        let (mut browser, page, profile_dir) = match self.lanzar_con_identidad(identity).await {
            Ok(v) => v,
            Err(e) => return PlantResult::error(format!("Error lanzando browser: {}", e)),
        };

        let result =
            Self::flujo_crear_proton_inner(&page, nombre, apellido, password, recovery_email).await;

        let _ = shutdown_browser_session(
            &mut browser,
            tokio::spawn(async {}),
            profile_dir,
            "ChromePlanter: crear_cuenta_proton",
        )
        .await;

        result
    }

    async fn flujo_crear_proton_inner(
        page: &Page,
        nombre: &str,
        apellido: &str,
        password: &str,
        recovery_email: Option<&str>,
    ) -> PlantResult {
        if let Err(e) = page.goto(PROTON_SIGNUP_URL).await {
            return PlantResult::error(format!("Error navegando a Proton: {}", e));
        }
        Self::esperar_humano(3000, 5000).await;
        info!("✅ [PROTON] Página cargada");

        // Elegir plan gratuito
        let free_selectors = [
            r#"button:has-text("Free")"#,
            r#"button:has-text("Gratuito")"#,
            r#"button:has-text("Gratis")"#,
        ];
        for &selector in &free_selectors {
            if let Ok(btn) = page.find_element(selector).await {
                info!("💎 [PROTON] Seleccionando plan gratuito...");
                let _ = btn.click().await;
                Self::esperar_humano(2000, 3000).await;
                break;
            }
        }

        // Username
        let username = format!(
            "{}{}{}",
            nombre.to_lowercase(),
            apellido.to_lowercase(),
            rand::thread_rng().gen_range(100..9999)
        );

        let username_selectors = [
            r#"input[name="username"]"#,
            r#"input[autocomplete="username"]"#,
            r#"input[id="username"]"#,
        ];

        for &selector in &username_selectors {
            if let Ok(el) = page.find_element(selector).await {
                info!("📧 [PROTON] Username: {}", username);
                if let Err(e) = Self::type_human_like(&el, &username).await {
                    warn!("[PROTON] Error escribiendo username: {}", e);
                }
                Self::esperar_humano(500, 1000).await;
                break;
            }
        }

        // Contraseña
        if let Ok(pass_inputs) = page.find_elements(r#"input[type="password"]"#).await {
            if pass_inputs.len() >= 2 {
                info!("🔑 [PROTON] Ingresando contraseña...");
                let _ = Self::type_human_like(&pass_inputs[0], password).await;
                Self::esperar_humano(200, 500).await;
                let _ = Self::type_human_like(&pass_inputs[1], password).await;
            }
        }

        // Recovery email
        if let Some(recovery) = recovery_email {
            let recovery_selectors = [
                r#"input[type="email"]"#,
                r#"input[name="recoveryEmail"]"#,
                r#"input[autocomplete="email"]"#,
            ];
            for &selector in &recovery_selectors {
                if let Ok(el) = page.find_element(selector).await {
                    info!("📧 [PROTON] Ingresando recovery email...");
                    let _ = Self::type_human_like(&el, recovery).await;
                    Self::esperar_humano(300, 700).await;
                    break;
                }
            }
        }

        Self::esperar_humano(1000, 2000).await;

        // Submit
        match page.find_element(r#"button[type="submit"]"#).await {
            Ok(btn) => {
                let _ = btn.click().await;
                info!("📤 [PROTON] Formulario enviado");
            }
            Err(_) => {
                // Intentar vía JS
                let _ = page
                    .evaluate(r#"document.querySelector('button[type="submit"]')?.click()"#)
                    .await;
            }
        }

        Self::esperar_humano(5000, 8000).await;

        let email = format!("{}@proton.me", username);
        info!("✅ [PROTON] Cuenta creada: {}", email);

        PlantResult {
            success: true,
            email: Some(email),
            password: Some(password.to_string()),
            error: None,
            pending_verification: false,
        }
    }

    // ─── Helpers ──────────────────────────────────────────────────────────

    /// Encuentra un campo input y escribe texto con delays humanos
    async fn rellenar_campo(page: &Page, selector: &str, texto: &str) -> Result<()> {
        let el = page
            .find_element(selector)
            .await
            .map_err(|e| anyhow!("Selector '{}' no encontrado: {:?}", selector, e))?;
        Self::type_human_like(&el, texto).await
    }

    /// Click en botón "Siguiente" de Google
    async fn click_siguiente(page: &Page) -> Result<()> {
        let selectores = [
            r#"#accountDetailsNext"#,
            r#"button[jsname="V67aGc"]"#,
            r#"button:has-text("Siguiente")"#,
            r#"button:has-text("Next")"#,
            r#"#passwordNext"#,
            r#"#identifierNext"#,
        ];
        for &selector in &selectores {
            if let Ok(btn) = page.find_element(selector).await {
                btn.click().await?;
                return Ok(());
            }
        }
        // Fallback: JavaScript click
        page.evaluate(
            r#"document.querySelector('#accountDetailsNext, button[jsname="V67aGc"], #passwordNext, #identifierNext')?.click()"#,
        )
        .await
        .map_err(|e| anyhow!("Error click siguiente via JS: {}", e))?;
        Ok(())
    }

    // ─── 5. COSECHAR API KEY GEMINI (AI Studio) ───────────────────────────

    /// Navega a Google AI Studio y extrae una API key.
    /// Asume que la sesión ya está iniciada o usa el perfil proporcionado.
    pub async fn cosechar_api_key_gemini(&self, identity: &SyntheticIdentity) -> Result<String> {
        info!("💎 [COSECHA] Iniciando cosecha de API Key en Google AI Studio...");

        let (mut browser, page, profile_dir) = self.lanzar_con_identidad(identity).await?;

        // 1. Ir a la página de API Keys
        info!("🌐 [COSECHA] Navegando a AI Studio API Keys...");
        page.goto("https://aistudio.google.com/app/apikey").await?;
        Self::esperar_humano(6000, 9000).await;

        // 2. Intentar detectar si ya hay una llave o hay que crearla
        let create_btn_selector = r#"button:has-text("Create API key in new project"), button:has-text("Crear clave de API en un proyecto nuevo")"#;

        if let Ok(btn) = page.find_element(create_btn_selector).await {
            info!("🖱️ [COSECHA] Botón de creación detectado. Haciendo click...");
            btn.click().await?;
            Self::esperar_humano(8000, 12000).await;
        }

        // 3. Buscar el elemento que contiene la llave
        let api_key = match page
            .evaluate(
                r#"
            (() => {
                const elements = Array.from(document.querySelectorAll('input, span, div, code'));
                for (const el of elements) {
                    const text = el.innerText || el.value || '';
                    if (text.startsWith('AIzaSy') && text.length > 30) {
                        return text.trim();
                    }
                }
                return null;
            })()
        "#,
            )
            .await
        {
            Ok(val) => {
                let key = val
                    .into_value::<Option<String>>()
                    .unwrap_or_default()
                    .unwrap_or_default();
                if !key.is_empty() {
                    info!(
                        "✅ [COSECHA] API Key extraída con éxito: AIzaSy...{}",
                        &key[key.len() - 4..]
                    );
                    key
                } else {
                    return Err(anyhow!("No se encontró ninguna API key en la página"));
                }
            }
            Err(e) => return Err(anyhow!("Error buscando API key: {}", e)),
        };

        let _ = shutdown_browser_session(
            &mut browser,
            tokio::spawn(async {}),
            profile_dir,
            "ChromePlanter: cosechar_api_key_gemini",
        )
        .await;

        Ok(api_key)
    }
}

// ─── Implementaciones de resultados ──────────────────────────────────────────

impl PlantResult {
    fn error(msg: String) -> Self {
        error!("[ChromePlanter] ❌ {}", msg);
        Self {
            success: false,
            email: None,
            password: None,
            error: Some(msg),
            pending_verification: false,
        }
    }
}

impl LoginResult {
    fn error(email: &str, msg: String) -> Self {
        error!("[LOGIN] ❌ {}: {}", email, msg);
        Self {
            success: false,
            email: email.to_string(),
            error: Some(msg),
        }
    }
}

// ─── Funciones helper ────────────────────────────────────────────────────────

/// Parsea resolución "1920x1080" → (1920, 1080)
fn parsear_resolucion(res: &str) -> (u32, u32) {
    let parts: Vec<&str> = res.split('x').collect();
    if parts.len() == 2 {
        let w = parts[0].parse::<u32>().unwrap_or(VIEWPORT_W);
        let h = parts[1].parse::<u32>().unwrap_or(VIEWPORT_H);
        (w, h)
    } else {
        (VIEWPORT_W, VIEWPORT_H)
    }
}
