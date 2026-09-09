// ──────────────────────────────────────────────
// 👻 SHADOW CRAWL CLIENT — Cliente HTTP para Cortex-Scout
// Se conecta al servidor local de ShadowCrawl (127.0.0.1:5000)
// Proporciona búsqueda web y scraping de alto rendimiento
// NO requiere dependencia externa — usa reqwest::Client estándar
// ──────────────────────────────────────────────

use serde::{Deserialize, Serialize};
use tracing::{info, warn};

/// Puerto por defecto del servidor ShadowCrawl (cortex-scout)
const SHADOWCRAWL_DEFAULT_PORT: u16 = 5000;

/// Resultado individual de búsqueda web
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShadowSearchResult {
    pub url: String,
    pub title: String,
    pub snippet: String,
}

/// Respuesta del endpoint /search
#[derive(Debug, Deserialize)]
struct ShadowSearchResponse {
    results: Vec<ShadowSearchResult>,
}

/// Respuesta del endpoint /scrape
#[derive(Debug, Deserialize)]
struct ShadowScrapeResponse {
    pub url: String,
    pub title: String,
    pub content: String,
    pub word_count: u32,
}

/// Cliente para el servidor ShadowCrawl local
pub struct ShadowCrawlClient {
    client: reqwest::Client,
    base_url: String,
}

impl Default for ShadowCrawlClient {
    fn default() -> Self {
        Self::new()
    }
}

impl ShadowCrawlClient {
    /// Crea un cliente apuntando a localhost:5000
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .connect_timeout(std::time::Duration::from_secs(3))
                .build()
                .expect("ShadowCrawlClient: Cliente HTTP válido"),
            base_url: format!("http://127.0.0.1:{}", SHADOWCRAWL_DEFAULT_PORT),
        }
    }

    /// Crea un cliente con URL base personalizada
    pub fn new_with_url(base_url: String) -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .connect_timeout(std::time::Duration::from_secs(3))
                .build()
                .expect("ShadowCrawlClient: Cliente HTTP válido"),
            base_url,
        }
    }

    /// Verifica si ShadowCrawl está corriendo
    pub async fn is_healthy(&self) -> bool {
        match self
            .client
            .get(format!("{}/health", self.base_url))
            .send()
            .await
        {
            Ok(resp) => resp.status().is_success(),
            Err(_) => false,
        }
    }

    /// Busca en la web usando ShadowCrawl
    pub async fn search(&self, query: &str) -> anyhow::Result<Vec<ShadowSearchResult>> {
        info!("👻 [SHADOWCRAWL] Buscando: {}", query);

        let resp = self
            .client
            .post(format!("{}/search", self.base_url))
            .json(&serde_json::json!({ "query": query }))
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("ShadowCrawl search error HTTP {}: {}", status, body);
        }

        let data: ShadowSearchResponse = resp.json().await?;
        info!(
            "👻 [SHADOWCRAWL] {} resultados para: {}",
            data.results.len(),
            query
        );
        Ok(data.results)
    }

    /// Scrapea una URL usando ShadowCrawl
    pub async fn scrape(&self, url: &str) -> anyhow::Result<ShadowScrapeResponse> {
        info!("👻 [SHADOWCRAWL] Scrapeando: {}", url);

        let resp = self
            .client
            .post(format!("{}/scrape", self.base_url))
            .json(&serde_json::json!({ "url": url }))
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("ShadowCrawl scrape error HTTP {}: {}", status, body);
        }

        let data: ShadowScrapeResponse = resp.json().await?;
        info!(
            "👻 [SHADOWCRAWL] Scrapeado: {} ({} palabras)",
            data.title, data.word_count
        );
        Ok(data)
    }
}
