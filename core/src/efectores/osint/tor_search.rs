// ──────────────────────────────────────────────
// 🌑 TOR SEARCH — Búsqueda en Tor/Ahmia
// Usa Tor SOCKS5 proxy para consultar motores de búsqueda onion
// Requiere: Tor corriendo en localhost:9050
// reqwest ya compilado con feature "socks"
// ──────────────────────────────────────────────

use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{info, warn};

/// Resultado de búsqueda en Tor/Ahmia
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TorSearchResult {
    pub title: String,
    pub url: String,
    pub snippet: String,
    pub source: String, // "ahmia", "onion_search"
}

/// Resultado de health check de Tor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TorStatus {
    pub tor_running: bool,
    pub external_ip: Option<String>,
    pub proxy: String,
}

/// 🌑 Cliente de búsqueda en Tor (Ahmia)
pub struct TorSearch {
    proxy_url: String,
    client: reqwest::Client,
}

impl Default for TorSearch {
    fn default() -> Self {
        Self::new()
    }
}

impl TorSearch {
    pub fn new() -> Self {
        info!("🌑 [TOR-SEARCH] Inicializando cliente Tor...");

        let proxy_url = "socks5h://127.0.0.1:9050".to_string();
        let proxy = reqwest::Proxy::all(&proxy_url).unwrap_or_else(|_| {
            warn!("🌑 [TOR-SEARCH] No se pudo configurar proxy SOCKS5, usando conexión directa");
            // Fallback: sin proxy
            reqwest::Proxy::all("http://127.0.0.1:9050").unwrap()
        });

        let client = reqwest::Client::builder()
            .proxy(proxy)
            .timeout(Duration::from_secs(30))
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
            .build()
            .unwrap_or_default();

        Self {
            proxy_url: proxy_url.to_string(),
            client,
        }
    }

    /// Busca en Ahmia (buscador de .onion)
    /// URL: https://ahmia.fi/search/?q={query}
    pub async fn search_ahmia(&self, query: &str) -> Vec<TorSearchResult> {
        info!("🌑 [TOR-SEARCH] Buscando en Ahmia: {}", query);

        let encoded_query = urlencoding(query);
        let url = format!("https://ahmia.fi/search/?q={}", encoded_query);

        match self.client.get(&url).send().await {
            Ok(resp) => {
                if resp.status().is_success() {
                    let html = resp.text().await.unwrap_or_default();
                    self.parse_ahmia_results(&html, query)
                } else {
                    warn!("🌑 [TOR-SEARCH] Ahmia respondió HTTP {}", resp.status());
                    // Fallback: buscar directamente en Brave los .onion
                    self.search_onion_brave(query).await
                }
            }
            Err(e) => {
                warn!("🌑 [TOR-SEARCH] Error conectando a Ahmia: {}", e);
                // Fallback: buscar directamente en Brave los .onion
                self.search_onion_brave(query).await
            }
        }
    }

    /// Busca dominios .onion relacionados via Brave Search (fallback)
    async fn search_onion_brave(&self, query: &str) -> Vec<TorSearchResult> {
        info!(
            "🌑 [TOR-SEARCH] Fallback: buscando .onion en Brave: {}",
            query
        );

        let brave_key = std::env::var("BRAVE_API_KEY").unwrap_or_default();
        if brave_key.is_empty() {
            warn!("🌑 [TOR-SEARCH] BRAVE_API_KEY no configurada para fallback");
            return Vec::new();
        }

        let search_query = format!("{} site:onion OR site:.onion OR ahmia", query);

        match self
            .client
            .get("https://api.search.brave.com/res/v1/web/search")
            .header("X-Subscription-Token", &brave_key)
            .header("Accept", "application/json")
            .query(&[("q", search_query.as_str()), ("count", "10")])
            .send()
            .await
        {
            Ok(resp) => {
                if resp.status().is_success() {
                    let data: serde_json::Value = resp.json().await.unwrap_or_default();
                    let mut results = Vec::new();

                    if let Some(web) = data
                        .get("web")
                        .and_then(|w| w.get("results"))
                        .and_then(|r| r.as_array())
                    {
                        for r in web {
                            let title = r.get("title").and_then(|t| t.as_str()).unwrap_or("");
                            let url = r.get("url").and_then(|u| u.as_str()).unwrap_or("");
                            let snippet =
                                r.get("description").and_then(|d| d.as_str()).unwrap_or("");

                            results.push(TorSearchResult {
                                title: title.to_string(),
                                url: url.to_string(),
                                snippet: snippet.to_string(),
                                source: "brave_onion_fallback".to_string(),
                            });
                        }
                    }

                    results
                } else {
                    Vec::new()
                }
            }
            Err(e) => {
                warn!("🌑 [TOR-SEARCH] Error en Brave fallback: {}", e);
                Vec::new()
            }
        }
    }

    /// Verifica el estado de Tor
    pub async fn check_tor_status(&self) -> TorStatus {
        info!("🌑 [TOR-SEARCH] Verificando estado de Tor...");

        // Intentar conectar a check.torproject.org via proxy Tor
        let tor_running = match self
            .client
            .get("https://check.torproject.org/")
            .timeout(Duration::from_secs(10))
            .send()
            .await
        {
            Ok(resp) => {
                let html = resp.text().await.unwrap_or_default();
                // Si Tor está funcionando, la página muestra un mensaje de congratulación
                html.contains("Congratulations") || html.contains("Thank you for using Tor")
            }
            Err(_) => false,
        };

        // Obtener IP externa via Tor
        let external_ip = if tor_running {
            match self
                .client
                .get("https://api.ipify.org?format=json")
                .timeout(Duration::from_secs(10))
                .send()
                .await
            {
                Ok(resp) => {
                    let data: serde_json::Value = resp.json().await.unwrap_or_default();
                    data.get("ip")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                }
                Err(_) => None,
            }
        } else {
            None
        };

        TorStatus {
            tor_running,
            external_ip,
            proxy: self.proxy_url.clone(),
        }
    }

    /// Busca en múltiples fuentes onion y consolida
    pub async fn search_deep(&self, query: &str) -> Vec<TorSearchResult> {
        let mut results = Vec::new();

        // 1. Ahmia
        let ahmia_results = self.search_ahmia(query).await;
        results.extend(ahmia_results);

        // 2. Si no hay resultados de Ahmia, intentar Brave fallback
        if results.is_empty() {
            let brave_results = self.search_onion_brave(query).await;
            results.extend(brave_results);
        }

        results
    }

    // ─── Privados ────────────────────────────────

    fn parse_ahmia_results(&self, html: &str, _query: &str) -> Vec<TorSearchResult> {
        let mut results = Vec::new();

        // Parsear resultados de Ahmia (formato HTML simple)
        // Buscar enlaces a .onion con descripciones
        let re = regex::Regex::new(r#"<a[^>]*href="([^"]*onion[^"]*)"[^>]*>([^<]*)</a>"#).ok();

        if let Some(re) = re {
            for cap in re.captures_iter(html) {
                let url = cap.get(1).map(|m| m.as_str()).unwrap_or("");
                let title = cap.get(2).map(|m| m.as_str()).unwrap_or("");

                if !url.is_empty() {
                    // Buscar snippet cercano
                    let snippet = Self::extract_nearby_snippet(html, url);

                    results.push(TorSearchResult {
                        title: Self::clean_html(title),
                        url: url.to_string(),
                        snippet: Self::clean_html(&snippet),
                        source: "ahmia".to_string(),
                    });
                }
            }
        }

        results
    }

    fn extract_nearby_snippet(html: &str, url: &str) -> String {
        if let Some(pos) = html.find(url) {
            let start = pos.saturating_sub(200);
            let end = (pos + url.len() + 200).min(html.len());
            let context = &html[start..end];
            // Intentar extraer texto entre tags cercanos
            let re = regex::Regex::new(r">([^<]{10,200})<").ok();
            if let Some(re) = re {
                for cap in re.captures_iter(context) {
                    let text = cap.get(1).map(|m| m.as_str()).unwrap_or("");
                    let clean = text.trim();
                    if clean.len() > 20 && !clean.contains('<') {
                        return clean.to_string();
                    }
                }
            }
            context.to_string()
        } else {
            String::new()
        }
    }

    fn clean_html(text: &str) -> String {
        let re = regex::Regex::new(r"<[^>]*>").ok();
        match re {
            Some(re) => re.replace_all(text, "").trim().to_string(),
            None => text.to_string(),
        }
    }
}

/// URL encoding simple para queries
fn urlencoding(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
            ' ' => '+'.to_string(),
            other => format!("%{:02X}", other as u32),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_urlencoding() {
        assert_eq!(urlencoding("hello world"), "hello+world");
        assert_eq!(urlencoding("test/123"), "test%2F123");
    }

    #[test]
    fn test_clean_html() {
        let cleaned = TorSearch::clean_html("<b>hello</b> <i>world</i>");
        assert_eq!(cleaned, "hello world");
    }

    #[tokio::test]
    async fn test_tor_search_creation() {
        let search = TorSearch::new();
        // Solo verificar que se crea sin errores
        let status = search.check_tor_status().await;
        // No checkeamos tor_running porque depende del sistema
    }
}
