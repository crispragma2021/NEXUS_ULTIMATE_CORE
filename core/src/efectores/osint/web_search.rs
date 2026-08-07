// ──────────────────────────────────────────────
// 🌐 WEB SEARCH CLIENT — Wrapper sobre ShadowCrawlAPI (Exa + Tavily)
// Expone la búsqueda multi-provider como efector OSINT estándar.
// Failover automático: Exa → Tavily → proxy local
// ──────────────────────────────────────────────

use crate::infra::shadowcrawl::{SearchResult, ShadowCrawlAPI};
use serde::{Deserialize, Serialize};
use tracing::info;

/// Resultado unificado de búsqueda web multi-proveedor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSearchResult {
    pub title: String,
    pub url: String,
    pub snippet: String,
    pub score: f32,
    pub source: String,
}

impl From<SearchResult> for WebSearchResult {
    fn from(sr: SearchResult) -> Self {
        Self {
            title: sr.title,
            url: sr.url,
            snippet: sr.snippet,
            score: sr.score,
            source: String::new(), // se asigna después según el provider
        }
    }
}

/// 🌐 Cliente de búsqueda web multi-proveedor
pub struct WebSearchClient {
    api: ShadowCrawlAPI,
}

impl Default for WebSearchClient {
    fn default() -> Self {
        Self::new()
    }
}

impl WebSearchClient {
    /// Crea un nuevo WebSearchClient cargando API keys de environment
    pub fn new() -> Self {
        let exa_key = std::env::var("EXA_API_KEY").ok();
        let tavily_key = std::env::var("TAVILY_API_KEY").ok();

        info!(
            "🌐 [WEB-SEARCH] Inicializado. Exa: {}, Tavily: {}",
            if exa_key.is_some() { "✅" } else { "❌" },
            if tavily_key.is_some() { "✅" } else { "❌" }
        );

        Self {
            api: ShadowCrawlAPI::new("http://127.0.0.1:5000", exa_key, tavily_key, None),
        }
    }

    /// Verifica si al menos un proveedor está disponible
    pub fn is_configured(&self) -> bool {
        // ShadowCrawlAPI::is_healthy verifica si hay API keys o proxy local
        futures::executor::block_on(self.api.is_healthy())
    }

    /// Busca en la web con failover automático entre proveedores
    pub async fn search(&self, query: &str) -> anyhow::Result<Vec<WebSearchResult>> {
        info!("🌐 [WEB-SEARCH] Buscando: '{}'", query);

        let results = self.api.search(query).await?;
        let mut web_results: Vec<WebSearchResult> = results.into_iter().map(|r| r.into()).collect();

        // Asignar source basado en disponibilidad (no podemos saber exactamente cuál proveyó,
        // pero ShadowCrawlAPI intenta Exa → Tavily → local)
        for r in &mut web_results {
            if r.source.is_empty() {
                r.source = "web-search".to_string();
            }
        }

        info!(
            "🌐 [WEB-SEARCH] {} resultados para: '{}'",
            web_results.len(),
            query
        );
        Ok(web_results)
    }

    /// Busca múltiples queries en paralelo
    pub async fn search_multi(&self, queries: &[&str]) -> anyhow::Result<Vec<WebSearchResult>> {
        use futures::future::join_all;

        let futures: Vec<_> = queries.iter().map(|q| self.search(q)).collect();
        let results = join_all(futures).await;

        let mut all = Vec::new();
        for r in results {
            if let Ok(res) = r {
                all.extend(res);
            }
        }

        Ok(all)
    }
}
