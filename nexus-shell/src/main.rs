// ==========================================
// 🐚 NEXUS Shell — Entry Point
// ==========================================
// El cuerpo soberano del Orquestador.
// Uso: nexus <comando> [argumentos...]

mod api;
mod cli;
mod config;
mod daemon;

use anyhow::{Context, Result};
use std::sync::Arc;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<()> {
    // Inicializar tracing (logs estructurados)
    let _subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_target(false)
        .init();

    // Cargar configuración
    config::init_config();
    let cfg = Arc::new(config::global_config().clone());

    // Parsear argumentos
    let args: Vec<String> = std::env::args().collect();
    let cmd_args = if args.len() > 1 { &args[1..] } else { &[] };

    let command = match cli::Command::parse(cmd_args) {
        Ok(cmd) => cmd,
        Err(e) => {
            eprintln!("❌ Error: {e}");
            cli::print_help();
            std::process::exit(1);
        }
    };

    // Ejecutar comando
    match command {
        cli::Command::DaemonStart => {
            info!("🚀 Iniciando NEXUS Daemon...");
            daemon::start_daemon_background(cfg).await?;
        }
        cli::Command::DaemonStop => {
            let pid_file = cfg.data_dir.join("nexus.pid");
            if pid_file.exists() {
                let pid_str = std::fs::read_to_string(&pid_file)
                    .context("Leyendo PID")?;
                if let Ok(pid) = pid_str.trim().parse::<i32>() {
                    // Enviar SIGTERM usando kill command
                    let output = std::process::Command::new("kill")
                        .arg(&pid.to_string())
                        .output()
                        .context("Ejecutando kill")?;
                    if output.status.success() {
                        let _ = std::fs::remove_file(&pid_file);
                        println!("✅ NEXUS Daemon detenido (PID: {pid})");
                    } else {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        eprintln!("❌ Error deteniendo: {stderr}");
                    }
                }
            } else {
                println!("ℹ️ NEXUS Daemon no está corriendo");
            }
        }
        cli::Command::DaemonStatus => {
            let pid_file = cfg.data_dir.join("nexus.pid");
            if pid_file.exists() {
                let pid_str = std::fs::read_to_string(&pid_file)
                    .context("Leyendo PID")?;
                println!("✅ NEXUS Daemon está activo (PID: {})", pid_str.trim());
            } else {
                println!("ℹ️ NEXUS Daemon no está corriendo");
                println!("   Usa 'nexus daemon start' para iniciarlo");
            }
        }
        cli::Command::DaemonForeground => {
            info!("🔥 NEXUS Daemon en primer plano...");
            daemon::run_daemon(cfg).await?;
        }
        cli::Command::Eval(prompt) => {
            cli::print_header();
            println!("🤔 Evaluando: {}", &prompt[..prompt.len().min(80)]);
            println!();

            let cerebro = api::init_nexus_cerebro().await?;
            let respuesta = cerebro.responder(&prompt).await;

            println!("🧠 Respuesta:");
            println!("{}", respuesta);
        }
        cli::Command::Pensar { modo, prompt } => {
            cli::print_header();
            println!("🤔 Modo: {modo}");
            // Truncar prompt para display
            let display = if prompt.len() > 80 {
                // Recorte seguro por límite de carácter UTF-8 (límite del 77º char)
                let boundary = prompt.char_indices().take(77).fold(0, |acc, (i, _)| i);
                format!("{}...", &prompt[..boundary.max(1).min(prompt.len())])
            } else {
                prompt.clone()
            };
            println!("📝 Prompt: {display}");
            println!();

            let cerebro = api::init_nexus_cerebro().await?;

            let prompt_final = match modo.as_str() {
                "razonar" | "razonamiento" => {
                    format!("[RAZONAMIENTO LÓGICO] {}\n\nAnaliza paso a paso, con estructura lógica, evidencia y conclusiones.", prompt)
                }
                "crear" | "creativo" => {
                    format!("[CREATIVIDAD] {}\n\nGenera ideas originales, metáforas y soluciones no convencionales.", prompt)
                }
                "debug" | "depurar" => {
                    format!("[DEBUG TÉCNICO] {}\n\nDiagnostica el problema, identifica causas raíz y propone soluciones específicas.", prompt)
                }
                _ => prompt.clone(),
            };

            let respuesta = cerebro.responder(&prompt_final).await;

            println!("🧠 Respuesta ({modo}):");
            println!("{}", respuesta);
        }
        cli::Command::V0Generate { prompt, session_id } => {
            cli::print_header();
            println!("🧬 Generando UI con pipeline multi-agente v0...");
            println!("📝 Prompt: {}", &prompt[..prompt.len().min(80)]);
            if let Some(sid) = &session_id {
                println!("🆔 Session: {sid}");
            }
            println!();

            let mut pipeline = nexus_ultimate_core::cerebro::v0::PipelineV0::nuevo(None);
            let resultado = pipeline.ejecutar_local(&prompt, session_id.as_deref());

            println!("✅ Pipeline limpio: {}", resultado.pipeline_limpio);
            println!("⚠️  Errores restantes: {}", resultado.errores_restantes);
            println!("📁 Archivos generados: {}", resultado.archivos_generados);
            println!("🆔 Session ID: {}", resultado.session_id);
            println!();
            println!("📂 Archivos finales:");
            for ruta in resultado.archivos_finales.keys() {
                println!("   - {ruta}");
            }
            println!();
            println!("📊 Telemetría:");
            println!("   Etapas: {}", resultado.telemetria.etapas.len());
            println!("   Gates fallidos: {}", resultado.telemetria.gates_fallidos);
            if !resultado.diff_summary.is_empty() {
                println!("   Diff: {}", resultado.diff_summary);
            }
        }
        cli::Command::Status => {
            cli::print_header();
            println!("📊 Estado del sistema:");
            println!("   Versión:    {}", env!("CARGO_PKG_VERSION"));
            println!("   CEREBRO:    ✅ 46 órganos activos");
            println!("   Memoria:    SQLite FTS5 + Embeddings");
            println!("   Daemon:     {}",
                if cfg.data_dir.join("nexus.pid").exists() { "✅ activo" } else { "⏹️  detenido" }
            );

            // Mostrar configuración
            println!();
            println!("⚙️  Configuración:");
            println!("   Puerto:     {}", cfg.http_port);
            println!("   Host:       {}", cfg.http_host);
            println!("   Data dir:   {:?}", cfg.data_dir);
            println!("   Log dir:    {:?}", cfg.log_dir);
            println!("   Modo:       {:?}", cfg.mode);
        }
        cli::Command::Help => {
            cli::print_header();
            cli::print_help();
        }
    }

    Ok(())
}
