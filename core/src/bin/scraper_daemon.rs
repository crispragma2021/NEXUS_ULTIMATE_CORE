//! 🚀 Binario `scraper-daemon`: demonio de scraping en segundo plano.
//!
//! Uso:
//! ```
//! cargo run -p nexus_ultimate_core --bin scraper-daemon
//! cargo run -p nexus_ultimate_core --bin scraper-daemon -- --db scraper.db --poll-ms 5000
//! ```
//!
//! Flujo:
//! 1. Inicializa SQLite (crea esquema si no existe).
//! 2. Construye el pipeline (fetcher + ollama opcional + cloud opcional).
//! 3. Lanza el bucle principal.
//! 4. Espera SIGINT/SIGTERM para graceful shutdown.

use anyhow::{anyhow, Context, Result};
use nexus_ultimate_core::scraping::pipeline::cloud_adapter::CloudAdapter;
use nexus_ultimate_core::scraping::pipeline::daemon::{Daemon, DaemonConfig};
use nexus_ultimate_core::scraping::pipeline::db::PipelineDb;
use nexus_ultimate_core::scraping::pipeline::fetcher::Fetcher;
use nexus_ultimate_core::scraping::pipeline::ollama_client::{OllamaClient, OllamaConfig};
use nexus_ultimate_core::scraping::pipeline::pipeline::{Pipeline, PipelineConfig};
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{error, info, warn, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();

    // ── Logging ─────────────────────────────────────────────
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).context("configurando tracing")?;

    // ── Argumentos ──────────────────────────────────────────
    let mut db_path = PathBuf::from("scraper.db");
    let mut poll_ms: u64 = 5000;
    let mut ollama_model: Option<String> = None;
    let mut openrouter_keys: Vec<String> = Vec::new();

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--db" => {
                db_path = PathBuf::from(args.next().unwrap_or_else(|| "scraper.db".into()));
            }
            "--poll-ms" => {
                poll_ms = args.next().and_then(|v| v.parse().ok()).unwrap_or(5000);
            }
            "--ollama-model" => {
                ollama_model = args.next();
            }
            "--openrouter-key" => {
                if let Some(k) = args.next() {
                    openrouter_keys.push(k);
                }
            }
            "--help" | "-h" => {
                print_help();
                return Ok(());
            }
            _ => {
                warn!("[MAIN] argumento desconocido: {arg}");
            }
        }
    }

    // ── DB ──────────────────────────────────────────────────
    info!("🗄️ [MAIN] abriendo DB: {}", db_path.display());
    let db = Arc::new(PipelineDb::open(&db_path).context("abriendo DB")?);

    // ── Pipeline ─────────────────────────────────────────────
    let fetcher = Arc::new(Fetcher::new(Some(db.clone())).context("creando fetcher")?);

    // Ollama (tier-1) opcional.
    let ollama = match &ollama_model {
        Some(model) => {
            info!("🤖 [MAIN] Ollama configurado con modelo {model}");
            Some(Arc::new(
                OllamaClient::new(OllamaConfig {
                    model: model.clone(),
                    ..Default::default()
                })
                .context("creando cliente Ollama")?,
            ))
        }
        None => {
            warn!("🤖 [MAIN] sin Ollama (usa --ollama-model qwen2.5:7b)");
            None
        }
    };

    // Cloud (tier-2) opcional.
    let cloud = if !openrouter_keys.is_empty() {
        info!(
            "☁️ [MAIN] CloudAdapter con {} keys OpenRouter",
            openrouter_keys.len()
        );
        let adapter = CloudAdapter::new(vec![Box::new(
            nexus_ultimate_core::scraping::pipeline::cloud_adapter::OpenRouterProvider::new(
                openrouter_keys,
                "deepseek/deepseek-r1:free",
            )
            .map_err(|e| anyhow!(e))?,
        )]);
        Some(Arc::new(adapter))
    } else {
        warn!("☁️ [MAIN] sin CloudAdapter (usa --openrouter-key sk-...)");
        None
    };

    let pipeline = Arc::new(Pipeline::new(
        fetcher.clone(),
        ollama,
        cloud,
        Some(db.clone()),
        PipelineConfig::default(),
    ));

    // ── Daemon ───────────────────────────────────────────────
    let config = DaemonConfig {
        poll_interval_ms: poll_ms,
        ..Default::default()
    };
    let daemon = Daemon::new(config, db, pipeline, fetcher);

    info!("🚀 [MAIN] lanzando scraper-daemon (Ctrl+C para detener)");

    // Ejecutar loop + esperar señal en paralelo.
    tokio::select! {
        _ = daemon.run() => {
            info!("[MAIN] loop terminó");
        }
        res = daemon.wait_for_shutdown() => {
            if let Err(e) = res {
                error!("[MAIN] error en shutdown: {e}");
                std::process::exit(1);
            }
        }
    }

    Ok(())
}

fn print_help() {
    println!("🚀 scraper-daemon — demonio de scraping NEXUS");
    println!();
    println!("Uso:");
    println!("  scraper-daemon [opciones]");
    println!();
    println!("Opciones:");
    println!("  --db <path>            Ruta de la DB SQLite (default: scraper.db)");
    println!("  --poll-ms <n>          Intervalo de polling en ms (default: 5000)");
    println!("  --ollama-model <m>     Modelo Ollama local para tier-1 (ej. qwen2.5:7b)");
    println!("  --openrouter-key <k>   API key OpenRouter para tier-2 (repetible)");
    println!("  -h, --help             Muestra esta ayuda");
    println!();
    println!("Ejemplo:");
    println!("  scraper-daemon --db scraper.db --ollama-model qwen2.5:7b --openrouter-key sk-xxx");
}
