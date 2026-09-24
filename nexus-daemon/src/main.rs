mod chat;
mod fs;
mod os;
mod pty;
mod vision;

use axum::{
    extract::ws::WebSocketUpgrade,
    response::Response,
    routing::get,
    Router,
};
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tracing::{error, info};

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    // Configurar sistema de logs
    tracing_subscriber::fmt::init();
    info!("🚀 Iniciando NEXUS Daemon Core (Orquestador Agéntico Soberano)...");

    let app_state = match chat::AppState::nuevo() {
        Ok(state) => state,
        Err(e) => {
            error!("❌ Error inicializando AppState de NEXUS Agent: {e}");
            std::process::exit(1);
        }
    };

    // Construir la aplicación con rutas
    let app = Router::new()
        .nest("/api", chat::router(app_state)) // Montar rutas de chat (/api/consultar)
        .nest("/api/fs", fs::router()) // Montar rutas del sistema de archivos
        .nest("/api/vision", vision::router()) // Montar rutas de visión nativa de NEXUS Agent
        .nest("/api/os", os::router()) // Montar rutas de control de S.O.
        .route("/api/terminal/ws", get(ws_handler))
        .layer(CorsLayer::permissive()); // Permitir que el frontend React se conecte

    // Iniciar servidor en el puerto 43210
    let addr = SocketAddr::from(([127, 0, 0, 1], 43210));
    info!("🌐 NEXUS Daemon escuchando en {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn ws_handler(ws: WebSocketUpgrade) -> Response {
    info!("🔌 Nueva conexión WebSocket solicitada desde la UI");
    ws.on_upgrade(pty::handle_pty_socket)
}
