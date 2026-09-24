// ──────────────────────────────────────────────
// 👻 SHADOW CRAWL CLIENT — Cliente HTTP para Cortex-Scout
// Re-delegado a ShadowCrawlAPI para unificación y failover SOTA
// ──────────────────────────────────────────────

use crate::infra::shadowcrawl::ShadowCrawlAPI;
use serde::{Deserialize, Serialize};
use tracing::info;

/// Puerto por defecto del servidor ShadowCrawl (cortex-scout)
const SHADOWCRAWL_DEFAULT_PORT: u16 = 5000;

/// Resultado individual de búsqueda web
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShadowSearchResult {
    pub url: String,
    pub title: String,
    pub snippet: String,
}

/// Respuesta del endpoint /scrape
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShadowScrapeResponse {
    pub url: String,
    pub title: String,
    pub content: String,
    pub word_count: u32,
}

/// Cliente unificado para búsqueda y scraping OSINT
pub struct ShadowCrawlClient {
    api: ShadowCrawlAPI,
}

impl Default for ShadowCrawlClient {
    fn default() -> Self {
        Self::new()
    }
}

impl ShadowCrawlClient {
    /// Crea un cliente apuntando a la configuración estándar
    pub fn new() -> Self {
        Self {
            api: ShadowCrawlAPI::from_env(),
        }
    }

    /// Crea un cliente con URL base personalizada
    pub fn new_with_url(base_url: String) -> Self {
        Self {
            api: ShadowCrawlAPI::new(&base_url, None, None, None),
        }
    }

    /// Verifica si el servicio de búsqueda está disponible
    pub async fn is_healthy(&self) -> bool {
        self.api.is_healthy().await
    }

    /// Busca en la web usando ShadowCrawlAPI con failover
    pub async fn search(&self, query: &str) -> anyhow::Result<Vec<ShadowSearchResult>> {
        info!("👻 [SHADOWCRAWL-CLIENT] Buscando con motor unificado: {}", query);
        let results = self.api.search(query).await?;
        Ok(results
            .into_iter()
            .map(|r| ShadowSearchResult {
                url: r.url,
                title: r.title,
                snippet: r.snippet,
            })
            .collect())
    }

    /// Scrapea una URL usando ShadowCrawlAPI
    pub async fn scrape(&self, url: &str) -> anyhow::Result<ShadowScrapeResponse> {
        info!("👻 [SHADOWCRAWL-CLIENT] Scrapeando con motor unificado: {}", url);
        let resp = self.api.scrape(url).await?;
        Ok(ShadowScrapeResponse {
            url: resp.url,
            title: resp.title,
            content: resp.content,
            word_count: resp.word_count as u32,
        })
    }
}
