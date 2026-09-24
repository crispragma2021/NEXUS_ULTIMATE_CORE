use serde::{Deserialize, Serialize};
use dashmap::DashMap;
use parking_lot::RwLock;
use std::sync::Arc;
use tokio::sync::broadcast;
use crate::prediccion;
use crate::nexus_futures;

// ─── Tipos de datos (Trading) ─────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TickMercado {
    pub simbolo: String,
    pub precio: f64,
    pub volumen: f64,
    pub timestamp: i64,
    pub compra: f64,
    pub venta: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Orden {
    pub id: String,
    pub simbolo: String,
    pub lado: String,
    pub tipo: String,
    pub cantidad: f64,
    pub precio: Option<f64>,
    pub estado: String,
    pub timestamp: i64,
    pub razon_nexus: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SenalTrading {
    pub simbolo: String,
    pub accion: String,
    pub confianza: f64,
    pub precio_entrada: f64,
    pub precio_stop_loss: f64,
    pub precio_take_profit: f64,
    pub razonamiento: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cartera {
    pub usd: f64,
    pub nvda: f64,
    pub aapl: f64,
}

#[derive(Debug, Deserialize)]
pub struct OrdenRequest {
    pub simbolo: String,
    pub lado: String,
    pub tipo: String,
    pub cantidad: f64,
    pub precio: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct ApiKeyRequest {
    pub api_key: String,
    pub secret_key: String,
    pub exchange: String,
}

// ─── Estado compartido ────────────────────────────────────────────────────────

pub const MAX_OPERACIONES_DEFECTO: u32 = 60;
pub const LIMITE_MIN: u32 = 1;
pub const LIMITE_MAX: u32 = 500;
pub const FRACCION_POR_OPERACION: f64 = 0.04;

pub struct AppState {
    pub ordenes: RwLock<Vec<Orden>>,
    pub senales: RwLock<Vec<SenalTrading>>,
    // Replaced Mutex<HashMap> with DashMap for concurrent high-speed reads/writes
    pub precio_actual: DashMap<String, TickMercado>,
    pub modo_auto: RwLock<bool>,
    pub modo_real: RwLock<bool>,
    pub cartera: RwLock<Cartera>,
    pub pensamientos: RwLock<Vec<String>>,
    pub operaciones_realizadas: RwLock<u32>,
    pub max_operaciones: RwLock<u32>,
    pub analizador: DashMap<String, prediccion::AnalizadorCompleto>,
    
    // Future client / Simulators (RwLock is better here too)
    pub futures_client: RwLock<Option<Arc<nexus_futures::FuturesClient>>>,
    pub futures_sim: RwLock<Option<Arc<nexus_futures::FuturesSimulator>>>,
    pub futures_sim_activo: RwLock<bool>,
    pub futures_modo: RwLock<bool>,
    pub futures_loop_activo: RwLock<bool>,
    pub futures_loop_telemetry: RwLock<Option<Arc<tokio::sync::Mutex<serde_json::Value>>>>,
    pub mercado_broadcast: broadcast::Sender<String>,
}

#[derive(Clone)]
pub struct AppStateArc {
    pub inner: Arc<AppState>,
}

impl AppStateArc {
    pub fn new() -> Self {
        let (tx, _rx) = broadcast::channel::<String>(512);
        
        Self {
            inner: Arc::new(AppState {
                ordenes: RwLock::new(Vec::new()),
                senales: RwLock::new(Vec::new()),
                precio_actual: DashMap::new(),
                modo_auto: RwLock::new(false),
                modo_real: RwLock::new(false),
                cartera: RwLock::new(Cartera {
                    usd: 10_000.0,
                    nvda: 0.0,
                    aapl: 0.0,
                }),
                pensamientos: RwLock::new(vec![String::from("🤖 Terminal Autónoma de NEXUS iniciada. Esperando mercado...")]),
                operaciones_realizadas: RwLock::new(0),
                max_operaciones: RwLock::new(MAX_OPERACIONES_DEFECTO),
                analizador: DashMap::new(),
                futures_client: RwLock::new(None),
                futures_sim: RwLock::new(Some(Arc::new(nexus_futures::FuturesSimulator::new(10_000.0)))),
                futures_sim_activo: RwLock::new(false),
                futures_modo: RwLock::new(false),
                futures_loop_activo: RwLock::new(false),
                futures_loop_telemetry: RwLock::new(None),
                mercado_broadcast: tx,
            }),
        }
    }
}
