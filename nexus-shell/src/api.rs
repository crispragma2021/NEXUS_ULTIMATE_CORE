// ==========================================
// 🐚 NEXUS Shell — API REST Handlers
// ==========================================

use crate::config::NexusShellConfig;
use anyhow::Result;
use axum::{
    extract::{State, Json as JsonExtractor, ws::{WebSocket, WebSocketUpgrade, Message}},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use futures::{SinkExt, StreamExt};
use nix::pty::{openpty, OpenptyResult};
use nix::unistd::{execv, fork, ForkResult, setsid, dup2};
use nix::sys::wait::waitpid;
use nix::libc::{STDIN_FILENO, STDOUT_FILENO, STDERR_FILENO, TIOCSCTTY, close};
use std::ffi::{CString};
use std::os::unix::io::{AsRawFd, FromRawFd};
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::sync::mpsc;
use axum::response::IntoResponse;

use chrono::{DateTime, Utc};
use nexus_ultimate_core::cerebro::orquestador::Orquestador;
use nexus_ultimate_core::brain::hippocampus::ArtificialHippocampus;
use nexus_ultimate_core::autodiagnostico::sentinel_core::{SentinelCore, HealthReport, ProbeResult, HealthStatus};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{info, error};

// ==========================================
// Estado compartido
// ==========================================

#[derive(Clone)]
pub struct AppState {
    pub cerebro: Arc<CerebroHandle>,
    pub config: Arc<NexusShellConfig>,
    pub started_at: DateTime<Utc>,
}

// ==========================================
// Modelos de respuesta
// ==========================================

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: &'static str,
    pub uptime_seconds: i64,
    pub cerebro_activo: bool,
    pub organos: usize,
    pub started_at: String,
    pub health_report: HealthReport, // Nuevo campo para el reporte completo
}

#[derive(Serialize)]
pub struct StatusResponse {
    pub version: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub uptime_seconds: i64,
    pub cerebro: CerebroStatus,
}

#[derive(Serialize)]
pub struct CerebroStatus {
    pub activo: bool,
    pub organos: usize,
    pub ultimo_latido: String,
}

#[derive(Deserialize)]
pub struct PensarRequest {
    pub prompt: String,
    #[serde(default = "default_modo")]
    pub modo: String,
    #[serde(default = "default_modelo")]
    pub modelo: String,
}

fn default_modo() -> String {
    "auto".to_string()
}

fn default_modelo() -> String {
    "auto".to_string()
}

#[derive(Serialize)]
pub struct PensarResponse {
    pub respuesta: String,
    pub modo: String,
    pub modelo: String,
    pub proveedor: String,
    pub procesado_en: String,
}

/// Normaliza un id de modelo/proveedor del frontend a una etiqueta de proveedor.
fn proveedor_para_modelo(modelo: &str) -> &'static str {
    let m = modelo.to_ascii_lowercase();
    if m.contains("nexus-local") || m.contains("ollama") || m.contains("lmstudio") {
        "NEXUS Local"
    } else if m.contains("nexus-puro") || m == "puro" {
        "NEXUS Puro"
    } else if m.contains("vertex") {
        "Vertex AI"
    } else if m.contains("deepseek") {
        "DeepSeek"
    } else if m.contains("gemini") {
        "Gemini"
    } else if m.contains("groq") {
        "Groq"
    } else if m.contains("mistral") {
        "Mistral"
    } else if m.contains("claude") || m.contains("anthropic") {
        "Anthropic"
    } else if m.contains("bedrock") {
        "Amazon Bedrock"
    } else if m.contains("openai") {
        "OpenAI"
    } else if m.contains("openrouter") {
        "OpenRouter"
    } else {
        "Córtex OMEGA"
    }
}

#[derive(Deserialize)]
pub struct EvalRequest {
    pub prompt: String,
}

#[derive(Serialize)]
pub struct EvalResponse {
    pub respuesta: String,
}

#[derive(Deserialize)]
pub struct MemoriaSearchRequest {
    pub query: String,
    #[serde(default = "default_memoria_limit")]
    pub limit: usize,
    #[serde(default = "default_memoria_offset")]
    pub offset: usize,
}

fn default_memoria_limit() -> usize {
    10
}
fn default_memoria_offset() -> usize {
    0
}

#[derive(Serialize)]
pub struct MemoriaSearchResult {
    pub query: String,
    pub total_results: usize,
    pub results: Vec<MemoriaEntryResponse>,
    pub elapsed_ms: u64,
}

#[derive(Serialize)]
pub struct MemoriaEntryResponse {
    pub id: i64,
    pub score: f32,
    pub contenido: String,
    pub categoria: String,
    pub timestamp: String,
}

/// POST /nexus/v1/memoria/buscar
async fn handle_memoria_buscar(
    State(state): State<AppState>,
    JsonExtractor(req): JsonExtractor<MemoriaSearchRequest>,
) -> Json<MemoriaSearchResult> {
    let start = std::time::Instant::now();

    let cerebro_handle = state.cerebro.clone(); // Clonar Arc<CerebroHandle>
    let query = req.query.clone();
    let limit = req.limit;

    let raw_results = tokio::task::spawn_blocking(move || {
        let rt = tokio::runtime::Handle::current();
        rt.block_on(async move {
            let cerebro = CerebroHandle::get(); // Acceder al Orquestador
            cerebro.hippocampus.buscar_semantica(&query, limit) // Usar buscar_semantica
                .unwrap_or_else(|e| {
                    error!("Error buscando en memoria semántica: {}", e);
                    Vec::new()
                })
        })
    }).await.unwrap_or_default();


    let results: Vec<MemoriaEntryResponse> = raw_results
        .into_iter()
        .map(|(id, content, score)| { // Ajustar desestructuración a (i64, String, f32)
            MemoriaEntryResponse {
                id,
                score,
                contenido: content,
                categoria: "semantica".to_string(), // Categoría fija por ahora
                timestamp: chrono::Utc::now().to_rfc3339(), // Timestamp actual, ya que buscar_semantica no lo devuelve
            }
        })
        .collect();

    let elapsed = start.elapsed();

    Json(MemoriaSearchResult {
        query: req.query,
        total_results: results.len(),
        results,
        elapsed_ms: elapsed.as_millis() as u64,
    })
}

// ==========================================
// Inicialización del CEREBRO
// Mismo patrón que claws_mcp.rs: puntero crudo + OnceLock
// ==========================================

use std::sync::OnceLock;

/// Puntero thread-safe al Orquestador.
/// # Safety
/// El Orquestador se inicializa UNA vez y vive toda la vida del proceso.
/// Solo se accede de forma compartida e inmutable (&self).
struct CerebroPtr(*const Orquestador);
unsafe impl Send for CerebroPtr {}
unsafe impl Sync for CerebroPtr {}

static CEREBRO: OnceLock<CerebroPtr> = OnceLock::new();

/// Handle thread-safe al CEREBRO. Unit struct — zero-cost wrapper.
/// Es explícitamente Send + Sync porque accede al Orquestador vía puntero crudo.
pub struct CerebroHandle;

// Safety: CerebroHandle solo accede al CEREBRO global via OnceLock<CerebroPtr>,
// que ya es Send + Sync. El handle en sí es un ZST.
unsafe impl Send for CerebroHandle {}
unsafe impl Sync for CerebroHandle {}

impl CerebroHandle {
    /// Inicializa el CEREBRO (llamar una sola vez al inicio)
    pub async fn new() -> Result<Self> {
        let memoria_db = std::env::var("NEXUS_MEMORIA_DB")
            .unwrap_or_else(|_| "data/nexus_memoria.lance".to_string());

        let hippocampus = Arc::new(
            ArtificialHippocampus::new(None, None, &memoria_db)
        );
        let orquestador = Orquestador::new(hippocampus).await;
        let ptr = Box::into_raw(Box::new(orquestador));

        if CEREBRO.set(CerebroPtr(ptr as *const Orquestador)).is_err() {
            unsafe { drop(Box::from_raw(ptr)); }
            anyhow::bail!("CEREBRO ya inicializado");
        }

        info!("🧠 CEREBRO inicializado con 46 órganos");
        Ok(CerebroHandle)
    }

    fn get() -> &'static Orquestador {
        let ptr = CEREBRO.get().expect("CEREBRO no inicializado");
        unsafe { &*ptr.0 }
    }

    /// Responde a un prompt usando el CEREBRO.
    /// Ejecuta en un thread separado vía spawn_blocking para evitar
    /// problemas con futures no-Send del Orquestador.
    pub async fn responder(&self, prompt: &str) -> String {
        let prompt = prompt.to_owned();
        // Usamos spawn_blocking para obtener un contexto thread-safe
        // donde Orquestador::responder puede correr sin restricciones Send
        tokio::task::spawn_blocking(move || {
            let rt = tokio::runtime::Handle::current();
            rt.block_on(async { Self::get().responder(&prompt).await })
        })
        .await
        .unwrap_or_else(|e| format!("Error en CEREBRO: {e}"))
    }
}

/// Inicializa el CEREBRO (wrapper para uso externo)
pub async fn init_nexus_cerebro() -> Result<Arc<CerebroHandle>> {
    let handle = CerebroHandle::new().await?;
    Ok(Arc::new(handle))
}

// ==========================================
// Handlers HTTP
// ==========================================

/// GET /nexus/v1/health
async fn handle_health(
    State(state): State<AppState>,
) -> Json<HealthResponse> {
    let uptime = (Utc::now() - state.started_at).num_seconds();

    // Ejecutar el diagnóstico completo del SentinelCore
    let sentinel_core = SentinelCore::new();
    let health_report = sentinel_core.run_full_diagnostic().await;

    Json(HealthResponse {
        status: health_report.estado.to_string(), // Usar el estado del reporte
        version: env!("CARGO_PKG_VERSION"),
        uptime_seconds: uptime,
        cerebro_activo: true,
        organos: 46,
        started_at: state.started_at.to_rfc3339(),
        health_report,
    })
}

/// GET /nexus/v1/status
async fn handle_status(
    State(state): State<AppState>,
) -> Json<StatusResponse> {
    let uptime = (Utc::now() - state.started_at).num_seconds();
    Json(StatusResponse {
        version: env!("CARGO_PKG_VERSION"),
        name: "NEXUS Shell",
        description: "🐚 Cuerpo soberano del Orquestador — 46 órganos del CEREBRO",
        uptime_seconds: uptime,
        cerebro: CerebroStatus {
            activo: true,
            organos: 46,
            ultimo_latido: Utc::now().to_rfc3339(),
        },
    })
}

/// POST /nexus/v1/pensar
async fn handle_pensar(
    State(state): State<AppState>,
    JsonExtractor(req): JsonExtractor<PensarRequest>,
) -> Json<PensarResponse> {
    let start = std::time::Instant::now();

    let prompt_final = match req.modo.as_str() {
        "razonar" | "razonamiento" => {
            format!("[RAZONAMIENTO LÓGICO] {}\n\nAnaliza paso a paso, con estructura lógica, evidencia y conclusiones.", req.prompt)
        }
        "crear" | "creativo" => {
            format!("[CREATIVIDAD] {}\n\nGenera ideas originales, metáforas y soluciones no convencionales.", req.prompt)
        }
        "debug" | "depurar" => {
            format!("[DEBUG TÉCNICO] {}\n\nDiagnostica el problema, identifica causas raíz y propone soluciones específicas.", req.prompt)
        }
        _ => req.prompt.clone(),
    };

    let respuesta = state.cerebro.responder(&prompt_final).await;
    let elapsed = start.elapsed();

    let proveedor = proveedor_para_modelo(&req.modelo);

    Json(PensarResponse {
        respuesta,
        modo: req.modo,
        modelo: req.modelo,
        proveedor: proveedor.to_string(),
        procesado_en: format!("{:.2}s", elapsed.as_secs_f64()),
    })
}

/// POST /nexus/v1/eval (alias rápido)
async fn handle_eval(
    State(state): State<AppState>,
    JsonExtractor(req): JsonExtractor<EvalRequest>,
) -> Json<EvalResponse> {
    let respuesta = state.cerebro.responder(&req.prompt).await;
    Json(EvalResponse { respuesta })
}

/// GET /nexus/v1/terminal/ws
pub async fn handle_terminal_ws(
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade(handle_socket)
}

// Función principal para manejar la conexión WebSocket de la terminal
async fn handle_socket(socket: WebSocket) {
    let (mut ws_sender, mut ws_receiver) = socket.split();

    // 1. Crear el PTY (Pseudo-Terminal)
    let OpenptyResult { master, slave } = match openpty(None, None) {
        Ok(res) => res,
        Err(e) => {
            error!("Error abriendo PTY: {}", e);
            let _ = ws_sender.send(Message::Text(format!("Error abriendo PTY: {}", e))).await;
            return;
        }
    };
    
    // 2. Bifurcar el proceso y ejecutar el shell
    match unsafe { fork() } {
        Ok(ForkResult::Parent { child }) => {
            let mut pty_master = unsafe { tokio::fs::File::from_raw_fd(master.as_raw_fd()) };

            // Canales para comunicarse con la tarea de gestión del PTY
            let (pty_to_ws_tx, mut pty_to_ws_rx) = mpsc::channel::<Message>(100);
            let (ws_to_pty_tx, mut ws_to_pty_rx) = mpsc::channel::<Vec<u8>>(100);

            // Tarea que posee pty_master y realiza la E/S asíncrona
            tokio::spawn(async move {
                let mut buf = vec![0; 1024];
                loop {
                    tokio::select! {
                        read_res = pty_master.read(&mut buf) => {
                            match read_res {
                                Ok(0) => break, // PTY cerrado
                                Ok(n) => {
                                    if pty_to_ws_tx.send(Message::Text(String::from_utf8_lossy(&buf[..n]).to_string())).await.is_err() {
                                        error!("Error enviando salida de PTY a WebSocket");
                                        break;
                                    }
                                }
                                Err(e) => {
                                    error!("Error leyendo PTY maestro: {}", e);
                                    break;
                                }
                            }
                        }
                        Some(input_data) = ws_to_pty_rx.recv() => {
                            if pty_master.write_all(&input_data).await.is_err() {
                                error!("Error escribiendo a PTY maestro");
                                break;
                            }
                        }
                    }
                }
            });

            // Loop principal para gestionar la conexión WebSocket
            loop {
                tokio::select! {
                    // Leer del WebSocket y enviar a la tarea del PTY
                    ws_msg = ws_receiver.next() => {
                        match ws_msg {
                            Some(Ok(Message::Text(text))) => {
                                if ws_to_pty_tx.send(text.into_bytes()).await.is_err() {
                                    error!("Error enviando mensaje de WebSocket al PTY");
                                    break;
                                }
                            }
                            Some(Ok(Message::Close(_))) | None => {
                                info!("WebSocket cerrado por el cliente.");
                                break;
                            }
                            Some(Err(e)) => {
                                error!("Error en WebSocket: {}", e);
                                break;
                            }
                            _ => {} // Ignorar otros tipos de mensajes
                        }
                    }
                    // Leer del canal PTY y enviar al WebSocket
                    pty_output = pty_to_ws_rx.recv() => {
                        match pty_output {
                            Some(msg) => {
                                if ws_sender.send(msg).await.is_err() {
                                    error!("Error enviando mensaje de PTY a WebSocket");
                                    break;
                                }
                            }
                            None => { // Canal PTY cerrado, la tarea del PTY ha terminado
                                info!("Canal de salida de PTY cerrado.");
                                break;
                            }
                        }
                    }
                }
            }
            // Esperar a que el proceso hijo termine
            let _ = waitpid(child, None);
        }
        Ok(ForkResult::Child) => {
            // Código del proceso hijo (ejecuta el shell)
            setsid().unwrap(); // Crear nueva sesión
            
            // Asegurarse de que el PTY es el terminal de control
            unsafe { nix::libc::ioctl(slave.as_raw_fd(), TIOCSCTTY, 1); }

            // Redirigir stdin, stdout, stderr al PTY esclavo
            // Ya tenemos el slave_fd directamente de openpty
            dup2(slave.as_raw_fd(), STDIN_FILENO).unwrap();
            dup2(slave.as_raw_fd(), STDOUT_FILENO).unwrap();
            dup2(slave.as_raw_fd(), STDERR_FILENO).unwrap();

            // Cerrar todos los FDs abiertos por el padre
            // Esto es crucial para que el hijo no mantenga referencias a FDs innecesarios
            for i in 3..256 { // Un rango seguro, ajusta si es necesario
                unsafe { close(i); }
            }

            let shell_path = CString::new("/bin/bash").unwrap();
            let args = [shell_path.clone()];
            let envs: [CString; 0] = []; // Puedes añadir variables de entorno si es necesario

            // Restablecer el manejo de señales y ejecutar el shell
            unsafe {
                execv(&shell_path, &args).unwrap();
            }
        }
        Err(e) => {
            error!("Error bifurcando proceso: {}", e);
            let _ = ws_sender.send(Message::Text(format!("Error: {}", e))).await;
            return;
        }
    }
}

#[derive(Deserialize)]
pub struct WebhookEventRequest {
    pub source: String,
    pub event_type: String,
    pub payload: serde_json::Value,
}

#[derive(Serialize)]
pub struct WebhookResponse {
    pub event_id: String,
    pub cerebro_reaction: String,
}

/// POST /nexus/v1/webhook/event
async fn handle_webhook_event(
    State(state): State<AppState>,
    JsonExtractor(req): JsonExtractor<WebhookEventRequest>,
) -> Json<WebhookResponse> {
    let event_id = uuid::Uuid::new_v4().to_string();
    info!("📥 Webhook recibido: [{}] de {}", req.event_type, req.source);

    let prompt = format!(
        "SISTEMA DE EVENTOS SOBERANO:\nOrigen: {}\nTipo: {}\nDatos: {:?}\n\nAnaliza este evento y determina si requiere una acción inmediata o registro en memoria.",
        req.source, req.event_type, req.payload
    );

    let reaction = state.cerebro.responder(&prompt).await;

    Json(WebhookResponse {
        event_id,
        cerebro_reaction: reaction,
    })
}

#[derive(Deserialize)]
pub struct DebateRequest {
    pub asunto: String,
}

#[derive(Serialize)]
pub struct DebateResponse {
    pub veredicto: nexus_ultimate_core::cerebro::corte_soberana::Verdict,
}

/// POST /nexus/v1/debatir
async fn handle_debatir(
    State(_state): State<AppState>,
    JsonExtractor(req): JsonExtractor<DebateRequest>,
) -> Json<DebateResponse> {
    let cerebro = CerebroHandle::get();
    let veredicto = cerebro.corte.debatir(&req.asunto).await;

    Json(DebateResponse {
        veredicto,
    })
}

// ==========================================
// Rutas
// ==========================================

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/health", get(handle_health))
        .route("/status", get(handle_status))
        .route("/pensar", post(handle_pensar))
        .route("/eval", post(handle_eval))
        .route("/terminal/ws", get(handle_terminal_ws))
        .route("/memoria/buscar", post(handle_memoria_buscar))
        .route("/webhook/event", post(handle_webhook_event))
        .route("/debatir", post(handle_debatir))
}
