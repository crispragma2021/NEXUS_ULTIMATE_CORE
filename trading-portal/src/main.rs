mod state;
mod db;
mod error;
mod api;
mod routes;
mod prediccion;
mod nexus_futures;


use std::net::SocketAddr;
use std::sync::Arc;
use tracing::{info, warn};
use axum::extract::ws::Message as AxumMessage;

pub use state::*;
pub use error::AppError;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "nexus_tr=info".into()),
        )
        .init();

    info!("🤖 [NEXUS-TR v2.0] Trading Portal");

    // Initialize Database
    if let Err(e) = db::init_db() {
        warn!("⚠️ [NEXUS-TR] Failed to initialize DB: {}. Orders won't persist.", e);
    }

    let state = AppStateArc::new();

    // Load persisted wallet
    if let Ok(Some(cartera)) = db::load_cartera() {
        *state.inner.cartera.write() = cartera;
    }
    // Load persisted orders
    if let Ok(ordenes) = db::load_orders() {
        *state.inner.ordenes.write() = ordenes;
    }

    // ═══ Fase A — Mercado vivo 24/7 (independiente de clientes WS) ═══
    {
        let (mercado_tx, mut mercado_rx) = tokio::sync::mpsc::unbounded_channel::<String>();
        let simbolos = vec!["NVDA", "AAPL", "MSFT", "AMZN", "META", "TSLA", "BTCUSDT"];
        for simbolo in simbolos {
            let tx = mercado_tx.clone();
            let sym = simbolo.to_string();
            tokio::spawn(async move {
                loop {
                    api::conectar_binance_ws(&sym, tx.clone()).await;
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                }
            });
        }
        
        let state_feed = state.inner.clone();
        let broadcast_tx = state_feed.mercado_broadcast.clone();
        tokio::spawn(async move {
            while let Some(msg) = mercado_rx.recv().await {
                api::procesar_tick_mercado(&state_feed, &msg).await;
                let _ = broadcast_tx.send(msg);
            }
        });
    }

    // ═══ Fase C — Auto-inicialización del backend futures ═══
    {
        let (api_key, sec_key) = api::load_binance_keys();
        let claves_presentes = api_key.is_some() && sec_key.is_some();
        if claves_presentes {
            let client = Arc::new(nexus_futures::FuturesClient::new(
                api_key.clone().unwrap_or_default(),
                sec_key.clone().unwrap_or_default(),
            ));
            
            let verif = tokio::time::timeout(
                std::time::Duration::from_secs(5),
                client.account_info(),
            )
            .await;
            match verif {
                Ok(Ok(_)) => {
                    *state.inner.futures_client.write() = Some(client);
                    *state.inner.futures_sim_activo.write() = false;
                    *state.inner.futures_modo.write() = true;
                    info!("🚀 [FUTURES] Cliente real VERIFICADO al arrancar (API válida).");
                }
                Ok(Err(e)) => {
                    info!("⚠️ [FUTURES] Claves presentes pero API inválida: {}. Activando SIMULADOR paper.", e);
                    let sim = Arc::new(nexus_futures::FuturesSimulator::new(10_000.0));
                    *state.inner.futures_sim.write() = Some(Arc::clone(&sim));
                    *state.inner.futures_sim_activo.write() = true;
                    *state.inner.futures_modo.write() = true;
                    info!("🧪 [FUTURES] SIMULADOR activado con $10,000.00 de paper trading (claves inválidas).");
                }
                Err(_) => {
                    info!("⚠️ [FUTURES] Tiempo agotado verificando API. Activando SIMULADOR paper.");
                    let sim = Arc::new(nexus_futures::FuturesSimulator::new(10_000.0));
                    *state.inner.futures_sim.write() = Some(Arc::clone(&sim));
                    *state.inner.futures_sim_activo.write() = true;
                    *state.inner.futures_modo.write() = true;
                }
            }
        } else {
            let sim = Arc::new(nexus_futures::FuturesSimulator::new(10_000.0));
            *state.inner.futures_sim.write() = Some(Arc::clone(&sim));
            *state.inner.futures_sim_activo.write() = true;
            *state.inner.futures_modo.write() = true;
            info!("🧪 [FUTURES] Sin API real (o marcador YOUR_*): SIMULADOR activado con $10,000.00 de paper trading.");
        }
    }

    let addr = SocketAddr::from(([0, 0, 0, 0], 42210));
    let app = routes::create_router(state);

    let socket = socket2::Socket::new(
        socket2::Domain::IPV4,
        socket2::Type::STREAM,
        Some(socket2::Protocol::TCP),
    )
    .expect("socket2: fallo al crear socket");
    socket.set_reuse_address(true).expect("socket2: fallo reuseaddr");
    #[cfg(target_os = "linux")]
    socket.set_reuse_port(true).expect("socket2: fallo reuseport");
    socket.set_nonblocking(true).expect("socket2: fallo set_nonblocking");
    let sock_addr: socket2::SockAddr = addr.into();
    socket.bind(&sock_addr).expect("socket2: fallo bind");
    socket.listen(1024).expect("socket2: fallo listen");
    let std_listener: std::net::TcpListener = socket.into();
    let listener = tokio::net::TcpListener::from_std(std_listener)
        .expect("tokio: fallo al convertir listener");

    info!("🌐 [NEXUS-TR] Portal Unificado escuchando en http://{} (lógica + ui)", addr);

    axum::serve(listener, app).await.unwrap();
}
