//! NEXUS Memory Guardian
//! Vigila la RAM cada 30 segundos y notifica al orquestador via HTTP
//! si la presión de memoria supera los umbrales definidos.

use reqwest::Client;
use serde_json::json;
use std::time::Duration;
use sysinfo::System;
use tokio::time::sleep;
use tracing::{error, info, warn};

const NEXUS_API: &str = "http://127.0.0.1:43211";
const LOG_PATH: &str = "/opt/NEXUS_ULTIMATE_CORE/logs/mem_guard.log";

/// Umbral de advertencia: menos de 3 GB libres
const WARN_THRESHOLD_KB: u64 = 3_000_000;
/// Umbral crítico: menos de 1.5 GB libres
const CRIT_THRESHOLD_KB: u64 = 1_500_000;

#[derive(Debug, Clone, Copy, PartialEq)]
enum AlertLevel {
    Ok,
    Warning,
    Critical,
}

impl AlertLevel {
    fn as_str(&self) -> &'static str {
        match self {
            AlertLevel::Ok => "ok",
            AlertLevel::Warning => "warning",
            AlertLevel::Critical => "critical",
        }
    }

    fn emoji(&self) -> &'static str {
        match self {
            AlertLevel::Ok => "✅",
            AlertLevel::Warning => "⚠️ ",
            AlertLevel::Critical => "🚨",
        }
    }
}

/// Lee el proceso que más RAM consume en este momento.
fn top_memory_process(sys: &System) -> String {
    sys.processes()
        .values()
        .max_by_key(|p| p.memory())
        .map(|p| {
            let name = p.name().to_string_lossy();
            let mb = p.memory() / 1024 / 1024;
            format!("{name} ({mb} MB)")
        })
        .unwrap_or_else(|| "desconocido".to_string())
}

/// Envía la alerta al API del orquestador.
async fn notify_orchestrator(client: &Client, level: AlertLevel, message: &str) {
    let payload = json!({
        "level": level.as_str(),
        "source": "mem_guard",
        "message": message,
    });

    match client
        .post(format!("{NEXUS_API}/internal/alert"))
        .json(&payload)
        .timeout(Duration::from_secs(3))
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => {
            info!("Alerta entregada al orquestador: {}", level.as_str());
        }
        Ok(resp) => {
            warn!("Orquestador respondió con status: {}", resp.status());
        }
        Err(e) => {
            // El orquestador puede estar caído — no es fatal para el guardián.
            warn!("No se pudo contactar al orquestador: {e}");
        }
    }
}

/// Escribe la alerta en el log soberano en disco.
fn write_log(level: AlertLevel, message: &str) {
    use std::io::Write;
    let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
    let line = format!("[{timestamp}] {} {}\n", level.emoji(), message);

    // Append al archivo de log
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(LOG_PATH)
    {
        let _ = file.write_all(line.as_bytes());
    }
}

#[tokio::main]
async fn main() {
    // Inicializar trazas al journal del sistema
    tracing_subscriber::fmt()
        .with_target(false)
        .with_thread_ids(false)
        .init();

    info!("🛡️  NEXUS Memory Guardian iniciado. Vigilando cada 30s...");
    info!("   Umbral WARNING  : < 3 GB libres");
    info!("   Umbral CRITICAL : < 1.5 GB libres");
    info!("   Destino alertas : {NEXUS_API}/internal/alert");

    let client = Client::new();
    let mut sys = System::new_all();
    let mut last_level = AlertLevel::Ok;

    loop {
        sys.refresh_memory();
        sys.refresh_processes(sysinfo::ProcessesToUpdate::All);

        let avail_kb = sys.available_memory() / 1024;
        let total_kb = sys.total_memory() / 1024;
        let used_kb = total_kb - avail_kb;

        let avail_gb = avail_kb as f64 / 1_048_576.0;
        let used_gb = used_kb as f64 / 1_048_576.0;

        let level = if avail_kb < CRIT_THRESHOLD_KB {
            AlertLevel::Critical
        } else if avail_kb < WARN_THRESHOLD_KB {
            AlertLevel::Warning
        } else {
            AlertLevel::Ok
        };

        // Solo notificar si el nivel cambió o sigue siendo crítico/warning
        if level != AlertLevel::Ok || last_level != level {
            let top = top_memory_process(&sys);
            let msg =
                format!("RAM libre: {avail_gb:.1}GB | Usada: {used_gb:.1}GB | Proceso top: {top}");

            match level {
                AlertLevel::Ok => {
                    if last_level != AlertLevel::Ok {
                        info!("✅ RAM normalizada. {msg}");
                    }
                }
                AlertLevel::Warning => {
                    warn!("⚠️  ADVERTENCIA MEMORIA — {msg}");
                    write_log(level, &msg);
                    notify_orchestrator(&client, level, &msg).await;
                }
                AlertLevel::Critical => {
                    error!("🚨 CRÍTICO MEMORIA — {msg}");
                    write_log(level, &msg);
                    notify_orchestrator(&client, level, &msg).await;
                }
            }
        }

        last_level = level;
        sleep(Duration::from_secs(30)).await;
    }
}
