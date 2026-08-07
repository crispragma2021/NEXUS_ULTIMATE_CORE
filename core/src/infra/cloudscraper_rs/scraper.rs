// 🔱 cloudscraper_rs — Transmutación Rust Pura de cloudscraper (Cloudflare bypass)
// Estrategia principal: bypass de Cloudflare vía headers realistas, User-Agent rotatorio,
// detección de challenges y cookies. Para JS rendering, usa browser_native del mismo proyecto.
//
// Fases:
// 1. Intento directo con reqwest + headers de Chrome real + UA rotatorio
// 2. Detección de desafíos CF (cf-browser-verification, challenge, etc)
// 3. Si CF detectado, sugiere usar browser_native::BrowserPool para JS rendering
//
// Cero dependencias externas nuevas. Solo reqwest + regex del arsenal existente.

use anyhow::{anyhow, Result};
use rand::seq::SliceRandom;
use std::time::Duration;
use tracing::{debug, info, warn};

// ── User-Agent pool realista ────────────────────────────────────────

const DESKTOP_AGENTS: &[&str] = &[
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/132.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/132.0.0.0 Safari/537.36",
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:133.0) Gecko/20100101 Firefox/133.0",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 14.7; rv:133.0) Gecko/20100101 Firefox/133.0",
    "Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:133.0) Gecko/20100101 Firefox/133.0",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/132.0.0.0 Safari/537.36 Edg/132.0.0.0",
];

fn random_ua() -> &'static str {
    DESKTOP_AGENTS
        .choose(&mut rand::thread_rng())
        .copied()
        .unwrap_or(DESKTOP_AGENTS[0])
}

/// Headers HTTP de Chrome 132 real (Linux)
fn browser_headers() -> Vec<(&'static str, &'static str)> {
    vec![
        ("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8"),
        ("Accept-Language", "es-ES,es;q=0.9,en;q=0.8"),
        ("Accept-Encoding", "gzip, deflate, br"),
        ("Sec-Ch-Ua", "\"Chromium\";v=\"132\", \"Google Chrome\";v=\"132\", \"Not?A_Brand\";v=\"99\""),
        ("Sec-Ch-Ua-Mobile", "?0"),
        ("Sec-Ch-Ua-Platform", "\"Linux\""),
        ("Sec-Fetch-Dest", "document"),
        ("Sec-Fetch-Mode", "navigate"),
        ("Sec-Fetch-Site", "none"),
        ("Sec-Fetch-User", "?1"),
        ("Upgrade-Insecure-Requests", "1"),
        ("Dnt", "1"),
        ("Connection", "keep-alive"),
    ]
}

/// Detecta si un body HTML contiene un desafío de Cloudflare
fn is_cloudflare_challenge(body: &str) -> bool {
    let lower = body.to_lowercase();
    lower.contains("cf-browser-verification")
        || lower.contains("__cf_challenge")
        || lower.contains("cf_chl_opt")
        || lower.contains("jschl-vc")
        || lower.contains("challenge-platform")
        || lower.contains("cdn-cgi/challenge-platform")
        || lower.contains("just a moment...")
        || lower.contains("checking your browser")
        || lower.contains("attention required") && lower.contains("cloudflare")
}

/// Detecta bloqueo CF por status code + body
fn is_cloudflare_block(body: &str, status: u16) -> bool {
    (status == 403 || status == 503)
        && (body.to_lowercase().contains("cloudflare") || body.contains("cf-"))
}

/// Resultado de extracción
#[derive(Debug, Clone)]
pub struct ScrapeResult {
    pub html: String,
    pub url: String,
    pub status: u16,
    pub cf_detected: bool,
    pub headers: Vec<(String, String)>,
}

/// Método usado
#[derive(Debug, Clone, PartialEq)]
pub enum ScrapeMethod {
    Direct,
    Browser,
}

/// Configuración
#[derive(Debug, Clone)]
pub struct CloudScraperConfig {
    pub timeout_secs: u64,
    pub max_redirects: u32,
    pub follow_redirects: bool,
    pub extra_headers: Vec<(String, String)>,
}

impl Default for CloudScraperConfig {
    fn default() -> Self {
        Self {
            timeout_secs: 30,
            max_redirects: 5,
            follow_redirects: true,
            extra_headers: Vec::new(),
        }
    }
}

/// Intento directo con reqwest
async fn try_direct(url: &str, config: &CloudScraperConfig) -> Result<ScrapeResult> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(config.timeout_secs))
        .danger_accept_invalid_certs(false)
        .redirect(if config.follow_redirects {
            reqwest::redirect::Policy::limited(config.max_redirects as usize)
        } else {
            reqwest::redirect::Policy::none()
        })
        .build()
        .map_err(|e| anyhow!("reqwest client: {}", e))?;

    let ua = random_ua();
    let mut req = client.get(url).header("User-Agent", ua);
    for (k, v) in browser_headers() {
        if k != "Accept-Encoding" {
            req = req.header(k, v);
        }
    }
    for (k, v) in &config.extra_headers {
        req = req.header(k.as_str(), v.as_str());
    }

    // Recorte seguro: límite en el byte del 40º carácter sin partir chars multibyte
    let ua_preview = {
        let boundary = ua.char_indices().take(40).fold(0, |acc, (i, _)| i);
        &ua[..boundary.max(1).min(ua.len())]
    };
    debug!("[cloudscraper] GET {} (UA: {})", url, ua_preview);
    let resp = req
        .send()
        .await
        .map_err(|e| anyhow!("Request failed: {}", e))?;
    let status = resp.status().as_u16();
    let final_url = resp.url().to_string();

    // Capturar headers de respuesta (útiles para debugging)
    let resp_headers: Vec<(String, String)> = resp
        .headers()
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
        .collect();

    let text = resp.text().await.map_err(|e| anyhow!("Body read: {}", e))?;

    if is_cloudflare_challenge(&text) {
        debug!(
            "[cloudscraper] CF challenge detected: {} (HTTP {})",
            url, status
        );
        return Err(anyhow!("Cloudflare challenge (HTTP {})", status));
    }

    if is_cloudflare_block(&text, status) {
        debug!(
            "[cloudscraper] CF block detected: {} (HTTP {})",
            url, status
        );
        return Err(anyhow!("Cloudflare block (HTTP {})", status));
    }

    if status >= 400 && status != 404 {
        // 404 puede ser válido, otros errores no
        return Err(anyhow!("HTTP {} for {}", status, url));
    }

    Ok(ScrapeResult {
        html: text,
        url: final_url,
        status,
        cf_detected: false,
        headers: resp_headers,
    })
}

/// Punto de entrada principal
pub async fn scrape(url: &str) -> Result<ScrapeResult> {
    scrape_with_config(url, &CloudScraperConfig::default()).await
}

/// scrape con configuración personalizada
pub async fn scrape_with_config(url: &str, config: &CloudScraperConfig) -> Result<ScrapeResult> {
    match try_direct(url, config).await {
        Ok(result) => {
            info!(
                "[cloudscraper] ✓ {} ({} bytes, HTTP {})",
                url,
                result.html.len(),
                result.status
            );
            Ok(result)
        }
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("Cloudflare") {
                warn!("[cloudscraper] ✗ CF detected: {}. Use browser_native::BrowserPool for JS rendering.", url);
                Err(anyhow!("Cloudflare challenge: {}. Try browser_native with chromiumoxide for JS rendering.", url))
            } else {
                warn!("[cloudscraper] ✗ Failed: {}: {}", url, msg);
                Err(anyhow!("{}: {}", url, msg))
            }
        }
    }
}

/// Extrae texto plano del HTML
pub fn extract_text(html: &str) -> String {
    let re = regex::Regex::new(r"<[^>]*>").unwrap();
    let without_tags = re.replace_all(html, " ");
    let re_space = regex::Regex::new(r"\s+").unwrap();
    let collapsed = re_space.replace_all(&without_tags, " ");
    collapsed.trim().to_string()
}

/// Extrae enlaces absolutos del HTML
pub fn extract_urls(base_url: &str, html: &str) -> Vec<String> {
    let re = regex::Regex::new(r#"href=["']([^"']+)["']"#).unwrap();
    let base_origin = base_url
        .split('/')
        .take(3) // https://example.com
        .collect::<Vec<_>>()
        .join("/");

    let mut urls = Vec::new();
    for cap in re.captures_iter(html) {
        if let Some(href) = cap.get(1) {
            let href_str = href.as_str();
            if href_str.starts_with("http") {
                urls.push(href_str.to_string());
            } else if href_str.starts_with('/') {
                urls.push(format!("{}{}", base_origin, href_str));
            }
            // ignore protocol-relative, mailto, etc
        }
    }
    urls
}

// ══════════════════════════════════════════════════════════════════════
// Tests
// ══════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_cf_challenge_verification() {
        let body = r#"<html><body><div id="cf-browser-verification"></div></body></html>"#;
        assert!(is_cloudflare_challenge(body));
    }

    #[test]
    fn test_detect_cf_challenge_just_moment() {
        let body = "Just a moment... Checking your browser";
        assert!(is_cloudflare_challenge(body));
    }

    #[test]
    fn test_detect_cf_challenge_attention() {
        let body = "Attention Required! | Cloudflare";
        assert!(is_cloudflare_challenge(body));
    }

    #[test]
    fn test_no_false_positive_on_normal() {
        let body = "<html><head><title>Normal page</title></head><body>Hello</body></html>";
        assert!(!is_cloudflare_challenge(body));
    }

    #[test]
    fn test_is_cloudflare_block_403() {
        assert!(is_cloudflare_block("cloudflare error", 403));
    }

    #[test]
    fn test_is_cloudflare_block_503() {
        assert!(is_cloudflare_block("Cloudflare Ray ID:", 503));
    }

    #[test]
    fn test_not_block_on_404() {
        assert!(!is_cloudflare_block("not found", 404));
    }

    #[test]
    fn test_extract_text() {
        let html = "<html><body><h1>Hello</h1><p>World</p></body></html>";
        assert_eq!(extract_text(html), "Hello World");
    }

    #[test]
    fn test_extract_text_collapses() {
        let html = "<div>  Hola    Mundo  </div>";
        assert_eq!(extract_text(html), "Hola Mundo");
    }

    #[test]
    fn test_extract_text_empty() {
        assert_eq!(extract_text(""), "");
    }

    #[test]
    fn test_extract_urls_absolute() {
        let html = r#"<a href="https://example.com/page">link</a>"#;
        let urls = extract_urls("https://example.com", html);
        assert!(urls.contains(&"https://example.com/page".to_string()));
    }

    #[test]
    fn test_extract_urls_relative() {
        let html = r#"<a href="/about">link</a>"#;
        let urls = extract_urls("https://example.com", html);
        assert!(urls.contains(&"https://example.com/about".to_string()));
    }

    #[test]
    fn test_extract_urls_skip_mailto() {
        let html = r#"<a href="mailto:test@test.com">email</a>"#;
        let urls = extract_urls("https://example.com", html);
        assert!(urls.is_empty());
    }

    #[test]
    fn test_config_default() {
        let cfg = CloudScraperConfig::default();
        assert_eq!(cfg.timeout_secs, 30);
        assert_eq!(cfg.max_redirects, 5);
    }

    #[test]
    fn test_scrape_method_display() {
        assert_eq!(format!("{:?}", ScrapeMethod::Direct), "Direct");
        assert_eq!(format!("{:?}", ScrapeMethod::Browser), "Browser");
    }

    #[test]
    fn test_is_cloudflare_challenge_case_insensitive() {
        let body = "CHECKING YOUR BROWSER BEFORE ACCESSING";
        assert!(is_cloudflare_challenge(body));
    }
}
