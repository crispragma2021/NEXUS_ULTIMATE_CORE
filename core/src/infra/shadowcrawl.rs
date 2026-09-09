// ============================================================================
// 🕵️ SHADOWCRAWL — Motor de Búsqueda y Scraping Multi-Proveedor (OMEGA)
// ============================================================================
// Absorbido de: legacy/nexus-orquestador/src/tentaculos/mod.rs (ShadowCrawlAPI)
// Propósito: Búsqueda semántica con failover entre Exa, Tavily y proxy local.
//            Scraping de URLs con proxy local ShadowCrawl.
// ============================================================================

use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{error, info, warn};

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub snippet: String,
    pub score: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResponse {
    pub results: Vec<SearchResult>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScrapeResponse {
    pub title: String,
    pub content: String,
    pub word_count: usize,
    pub url: String,
}

// Estructuras de Exa API
#[derive(Debug, Serialize, Deserialize)]
struct ExaSearchRequest {
    query: String,
    #[serde(rename = "numResults")]
    num_results: usize,
    #[serde(rename = "type")]
    search_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ExaResult {
    title: Option<String>,
    url: String,
    score: Option<f32>,
    text: Option<String>,
    highlights: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ExaSearchResponse {
    results: Vec<ExaResult>,
}

// Estructuras de Tavily API
#[derive(Debug, Serialize, Deserialize)]
struct TavilySearchRequest {
    api_key: String,
    query: String,
    max_results: usize,
    search_depth: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct TavilyResult {
    title: String,
    url: String,
    content: String,
    score: f32,
}

#[derive(Debug, Serialize, Deserialize)]
struct TavilySearchResponse {
    results: Vec<TavilyResult>,
}

/// 🕵️ ShadowCrawl: Motor de búsqueda con failover automático.
pub struct ShadowCrawlAPI {
    client: Client,
    base_url: String,
    exa_key: Option<String>,
    tavily_key: Option<String>,
    firecrawl_key: Option<String>,
}

impl ShadowCrawlAPI {
    pub fn new(
        base_url: &str,
        exa_key: Option<String>,
        tavily_key: Option<String>,
        firecrawl_key: Option<String>,
    ) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(60))
                .build()
                .unwrap_or_default(),
            base_url: base_url.trim_end_matches('/').to_string(),
            exa_key,
            tavily_key,
            firecrawl_key,
        }
    }

    pub async fn is_healthy(&self) -> bool {
        if self.exa_key.is_some() || self.tavily_key.is_some() {
            return true;
        }
        match self
            .client
            .get(format!("{}/health", self.base_url))
            .send()
            .await
        {
            Ok(resp) => resp.status() == StatusCode::OK,
            Err(_) => false,
        }
    }

    pub async fn search(&self, query: &str) -> anyhow::Result<Vec<SearchResult>> {
        // 1. Intentar Exa API
        if let Some(ref key) = self.exa_key {
            if !key.is_empty() {
                info!(
                    "🔍 [SHADOWCRAW-EXA] Buscando semánticamente en Exa: {}",
                    query
                );
                match self.search_exa(query, key).await {
                    Ok(results) => {
                        info!("✅ [SHADOWCRAW-EXA] Búsqueda completada con éxito.");
                        return Ok(results);
                    }
                    Err(e) => {
                        warn!(
                            "⚠️ [SHADOWCRAW-EXA] Error: {}. Reintentando con fallback...",
                            e
                        );
                    }
                }
            }
        }

        // 2. Intentar Tavily API
        if let Some(ref key) = self.tavily_key {
            if !key.is_empty() {
                info!("🔍 [SHADOWCRAW-TAVILY] Buscando en Tavily: {}", query);
                match self.search_tavily(query, key).await {
                    Ok(results) => {
                        info!("✅ [SHADOWCRAW-TAVILY] Búsqueda completada con éxito.");
                        return Ok(results);
                    }
                    Err(e) => {
                        warn!(
                            "⚠️ [SHADOWCRAW-TAVILY] Error: {}. Reintentando con fallback...",
                            e
                        );
                    }
                }
            }
        }

        // 3. Fallback a ShadowCrawl local proxy
        info!("🔍 [SHADOWCRAW-LOCAL] Buscando en proxy local: {}", query);
        let resp = self
            .client
            .post(format!("{}/search", self.base_url))
            .json(&serde_json::json!({ "query": query }))
            .send()
            .await?;

        if resp.status() != StatusCode::OK {
            let err = resp.text().await?;
            error!("❌ [SHADOWCRAW-LOCAL] Error: {}", err);
            anyhow::bail!("ShadowCrawl search error: {}", err);
        }

        let body: SearchResponse = resp.json().await?;
        Ok(body.results)
    }

    async fn search_exa(&self, query: &str, key: &str) -> anyhow::Result<Vec<SearchResult>> {
        let req_body = ExaSearchRequest {
            query: query.to_string(),
            num_results: 5,
            search_type: "auto".to_string(),
        };

        let resp = self
            .client
            .post("https://api.exa.ai/search")
            .header("x-api-key", key)
            .header("Content-Type", "application/json")
            .json(&req_body)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let err_text = resp.text().await.unwrap_or_default();
            anyhow::bail!("Exa API returned status {}: {}", status, err_text);
        }

        let exa_resp: ExaSearchResponse = resp.json().await?;
        let results = exa_resp
            .results
            .into_iter()
            .map(|r| {
                let snippet = r
                    .text
                    .or_else(|| r.highlights.map(|h| h.join(" | ")))
                    .unwrap_or_default();
                SearchResult {
                    title: r.title.unwrap_or_default(),
                    url: r.url,
                    snippet,
                    score: r.score.unwrap_or(0.0),
                }
            })
            .collect();

        Ok(results)
    }

    async fn search_tavily(&self, query: &str, key: &str) -> anyhow::Result<Vec<SearchResult>> {
        let req_body = TavilySearchRequest {
            api_key: key.to_string(),
            query: query.to_string(),
            max_results: 5,
            search_depth: "basic".to_string(),
        };

        let resp = self
            .client
            .post("https://api.tavily.com/search")
            .header("Content-Type", "application/json")
            .json(&req_body)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let err_text = resp.text().await.unwrap_or_default();
            anyhow::bail!("Tavily API returned status {}: {}", status, err_text);
        }

        let tavily_resp: TavilySearchResponse = resp.json().await?;
        let results = tavily_resp
            .results
            .into_iter()
            .map(|r| SearchResult {
                title: r.title,
                url: r.url,
                snippet: r.content,
                score: r.score,
            })
            .collect();

        Ok(results)
    }

    pub async fn scrape(&self, url: &str) -> anyhow::Result<ScrapeResponse> {
        info!("📄 [SHADOWCRAW] Scrapeando: {}", url);
        let resp = self
            .client
            .post(format!("{}/scrape", self.base_url))
            .json(&serde_json::json!({ "url": url }))
            .send()
            .await?;

        if resp.status() != StatusCode::OK {
            anyhow::bail!("ShadowCrawl scrape error: {}", resp.status());
        }

        let body: ScrapeResponse = resp.json().await?;
        Ok(body)
    }
}
