use anyhow::{Context, Result};
use chromiumoxide::browser::{Browser, BrowserConfig};
use chromiumoxide::handler::viewport::Viewport;
use clap::Parser;
use futures::StreamExt;
use regex::Regex;
use serde_json::json;
use std::path::Path;

#[derive(Parser, Debug)]
#[command(author, version, about = "NEXUS Ghost-Scraper (Pure Rust)")]
struct Args {
    #[arg(short, long, default_value = "https://checkip.amazonaws.com")]
    url: String,

    #[arg(short, long)]
    proxy: Option<String>,

    #[arg(long, default_value_t = false)]
    check_ip: bool,

    #[arg(short, long, default_value = "evidence.png")]
    screenshot: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let mut args = Args::parse();

    if args.check_ip {
        args.url = "https://checkip.amazonaws.com".to_string();
    }

    println!(
        "🧪 [GHOST_ENGINE] Iniciando Transfixión Rust en: {}",
        args.url
    );

    let mut config_builder = BrowserConfig::builder()
        .no_sandbox()
        .window_size(1280, 800)
        .viewport(Viewport {
            width: 1280,
            height: 800,
            device_scale_factor: None,
            emulating_mobile: false,
            has_touch: false,
            is_landscape: false,
        });

    if let Some(p) = &args.proxy {
        println!("🕸️ [GHOST_ENGINE] Velo del Templo (Proxy): {}", p);
        config_builder = config_builder.arg(format!("--proxy-server={}", p));
    }

    let (mut browser, mut handler) =
        Browser::launch(config_builder.build().map_err(anyhow::Error::msg)?).await?;

    // Manejar eventos de Chrome en segundo plano
    let _handle = tokio::spawn(async move {
        while let Some(h) = handler.next().await {
            if h.is_err() {
                break;
            }
        }
    });

    let page = browser.new_page(&args.url).await?;

    // Esperar a que la página cargue sustancialmente
    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

    // Capturar evidencia
    println!("📸 [GHOST_ENGINE] Capturando Evidencia Visual...");
    let screenshot_data = page
        .screenshot(
            chromiumoxide::page::ScreenshotParams::builder()
                .full_page(true)
                .build(),
        )
        .await?;

    std::fs::write(&args.screenshot, screenshot_data)?;
    println!(
        "✅ [GHOST_ENGINE] Evidencia guardada en: {}",
        args.screenshot
    );

    // Si es FB o estamos analizando, extraer información básica
    let content = page.content().await?;
    let text = page
        .evaluate("document.body.innerText")
        .await?
        .into_value::<String>()?;

    if args.url.contains("facebook.com") {
        println!("🔍 [GHOST_ENGINE] Analizando hilos de relación...");

        let re_married = Regex::new(r"(?i)Casada con (.*)").unwrap();
        let re_from = Regex::new(r"(?i)De (.*)").unwrap();
        let re_lives = Regex::new(r"(?i)Vive en (.*)").unwrap();

        let relations = json!({
            "marriedTo": re_married.captures(&text).and_then(|c| c.get(1)).map(|m| m.as_str()).unwrap_or("No detectado"),
            "from": re_from.captures(&text).and_then(|c| c.get(1)).map(|m| m.as_str()).unwrap_or("No detectado"),
            "lives": re_lives.captures(&text).and_then(|c| c.get(1)).map(|m| m.as_str()).unwrap_or("No detectado"),
        });

        println!(
            "📊 [RESULTADO]:\n{}",
            serde_json::to_string_pretty(&relations)?
        );
    } else if args.check_ip {
        println!("✅ [NEXUS] IP Fantasma Confirmada: {}", text.trim());
    }

    // Persistir HTML para análisis forense profundo
    std::fs::write("last_audit.html", content)?;

    browser.close().await?;
    println!("🛡️ [GHOST_ENGINE] Perimetral Cerrado.");

    Ok(())
}
