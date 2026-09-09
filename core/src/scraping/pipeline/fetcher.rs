//! Captura determinista de HTML (F1.1).
//!
//! - `Fetcher` usa `reqwest` (HTTP estático) con timeout, redirects limitados
//!   y User-Agent configurable / rotación.
//! - Respeto a `robots.txt` (spec §6.1) con cache en SQLite (TTL 24h) y
//!   `Crawl-delay`.
//! - Rate limiting por dominio (spec §6.2) usando `rate_limit_state`.
//!
//! La estrategia `headless` (chromiumoxide) se integra en F4; esta fase
//! implementa `http` con el hook `headless_fetch` para el fallback.

use crate::scraping::pipeline::db::PipelineDb;
use crate::scraping::pipeline::schemas::{Strategy, TaskSchema};
use anyhow::{anyhow, Result};
use std::time::Duration;
use url::Url;

/// Lista de User-Agents de navegadores reales para rotación.
const USER_AGENTS: [&str; 5] = [
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/125.0.0.0 Safari/537.36",
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/125.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.4 Safari/605.1.15",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:126.0) Gecko/20100101 Firefox/126.0",
    "Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:126.0) Gecko/20100101 Firefox/126.0",
];

/// Resultado de un fetch.
pub struct FetchOutput {
    pub html: String,
    pub final_url: String,
    pub elapsed_ms: u64,
}

/// Captura HTTP de una página.
#[derive(Clone)]
pub struct Fetcher {
    client: reqwest::Client,
    db: Option<std::sync::Arc<PipelineDb>>,
    /// Permite inyectar un capturador headless (chromiumoxide) en F4.
    pub headless_fetch: Option<std::sync::Arc<dyn HeadlessFetch + Send + Sync>>,
}

/// Trait para captura headless (SPA). Implementado en F4 con chromiumoxide.
pub trait HeadlessFetch: Send + Sync {
    fn fetch_rendered(&self, url: &str) -> Result<String>;
}

impl Fetcher {
    /// Construye un capturador HTTP.
    pub fn new(db: Option<std::sync::Arc<PipelineDb>>) -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .redirect(reqwest::redirect::Policy::limited(5))
            .build()?;
        Ok(Self {
            client,
            db,
            headless_fetch: None,
        })
    }

    /// Devuelve un User-Agent rotativo según la tarea.
    fn resolve_ua(&self, task: &TaskSchema) -> String {
        // Si el task define un UA, respetarlo; si no, rotar de la lista base
        // usando el task_id como semilla determinista.
        if task.user_agent.contains("NexusScraper") || task.user_agent.is_empty() {
            let idx = (task.task_id.len() + task.url.len()) % USER_AGENTS.len();
            USER_AGENTS[idx].to_string()
        } else {
            task.user_agent.clone()
        }
    }

    /// Ejecuta la captura según la estrategia de la tarea.
    pub async fn fetch(&self, task: &TaskSchema) -> Result<FetchOutput> {
        task.validate().map_err(|e| anyhow!(e))?;

        // 1. Respeto a robots.txt (si está habilitado).
        if task.respect_robots_txt {
            if let Err(e) = self.check_robots(task).await {
                // Marcar rate limit fallo y propagar como error de robots.
                if let Some(db) = &self.db {
                    if let Ok(domain) = domain_of(&task.url) {
                        let _ = db.record_request(&domain, false);
                    }
                }
                return Err(e);
            }
        }

        // 2. Captura según estrategia.
        let start = std::time::Instant::now();
        let (html, final_url) = match task.strategy {
            Strategy::Http => self.fetch_http(task).await?,
            Strategy::Headless => {
                let fetcher = self
                    .headless_fetch
                    .as_ref()
                    .ok_or_else(|| anyhow!("estrategia headless sin implementar (F4)"))?;
                let url = task.url.clone();
                (fetcher.fetch_rendered(&url)?, url)
            }
        };

        // 3. Registrar éxito en rate limit state.
        if let Some(db) = &self.db {
            if let Ok(domain) = domain_of(&task.url) {
                let _ = db.record_request(&domain, true);
            }
        }

        Ok(FetchOutput {
            html,
            final_url,
            elapsed_ms: start.elapsed().as_millis() as u64,
        })
    }

    /// Descarga HTML con `reqwest` y UA rotativo.
    async fn fetch_http(&self, task: &TaskSchema) -> Result<(String, String)> {
        let ua = self.resolve_ua(task);
        let resp = self
            .client
            .get(&task.url)
            .header("User-Agent", ua)
            .header("Accept-Language", "es-ES,es;q=0.9,en;q=0.8")
            .send()
            .await?;

        if resp.status().is_success() {
            let final_url = resp.url().to_string();
            let html = resp.text().await?;
            Ok((html, final_url))
        } else {
            Err(anyhow!(
                "HTTP {} al fetchear {}",
                resp.status().as_u16(),
                task.url
            ))
        }
    }

    /// Verifica robots.txt: descarga/parsea y decide si la URL está permitida.
    async fn check_robots(&self, task: &TaskSchema) -> Result<()> {
        let url = Url::parse(&task.url)?;
        let domain = url
            .host_str()
            .ok_or_else(|| anyhow!("sin host: {}", task.url))?;
        let scheme = url.scheme();
        let base = format!("{scheme}://{domain}");

        // 1. Cache en DB.
        if let Some(db) = &self.db {
            if let Some(rules) = db.get_robots_cache(domain)? {
                if robots_allows(&rules, &task.url) {
                    return Ok(());
                }
                return Err(anyhow!("blocked_by_robots: {}", task.url));
            }
        }

        // 2. Descargar robots.txt (timeout corto; si falla, permitir por defecto).
        let robots_url = format!("{base}/robots.txt");
        let ua = self.resolve_ua(task);
        let resp = match self
            .client
            .get(&robots_url)
            .header("User-Agent", ua)
            .send()
            .await
        {
            Ok(r) => r,
            Err(_) => return Ok(()), // sin robots.txt → permitido
        };

        let rules = if resp.status().is_success() {
            resp.text().await.unwrap_or_default()
        } else {
            String::new()
        };

        // 3. Cachear + evaluar.
        if let Some(db) = &self.db {
            let _ = db.set_robots_cache(domain, &rules);
        }
        if robots_allows(&rules, &task.url) {
            Ok(())
        } else {
            Err(anyhow!("blocked_by_robots: {}", task.url))
        }
    }
}

/// Devuelve el dominio de una URL.
pub fn domain_of(url: &str) -> Result<String> {
    let url = Url::parse(url)?;
    Ok(url.host_str().unwrap_or("").to_string())
}

/// Parser mínimo de robots.txt (spec §6.1): User-agent, Allow, Disallow,
/// Crawl-delay. Se aplican reglas de `*` y de `NexusScraper`.
fn robots_allows(rules: &str, url: &str) -> bool {
    let mut disallow: Vec<&str> = Vec::new();
    let mut allow: Vec<&str> = Vec::new();
    let mut current_agent = String::new();
    let mut crawl_delay: Option<u64> = None;

    for raw_line in rules.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        // Dividir en clave/valor.
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        let key = key.trim().to_ascii_lowercase();
        let value = value.trim();

        match key.as_str() {
            "user-agent" => current_agent = value.to_ascii_lowercase(),
            "allow" => {
                if current_agent == "*" || current_agent.contains("nexus") {
                    allow.push(value);
                }
            }
            "disallow" => {
                if current_agent == "*" || current_agent.contains("nexus") {
                    disallow.push(value);
                }
            }
            "crawl-delay" => {
                if current_agent == "*" || current_agent.contains("nexus") {
                    crawl_delay = value.parse().ok();
                }
            }
            _ => {}
        }
    }

    // (Crawl-delay se aplica en el rate limiter; aquí solo se lee.)
    let _ = crawl_delay;

    // Regla: Allow gana sobre Disallow para el mismo path.
    let path = match Url::parse(url) {
        Ok(u) => u.path().to_string(),
        Err(_) => return true,
    };

    for a in allow {
        if path.starts_with(a) {
            return true;
        }
    }
    for d in disallow {
        if d.is_empty() {
            continue; // "Disallow:" vacío → permite todo.
        }
        if path.starts_with(d) {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn robots_bloquea_ruta_disallow() {
        let rules = "User-agent: *\nDisallow: /admin/\nDisallow: /private";
        assert!(robots_allows(rules, "https://example.com/public"));
        assert!(!robots_allows(rules, "https://example.com/admin/x"));
        assert!(!robots_allows(rules, "https://example.com/private/data"));
    }

    #[test]
    fn robots_allow_gana_a_disallow() {
        let rules = "User-agent: *\nDisallow: /\nAllow: /public/";
        assert!(robots_allows(rules, "https://example.com/public/a.html"));
        assert!(!robots_allows(rules, "https://example.com/private/b.html"));
    }

    #[test]
    fn robots_vacio_permite_todo() {
        assert!(robots_allows("", "https://example.com/"));
        assert!(robots_allows(
            "User-agent: *\nDisallow:\n",
            "https://example.com/"
        ));
    }

    #[test]
    fn domain_extrae_host() {
        assert_eq!(
            domain_of("https://example.com/path?x=1").unwrap(),
            "example.com"
        );
    }
}
