use axum::{
    routing::{delete, get, post},
    Router,
};
use tower_http::{cors::CorsLayer, services::ServeDir};
use crate::state::AppStateArc;
use crate::api::*;

pub fn create_router(state: AppStateArc) -> Router {
    Router::new()
        // ─── Trading ───
        .route("/api/health", get(api_health))
        .route("/api/precio", get(api_precio))
        .route("/api/ordenes", get(api_ordenes).post(api_crear_orden))
        .route("/api/senales", get(api_senales))
        .route("/api/auto-trading", get(api_auto_trading))
        .route("/api/auto-trading/estado", get(api_auto_trading_estado))
        .route("/api/limite-operaciones", get(api_limite_operaciones).post(api_limite_operaciones))
        .route("/api/cartera", get(api_cartera))
        .route("/api/cartera/establecer", post(api_establecer_balance))
        .route("/api/pensamientos", get(api_pensamientos))
        .route("/api/configurar-exchange", post(api_configurar_exchange))
        .route("/api/real-status", get(api_real_status))
        .route("/api/modo-real", post(api_modo_real))
        
        // ─── Predicción ML (NEXUS v3.0) ───
        .route("/api/prediccion/reporte", get(api_prediccion_reporte))
        .route("/api/prediccion/analizar", get(api_prediccion_analizar))
        
        // ─── Energía (cadena maestra) ───
        .route("/api/energia/estado", get(api_energia_estado))
        
        // ─── Futures USDT-M ───
        .route("/api/futures/configurar", post(api_futures_configurar))
        .route("/api/futures/modo", post(api_futures_modo))
        .route("/api/futures/orden", post(api_futures_orden))
        .route("/api/futures/orden/close", post(api_futures_cerrar_posicion))
        .route("/api/futures/posiciones", get(api_futures_posiciones))
        .route("/api/futures/balance", get(api_futures_balance))
        .route("/api/futures/leverage", post(api_futures_leverage))
        .route("/api/futures/modo-hedge", post(api_futures_modo_hedge))
        .route("/api/futures/ordenes", delete(api_futures_cancelar_todas))
        .route("/api/futures/ordenes-abiertas", get(api_futures_ordenes_abiertas))
        .route("/api/futures/trades/{symbol}", get(api_futures_trades))
        .route("/api/futures/snapshot/{symbol}", get(api_futures_snapshot))
        .route("/api/futures/loop", post(api_futures_loop))
        .route("/api/futures/loop/estado", get(api_futures_loop_estado))
        .route("/api/futures/simulacion", get(api_futures_simulacion).post(api_futures_simulacion))
        
        // ─── WebSocket ───
        .route("/ws", get(ws_handler))
        
        // ─── Frontend estático ───
        .nest_service("/", ServeDir::new("frontend/dist"))
        .layer(CorsLayer::permissive())
        .with_state(state)
}
