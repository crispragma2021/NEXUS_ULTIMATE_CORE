use axum::{
    extract::Path,
    http::StatusCode,
    routing::{delete, get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddress;
use tower_http::cors::CorsLayer;

// --- ESTRUCTURAS SERDE DE ENTIDADES ---
#[derive(Serialize, Deserialize, Clone)]
pub struct User {
    pub id: usize,
    pub name: String,
    pub email: String,
    pub created_at: String,
}

#[derive(Deserialize)]
pub struct CreateUserDto {
    pub name: String,
    pub email: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Order {
    pub id: usize,
    pub user_id: usize,
    pub amount: f64,
    pub status: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct KpiItem {
    pub title: String,
    pub value: String,
    pub change: String,
    pub trend: String,
}

// --- SERVIDOR PRINCIPAL AXUM ---
#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = Router::new()
        // Health
        .route("/api/v1/health", get(health_handler))
        // Users CRUD
        .route("/api/v1/users", get(list_users_handler).post(create_user_handler))
        .route("/api/v1/users/:id", delete(delete_user_handler))
        // Orders CRUD
        .route("/api/v1/orders/recent", get(list_orders_handler))
        // Analytics KPIs
        .route("/api/v1/analytics/kpis", get(kpis_handler))
        .layer(CorsLayer::permissive());

    let addr = SocketAddress::from(([127, 0, 0, 1], 8080));
    println!("🚀 [RUST AXUM] Servidor backend dinámico ejecutándose en http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// --- HANDLERS ---
async fn health_handler() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "service": "NEXUS Axum Sovereign Engine",
        "timestamp": chrono_like_timestamp()
    }))
}

async fn list_users_handler() -> Json<Vec<User>> {
    Json(vec![
        User { id: 101, name: "Cris Pragma".to_string(), email: "cris@nexus.core".to_string(), created_at: "2026-09-16".to_string() },
        User { id: 102, name: "NEXUS Agent".to_string(), email: "agent@nexus.core".to_string(), created_at: "2026-09-16".to_string() },
    ])
}

async fn create_user_handler(Json(payload): Json<CreateUserDto>) -> (StatusCode, Json<User>) {
    let new_user = User {
        id: 103,
        name: payload.name,
        email: payload.email,
        created_at: chrono_like_timestamp(),
    };
    (StatusCode::CREATED, Json(new_user))
}

async fn delete_user_handler(Path(id): Path<usize>) -> StatusCode {
    println!("🗑️ Eliminando usuario con ID: {}", id);
    StatusCode::NO_CONTENT
}

async fn list_orders_handler() -> Json<Vec<Order>> {
    Json(vec![
        Order { id: 501, user_id: 101, amount: 250.00, status: "completed".to_string() },
        Order { id: 502, user_id: 102, amount: 1280.50, status: "processing".to_string() },
    ])
}

async fn kpis_handler() -> Json<Vec<KpiItem>> {
    Json(vec![
        KpiItem { title: "Ingresos Totales".to_string(), value: "$45,231.89".to_string(), change: "+20.1% vs mes pasado".to_string(), trend: "up".to_string() },
        KpiItem { title: "Suscripciones Activas".to_string(), value: "2,350".to_string(), change: "+180.1% crecimiento".to_string(), trend: "up".to_string() },
    ])
}

fn chrono_like_timestamp() -> String {
    "2026-09-16T13:40:00Z".to_string()
}
