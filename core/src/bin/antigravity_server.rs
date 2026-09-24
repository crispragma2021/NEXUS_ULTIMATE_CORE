// ============================================================================
// 🛰️ ANTIGRAVITY SERVER — API REST del Orquestador (Reglas 1-4)
// ============================================================================
// Expone los 3 componentes de negocio al frontend React (ui/):
//
//   GET  /api/projects            → proyectos + estado LED + puerto inmutable
//   POST /api/projects            → registrar proyecto (asigna puerto 8000-8999)
//   GET  /api/projects/:id/status → healthcheck en vivo
//   POST /api/messages/resolve    → ScopeMapper: contexto aislado por proyecto
//
// Stack: axum + rusqlite + los 3 módulos de core/orquestador/.
// ============================================================================

use anyhow::Result;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use nexus_ultimate_core::orquestador::health_monitor::{HealthMonitor, ServiceStatus};
use nexus_ultimate_core::orquestador::port_registry::PortRegistry;
use nexus_ultimate_core::orquestador::scope_mapper::{ProjectScope, ScopeMapper};
use nexus_ultimate_core::scraping::pipeline::cerebro::Cerebro;
use nexus_ultimate_core::scraping::pipeline::embedding::EmbeddingEngine;
use nexus_ultimate_core::scraping::pipeline::vector_store::VectorStore;
use nexus_ultimate_core::sentidos::omnipresent_vision::OmnipresentVision;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::process::Command;
use tower_http::cors::{Any, CorsLayer};
use axum::http::Method;

/// Estado compartido del servidor.
struct AppState {
    scopes: Arc<ScopeMapper>,
    ports: Arc<PortRegistry>,
    health: Arc<HealthMonitor>,
    /// Cerebro RAG (opcional): conecta el ScopeMapper con la recuperación
    /// semántica por proyecto. `None` si Ollama/embeddings no están disponibles.
    cerebro: Option<Arc<Cerebro>>,
}

// ─── DTOs ────────────────────────────────────────────────────────────────

#[derive(Serialize)]
struct ProjectDto {
    id: String,
    name: String,
    status: String,
    port: Option<u16>,
}

#[derive(Deserialize)]
struct RegisterProjectReq {
    id: String,
    name: String,
    aliases: Vec<String>,
    files: Vec<String>,
    log_dir: Option<String>,
}

#[derive(Deserialize)]
struct ResolveMsgReq {
    message: String,
}

#[derive(Serialize)]
struct ResolveMsgResp {
    project_id: Option<String>,
    context: Option<String>,
}

#[derive(Serialize)]
struct VisionDto {
    base64: Option<String>,
    ocr: Option<String>,
    active: bool,
}

#[derive(Deserialize)]
struct PipelineExecuteReq {
    pipeline_type: String,
    prompt: String,
}

#[derive(Serialize)]
struct PipelineExecuteResp {
    success: bool,
    output: String,
}

// ─── Handlers ────────────────────────────────────────────────────────────

/// GET /api/projects — lista proyectos con estado LED + puerto.
async fn list_projects(State(st): State<Arc<AppState>>) -> Json<Vec<ProjectDto>> {
    let ids = st.scopes.list_projects().unwrap_or_default();
    let assignments = st.ports.list_assignments().unwrap_or_default();
    let port_map: HashMap<String, u16> = assignments.into_iter().collect();

    let mut out = Vec::new();
    for id in ids {
        let Ok(scope) = st.scopes.build_scope(&id) else {
            continue;
        };
        let port = port_map.get(&id).copied();
        let status = match port {
            Some(p) => match st.health.check(&id, p).status {
                ServiceStatus::Up => "up",
                ServiceStatus::Down => "down",
            },
            None => "down",
        };
        out.push(ProjectDto {
            id,
            name: scope.name,
            status: status.to_string(),
            port,
        });
    }
    Json(out)
}

/// POST /api/projects — registra un proyecto y le asigna puerto inmutable.
async fn register_project(
    State(st): State<Arc<AppState>>,
    Json(req): Json<RegisterProjectReq>,
) -> Result<(StatusCode, Json<ProjectDto>), StatusCode> {
    let scope = ProjectScope {
        id: req.id.clone(),
        name: req.name.clone(),
        aliases: req.aliases,
        files: req.files,
        env_vars: HashMap::new(),
        log_dir: req.log_dir.unwrap_or_else(|| "logs".into()),
    };
    st.scopes
        .register_project(&scope)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let port = st
        .ports
        .assign_port(&req.id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok((
        StatusCode::CREATED,
        Json(ProjectDto {
            id: req.id,
            name: req.name,
            status: "down".into(),
            port: Some(port),
        }),
    ))
}

/// GET /api/projects/:id/status — healthcheck en vivo para el LED.
async fn project_status(
    State(st): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<ProjectDto>, StatusCode> {
    let scope = st
        .scopes
        .build_scope(&id)
        .map_err(|_| StatusCode::NOT_FOUND)?;
    let port = st.ports.get_port(&id).map_err(|_| StatusCode::NOT_FOUND)?;
    let status = st.health.check(&id, port).status;
    Ok(Json(ProjectDto {
        id,
        name: scope.name,
        status: if status == ServiceStatus::Up {
            "up"
        } else {
            "down"
        }
        .into(),
        port: Some(port),
    }))
}

/// POST /api/messages/resolve — ScopeMapper (Regla 1) + Cerebro RAG.
///
/// Flujo completo (conexión de las dos capas):
/// 1. ScopeMapper detecta qué proyecto menciona el usuario.
/// 2. Aísla el contexto estático del proyecto (archivos/logs/variables).
/// 3. Cerebro RAG recupera los fragmentos SEMÁNTICOS más relevantes del
///    conocimiento acumulado de ESE proyecto (nunca de otros).
/// 4. Combina ambos → prompt compacto para el LLM local.
async fn resolve_message(
    State(st): State<Arc<AppState>>,
    Json(req): Json<ResolveMsgReq>,
) -> Json<ResolveMsgResp> {
    let Some(project_id) = st.scopes.detect_project(&req.message).ok().flatten() else {
        // Sin proyecto → sin contexto (ahorro de tokens).
        return Json(ResolveMsgResp {
            project_id: None,
            context: None,
        });
    };

    // 1. Contexto estático del proyecto (ScopeMapper).
    let static_ctx = st
        .scopes
        .resolve_context(&req.message)
        .ok()
        .flatten()
        .unwrap_or_default();

    // 2. Contexto semántico del proyecto (Cerebro RAG).
    let mut rag_ctx = String::new();
    if let Some(cerebro) = &st.cerebro {
        match cerebro
            .build_project_context(&req.message, &project_id, 3)
            .await
        {
            Ok((ctx, hits)) if !hits.is_empty() => {
                rag_ctx = format!("### CONOCIMIENTO PREVIO DEL PROYECTO (RAG)\n{ctx}");
            }
            _ => {} // sin conocimiento indexado aún
        }
    }

    // 3. Combinar: contexto aislado + fragmentos semánticos.
    let mut context = String::new();
    context.push_str(&static_ctx);
    if !rag_ctx.is_empty() {
        context.push('\n');
        context.push_str(&rag_ctx);
    }

    Json(ResolveMsgResp {
        project_id: Some(project_id),
        context: Some(context),
    })
}

/// GET /api/vision/latest — devuelve la última captura y texto de OmnipresentVision.
async fn get_latest_vision() -> Json<VisionDto> {
    let eye = OmnipresentVision::instance();
    let lock = eye.read().await;
    Json(VisionDto {
        base64: lock.ultimo_frame_b64.clone(),
        ocr: lock.ultimo_texto_ocr.clone(),
        active: lock.activo,
    })
}

/// POST /api/pipelines/execute — ejecuta un script Node de pipeline.
async fn execute_pipeline(
    Json(req): Json<PipelineExecuteReq>,
) -> Json<PipelineExecuteResp> {
    let script_name = match req.pipeline_type.as_str() {
        "extract_ui" => "pipeline/agent_ui/extract_ui.js",
        "generate_code" => "pipeline/agent_backend/generate_code.js",
        "infer_architecture" => "pipeline/agent_architect/infer_architecture.js",
        _ => return Json(PipelineExecuteResp { success: false, output: "Invalid pipeline".into() }),
    };

    let core_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into()));
    let workspace_dir = core_dir.parent().unwrap_or(&core_dir);
    let script_path = workspace_dir.join(script_name);

    match Command::new("node")
        .arg(&script_path)
        .arg(&req.prompt)
        .current_dir(workspace_dir)
        .output() 
    {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            let full_out = format!("{}\n{}", stdout, stderr);
            Json(PipelineExecuteResp { success: output.status.success(), output: full_out })
        }
        Err(e) => {
            Json(PipelineExecuteResp { success: false, output: e.to_string() })
        }
    }
}

// ─── main ────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> Result<()> {
    let port: u16 = std::env::var("ANTIGRAVITY_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(43211);

    let db_dir = PathBuf::from(
        std::env::var("ANTIGRAVITY_DB").unwrap_or_else(|_| "data/orquestador".into()),
    );
    std::fs::create_dir_all(&db_dir)?;

    // Cerebro RAG (opcional): se activa si Ollama (nomic-embed-text) está
    // disponible. Conecta el ScopeMapper con la recuperación semántica por
    // proyecto. Sin Ollama, el servidor sigue funcionando solo con ScopeMapper.
    let cerebro: Option<Arc<Cerebro>> = match EmbeddingEngine::default() {
        Ok(embedding) => {
            let store = Arc::new(VectorStore::open(&db_dir.join("brain.db"))?);
            tracing::info!(
                "🧠 [ANTIGRAVITY] Cerebro RAG activo (recuperación semántica por proyecto)"
            );
            Some(Arc::new(Cerebro::new(embedding, store)))
        }
        Err(_) => {
            tracing::warn!("⚠️ [ANTIGRAVITY] sin Cerebro RAG (Ollama no disponible)");
            None
        }
    };

    let state = Arc::new(AppState {
        scopes: Arc::new(ScopeMapper::open(&db_dir.join("scopes.db"))?),
        ports: Arc::new(PortRegistry::open(&db_dir.join("ports.db"))?),
        health: Arc::new(HealthMonitor::default()),
        cerebro,
    });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/api/projects", get(list_projects).post(register_project))
        .route("/api/projects/:id/status", get(project_status))
        .route("/api/messages/resolve", post(resolve_message))
        .route("/api/vision/latest", get(get_latest_vision))
        .route("/api/pipelines/execute", post(execute_pipeline))
        .route("/health", get(|| async { "ok" }))
        .layer(cors)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{port}")).await?;
    tracing::info!("🛰️ [ANTIGRAVITY] servidor en http://127.0.0.1:{port}");
    axum::serve(listener, app).await?;
    Ok(())
}
