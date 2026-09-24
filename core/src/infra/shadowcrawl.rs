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

/// Purga parámetros de seguimiento (UTM, gclid, fbclid, ref_) de las URLs.
pub fn limpiar_url_tracking(url: &str) -> String {
    if let Ok(mut parsed) = url::Url::parse(url) {
        let params_a_remover = [
            "utm_source", "utm_medium", "utm_campaign", "utm_term",
            "utm_content", "gclid", "fbclid", "ref_", "srsltid", "ncid",
        ];
        let query_pairs: Vec<(String, String)> = parsed
            .query_pairs()
            .filter(|(k, _)| !params_a_remover.contains(&k.as_ref()))
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();

        parsed.set_query(None);
        if !query_pairs.is_empty() {
            let mut serializer = parsed.query_pairs_mut();
            for (k, v) in query_pairs {
                serializer.append_pair(&k, &v);
            }
        }
        parsed.to_string()
    } else {
        url.to_string()
    }
}

/// Extrae el texto principal de una página HTML colapsando etiquetas y ruido.
pub fn extraer_markdown_limpio(html: &str) -> String {
    use regex::Regex;

    // 1. Eliminar scripts, estilos, navegaciones y pie de página
    let re_script = Regex::new(r"(?is)<script[^>]*?>.*?</script>").unwrap();
    let re_style = Regex::new(r"(?is)<style[^>]*?>.*?</style>").unwrap();
    let re_nav = Regex::new(r"(?is)<nav[^>]*?>.*?</nav>").unwrap();
    let re_footer = Regex::new(r"(?is)<footer[^>]*?>.*?</footer>").unwrap();

    let sin_scripts = re_script.replace_all(html, "");
    let sin_estilos = re_style.replace_all(&sin_scripts, "");
    let sin_nav = re_nav.replace_all(&sin_estilos, "");
    let sin_footer = re_footer.replace_all(&sin_nav, "");

    // 2. Convertir etiquetas <h1..6> y <p> en saltos de línea
    let re_headers = Regex::new(r"(?i)<h[1-6][^>]*?>(.*?)</h[1-6]>").unwrap();
    let re_p = Regex::new(r"(?i)<p[^>]*?>(.*?)</p>").unwrap();
    let re_tags = Regex::new(r"<[^>]+>").unwrap();

    let headers_fmt = re_headers.replace_all(&sin_footer, "\n# $1\n");
    let p_fmt = re_p.replace_all(&headers_fmt, "\n$1\n");
    let sin_tags = re_tags.replace_all(&p_fmt, " ");

    // 3. Normalizar espacios y saltos de línea
    let re_spaces = Regex::new(r"[ \t]+").unwrap();

    let lines: Vec<String> = sin_tags
        .lines()
        .map(|l| re_spaces.replace_all(l.trim(), " ").to_string())
        .filter(|l| !l.is_empty())
        .collect();

    lines.join("\n")
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

    pub fn from_env() -> Self {
        let base_url = std::env::var("SHADOWCRAWL_URL")
            .unwrap_or_else(|_| "http://127.0.0.1:5000".to_string());
        let exa_key = std::env::var("EXA_API_KEY").ok();
        let tavily_key = std::env::var("TAVILY_API_KEY").ok();
        let firecrawl_key = std::env::var("FIRECRAWL_API_KEY").ok();
        Self::new(&base_url, exa_key, tavily_key, firecrawl_key)
    }
}

impl Default for ShadowCrawlAPI {
    fn default() -> Self {
        Self::from_env()
    }
}

impl ShadowCrawlAPI {

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
        let mut raw_results = self.search_internal(query).await?;
        
        // Limpiar parámetros tracking de todas las URLs devueltas
        for r in &mut raw_results {
            r.url = limpiar_url_tracking(&r.url);
        }

        Ok(raw_results)
    }

    async fn search_internal(&self, query: &str) -> anyhow::Result<Vec<SearchResult>> {
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

        // 3. Fallback a ShadowCrawl local proxy si responde
        info!("🔍 [SHADOWCRAW-LOCAL] Buscando en proxy local: {}", query);
        if let Ok(resp) = self
            .client
            .post(format!("{}/search", self.base_url))
            .json(&serde_json::json!({ "query": query }))
            .send()
            .await
        {
            if resp.status() == StatusCode::OK {
                if let Ok(body) = resp.json::<SearchResponse>().await {
                    return Ok(body.results);
                }
            }
        }

        // 4. Fallback Nativo a Brave Stealth Search (cero API keys, cero dependencias)
        info!("🔍 [SHADOWCRAW-BRAVE-STEALTH] Ejecutando búsqueda nativa en Brave Search: {}", query);
        self.search_brave_stealth(query).await
    }

    /// Búsqueda nativa de rescate en Brave Search en modo Stealth (0.00$ costo, 0 API keys).
    async fn search_brave_stealth(&self, query: &str) -> anyhow::Result<Vec<SearchResult>> {
        let q_encoded: String = url::form_urlencoded::byte_serialize(query.as_bytes()).collect();
        let url = format!("https://search.brave.com/search?q={}", q_encoded);
        let user_agent = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36";

        let resp = self
            .client
            .get(&url)
            .header("User-Agent", user_agent)
            .header("Accept-Language", "es-ES,es;q=0.9,en;q=0.8")
            .send()
            .await?;

        if !resp.status().is_success() {
            anyhow::bail!("Brave Search HTTP error: {}", resp.status());
        }

        let html = resp.text().await?;
        use regex::Regex;

        let re_link = Regex::new(r#"(?s)<a[^>]+href="([^"]+)"[^>]*class="[^"]*heading[^"]*"[^>]*>(.*?)</a>"#).unwrap();
        let mut results = Vec::new();

        for caps in re_link.captures_iter(&html).take(5) {
            let href = caps.get(1).map_or("", |m| m.as_str());
            let title_raw = caps.get(2).map_or("", |m| m.as_str());
            let title = extraer_markdown_limpio(title_raw);
            if !href.is_empty() && href.starts_with("http") {
                results.push(SearchResult {
                    title: title.clone(),
                    url: href.to_string(),
                    snippet: format!("Resultado nativo para {}", query),
                    score: 0.85,
                });
            }
        }

        if results.is_empty() {
            // Fallback genérico si los selectores cambian ligeramente
            results.push(SearchResult {
                title: format!("Búsqueda Brave para {}", query),
                url: url.clone(),
                snippet: "Resultado obtenido vía Brave Search Stealth.".to_string(),
                score: 0.7,
            });
        }

        Ok(results)
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

// ============================================================================
// 🧪 PRUEBAS UNITARIAS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_limpiar_url_tracking() {
        let url_sucia = "https://example.com/page?utm_source=google&utm_medium=cpc&gclid=12345&id=99";
        let url_limpia = limpiar_url_tracking(url_sucia);
        assert!(!url_limpia.contains("utm_source"));
        assert!(!url_limpia.contains("gclid"));
        assert!(url_limpia.contains("id=99"));
    }

    #[test]
    fn test_extraer_markdown_limpio() {
        let html = "<html><head><style>body { color: red; }</style></head><body><nav>Menu</nav><h1>Titulo Principal</h1><p>Texto de prueba con <b>negrita</b>.</p><footer>Pie</footer></body></html>";
        let md = extraer_markdown_limpio(html);
        assert!(md.contains("Titulo Principal"));
        assert!(md.contains("Texto de prueba con negrita"));
        assert!(!md.contains("color: red"));
        assert!(!md.contains("<p>"));
    }
}
