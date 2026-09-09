// ──────────────────────────────────────────────
// 🦁 BRAVE SEARCH CLIENT — Búsqueda web via Brave Search API
// 2000 consultas/mes gratis. Sin CAPTCHA. Resultados estructurados.
// Documentación: https://api.search.brave.com/app/documentation/web-search
// ──────────────────────────────────────────────

use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{info, warn};

/// Resultado individual de Brave Search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BraveSearchResult {
    pub title: String,
    pub url: String,
    pub snippet: String,
    pub source: String,
    pub is_local: bool,
}

impl BraveSearchResult {
    fn from_brave_web(result: BraveWebResult) -> Self {
        Self {
            title: result.title.unwrap_or_default(),
            url: result.url.unwrap_or_default(),
            snippet: result.description.unwrap_or_default(),
            source: "brave".to_string(),
            is_local: result.is_local.unwrap_or(false),
        }
    }

    fn from_brave_news(result: BraveNewsResult) -> Self {
        Self {
            title: result.title.unwrap_or_default(),
            url: result.url.unwrap_or_default(),
            snippet: result.description.unwrap_or_default(),
            source: "brave-news".to_string(),
            is_local: false,
        }
    }
}

// ── Estructuras de respuesta Brave Search API ──

#[derive(Debug, Deserialize)]
struct BraveSearchResponse {
    web: Option<BraveWebResponse>,
    news: Option<BraveNewsResponse>,
    #[serde(rename = "query")]
    _query: Option<BraveQueryInfo>,
}

#[derive(Debug, Deserialize)]
struct BraveWebResponse {
    results: Vec<BraveWebResult>,
}

#[derive(Debug, Deserialize)]
struct BraveWebResult {
    title: Option<String>,
    url: Option<String>,
    description: Option<String>,
    #[serde(rename = "is_local")]
    is_local: Option<bool>,
    #[serde(rename = "is_source_local")]
    _is_source_local: Option<bool>,
    #[serde(rename = "age")]
    _age: Option<String>,
}

#[derive(Debug, Deserialize)]
struct BraveNewsResponse {
    results: Vec<BraveNewsResult>,
}

#[derive(Debug, Deserialize)]
struct BraveNewsResult {
    title: Option<String>,
    url: Option<String>,
    description: Option<String>,
    #[serde(rename = "age")]
    _age: Option<String>,
    #[serde(rename = "source")]
    _source: Option<String>,
}

#[derive(Debug, Deserialize)]
struct BraveQueryInfo {
    original: String,
}

/// 🦁 Cliente para Brave Search API
pub struct BraveSearchClient {
    client: reqwest::Client,
    api_key: String,
}

impl Default for BraveSearchClient {
    fn default() -> Self {
        Self::new()
    }
}

impl BraveSearchClient {
    /// Crea un nuevo cliente Brave Search, leyendo la API key de BRAVE_API_KEY en environment
    pub fn new() -> Self {
        let api_key = std::env::var("BRAVE_API_KEY").unwrap_or_else(|_| {
            warn!("⚠️ BRAVE_API_KEY no configurada. Brave Search no funcionará.");
            String::new()
        });

        Self {
            client: reqwest::Client::builder()
                .user_agent("NEXUS-OSINT/1.0")
                .timeout(Duration::from_secs(15))
                .build()
                .expect("BraveSearchClient: Cliente HTTP válido"),
            api_key,
        }
    }

    /// Crea un cliente con API key explícita
    pub fn new_with_key(api_key: String) -> Self {
        Self {
            client: reqwest::Client::builder()
                .user_agent("NEXUS-OSINT/1.0")
                .timeout(Duration::from_secs(15))
                .build()
                .expect("BraveSearchClient: Cliente HTTP válido"),
            api_key,
        }
    }

    /// Verifica si la API key está configurada
    pub fn is_configured(&self) -> bool {
        !self.api_key.is_empty()
    }

    /// Busca en la web usando Brave Search API
    /// Retorna hasta `count` resultados (máximo 20 por request)
    pub async fn search(&self, query: &str, count: u8) -> anyhow::Result<Vec<BraveSearchResult>> {
        if self.api_key.is_empty() {
            anyhow::bail!("BRAVE_API_KEY no configurada");
        }

        let count = count.min(20).max(1);
        info!(
            "🦁 [BRAVE-SEARCH] Buscando: '{}' ({} resultados)",
            query, count
        );

        let resp = self
            .client
            .get("https://api.search.brave.com/res/v1/web/search")
            .header("X-Subscription-Token", &self.api_key)
            .header("Accept", "application/json")
            .header("Accept-Encoding", "gzip")
            .query(&[("q", query), ("count", &count.to_string())])
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Brave Search API error HTTP {}: {}", status, body);
        }

        let data: BraveSearchResponse = resp.json().await?;
        let mut results: Vec<BraveSearchResult> = Vec::new();

        // Resultados web
        if let Some(web) = data.web {
            for r in web.results {
                results.push(BraveSearchResult::from_brave_web(r));
            }
        }

        // Resultados de noticias como bonus
        if let Some(news) = data.news {
            for r in news.results {
                results.push(BraveSearchResult::from_brave_news(r));
            }
        }

        info!(
            "🦁 [BRAVE-SEARCH] {} resultados para: '{}'",
            results.len(),
            query
        );
        Ok(results)
    }

    /// Busca con dork-like query (preserva operadores avanzados)
    pub async fn search_dork(&self, query: &str) -> anyhow::Result<Vec<BraveSearchResult>> {
        // Brave Search soporta operadores: site:, intitle:, inurl:, ext:, etc.
        self.search(query, 15).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_brave_search_client_creation() {
        let client = BraveSearchClient::new_with_key("test-key".to_string());
        assert!(client.is_configured());
    }

    #[test]
    fn test_brave_search_client_no_key() {
        let client = BraveSearchClient::new_with_key(String::new());
        assert!(!client.is_configured());
    }
}
