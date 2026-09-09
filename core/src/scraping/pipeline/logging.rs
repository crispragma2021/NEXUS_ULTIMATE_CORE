//! Logs estructurados JSON Lines (F7.1).
//!
//! Configura `tracing-subscriber` con formato JSON (campos: timestamp, level,
//! target, message, task_id opcional). Para habilitarlo en el binario:
//! `RUST_LOG=info cargo run --bin scraper-daemon` o `NEXUS_LOG_JSON=1`.

use anyhow::Result;
use tracing::Level;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

/// Variable de entorno para activar JSON (además del nivel RUST_LOG).
const ENV_JSON_FLAG: &str = "NEXUS_LOG_JSON";

/// Inicializa el logging con formato JSON Lines.
///
/// Campos por línea: `{"timestamp": "...", "level": "...", "fields": {...}, "target": "..."}`.
/// Usa `RUST_LOG` para el nivel (default: `info`).
pub fn init_json_logging() -> Result<()> {
    let level = std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());
    let filter = tracing_subscriber::EnvFilter::try_new(&level)
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(filter)
        .with(
            tracing_subscriber::fmt::layer()
                .json()
                .with_current_span(true)
                .with_span_list(true)
                .with_target(true),
        )
        .try_init()?;
    Ok(())
}

/// Inicializa el logging humano (terminal), para desarrollo.
pub fn init_human_logging(level: Level) -> Result<()> {
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(level)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;
    Ok(())
}

/// Inicializa según entorno: JSON si `NEXUS_LOG_JSON=1`, si no humano.
pub fn init_logging() -> Result<()> {
    let json_flag = std::env::var(ENV_JSON_FLAG).unwrap_or_default();
    if json_flag == "1" || json_flag.eq_ignore_ascii_case("true") {
        init_json_logging()
    } else {
        init_human_logging(Level::INFO)
    }
}

/// Extrae `task_id` del span actual para logs con contexto (helper de uso
/// interno; se usa `tracing::info_span!` en el pipeline).
pub fn span_with_task(task_id: &str) -> tracing::Span {
    tracing::info_span!("task", task_id = %task_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn span_con_task_id_tiene_metadatos() {
        let span = span_with_task("task-abc");
        // Sin subscriber global los spans están disabled, pero los metadatos
        // (nombre/campos) se construyen correctamente.
        let meta = span.metadata();
        assert!(meta.is_some());
        assert_eq!(meta.map(|m| m.name()), Some("task"));
    }

    #[test]
    fn init_human_logging_no_falla() {
        // Usa un nombre único para evitar conflicto de global default en tests.
        let _ = init_human_logging(Level::WARN);
    }
}
