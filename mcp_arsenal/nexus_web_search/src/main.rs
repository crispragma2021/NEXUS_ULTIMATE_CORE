use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, BufReader as TokioBufReader};
use tracing::{info, error, Level};
use tracing_subscriber::FmtSubscriber;

#[derive(Serialize, Deserialize, Debug)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub snippet: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WebSearchResponse {
    pub query: String,
    pub results: Vec<SearchResult>,
}

async fn perform_brave_search(query: &str) -> Result<Vec<SearchResult>> {
    // Compatible con BRAVE_API_KEY (usada por omega_search.cjs) y BRAVE_SEARCH_API_KEY
    let api_key = std::env::var("BRAVE_API_KEY")
        .or_else(|_| std::env::var("BRAVE_SEARCH_API_KEY"))
        .context("BRAVE_API_KEY no configurada en las variables de entorno.")?;

    let encoded_query = urlencoding::encode(query);
    let url = format!("https://api.search.brave.com/res/v1/web/search?q={}", encoded_query);

    let client = reqwest::Client::new();
    let res = client.get(&url)
        .header("Accept", "application/json")
        .header("X-Subscription-Token", api_key)
        .send()
        .await?
        .error_for_status()?;

    let body: serde_json::Value = res.json().await?;

    let mut results: Vec<SearchResult> = Vec::new();

    if let Some(web_results) = body["web"]["results"].as_array() {
        for item in web_results {
            if let (Some(title), Some(url), Some(snippet)) = (
                item["title"].as_str(),
                item["url"].as_str(),
                item["description"].as_str(), // Brave Search usa 'description' para el snippet
            ) {
                results.push(SearchResult {
                    title: title.to_string(),
                    url: url.to_string(),
                    snippet: snippet.to_string(),
                });
            }
        }
    }
    
    Ok(results)
}


#[tokio::main]
async fn main() -> Result<()> {
    // Inicializar tracing
    let _subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_target(false)
        .init();

    info!("🚀 NEXUS Web Search MCP iniciado.");

    // Esperar a que se escriba el query en stdin
    let stdin = tokio::io::stdin();
    let mut reader = TokioBufReader::new(stdin);
    let mut line = String::new();

    reader.read_line(&mut line).await.context("Error leyendo de stdin")?;
    let query = line.trim();

    if query.is_empty() {
        error!("No se proporcionó un query en stdin.");
        return Ok(());
    }

    info!("🔎 Realizando búsqueda web para: \"{}\"", query);

    match perform_brave_search(query).await {
        Ok(results) => {
            let response = WebSearchResponse { query: query.to_string(), results };
            println!("{}", serde_json::to_string(&response)?);
        },
        Err(e) => {
            error!("Error durante la búsqueda web: {:?}", e);
            eprintln!("{{\"error\": \"Error en búsqueda web: {}\"}}", e);
        },
    }

    Ok(())
}
