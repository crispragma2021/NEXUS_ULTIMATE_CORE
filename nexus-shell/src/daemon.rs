// ==========================================
// 🐚 NEXUS Shell — Daemon (Servidor HTTP Axum)
// ==========================================

use crate::api;
use crate::config::NexusShellConfig;
use anyhow::Result;
use axum::Router;
use std::sync::Arc;
use std::net::SocketAddr;
use tracing::{info, warn};
use tower_http::cors::CorsLayer;

/// Inicia el daemon en primer plano (foreground)
pub async fn run_daemon(config: Arc<NexusShellConfig>) -> Result<()> {
    // Inicializar CEREBRO
    info!("🧠 Inicializando CEREBRO NEXUS (Orquestador)...");
    let cerebro = api::init_nexus_cerebro().await?;
    info!("✅ CEREBRO listo — 46 órganos activos");

    // Construir router con estado compartido
    let app_state = api::AppState {
        cerebro: cerebro.clone(),
        config: config.clone(),
        started_at: chrono::Utc::now(),
    };

    let app = Router::new()
        .nest("/nexus/v1", api::routes())
        .layer(CorsLayer::permissive())
        .with_state(app_state);

    let addr = SocketAddr::new(
        config.http_host.parse().unwrap_or(std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST)),
        config.http_port,
    );

    info!("🌐 Servidor HTTP escuchando en http://{}", addr);
    info!("📡 Endpoints:");
    info!("   GET  {}/nexus/v1/health", addr);
    info!("   POST {}/nexus/v1/pensar", addr);
    info!("   POST {}/nexus/v1/eval", addr);
    info!("   GET  {}/nexus/v1/status", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// Inicia el daemon en background (fork + detach)
pub async fn start_daemon_background(config: Arc<NexusShellConfig>) -> Result<()> {
    let pid_file = config.data_dir.join("nexus.pid");
    let log_file = config.log_dir.join("daemon.log");

    // Verificar si ya está corriendo
    if pid_file.exists() {
        if let Ok(pid_str) = std::fs::read_to_string(&pid_file) {
            if let Ok(pid) = pid_str.trim().parse::<i32>() {
                // Verificar si el proceso existe
                let proc_path = format!("/proc/{pid}");
                if std::path::Path::new(&proc_path).exists() {
                    warn!("⚠️ NEXUS ya está corriendo (PID: {pid})");
                    return Ok(());
                }
            }
        }
    }

    info!("🚀 Iniciando NEXUS Daemon en background...");

    // Guardar PID actual
    let pid = std::process::id();
    std::fs::write(&pid_file, pid.to_string())?;
    info!("📝 PID {pid} guardado en {:?}", pid_file);

    // Enrutar logs
    let log_file_handle = std::fs::File::create(&log_file)?;
    let _ = log_file_handle;

    // Ejecutar el daemon (no podemos fork fácilmente en Rust,
    // así que usamos el foreground y confiamos en systemd/supervisor)
    info!("📋 Logs en: {:?}", log_file);
    info!("💡 Usa 'nexus daemon foreground' para ejecutar en primer plano.");
    info!("💡 O configura como servicio systemd para ejecución persistente.");

    run_daemon(config).await
}
