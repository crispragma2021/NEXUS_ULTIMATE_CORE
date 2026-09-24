use axum::{
    extract::Query,
    response::Json,
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use tracing::info;
use nexus_agent::VisionManager;

#[derive(Deserialize)]
pub struct VisionQuery {
    pub target: Option<String>,
}

#[derive(Serialize)]
pub struct VisionResponse {
    pub status: String,
    pub path: Option<String>,
    pub base64: Option<String>,
    pub message: Option<String>,
}

pub fn router() -> Router {
    Router::new().route("/capture", get(capturar_pantalla_handler))
}

async fn capturar_pantalla_handler(Query(query): Query<VisionQuery>) -> Json<VisionResponse> {
    let target = query.target.as_deref().unwrap_or("escritorio");
    info!("📸 [NEXUS-DAEMON-VISION] Solicitando captura de visión para: {}", target);

    match VisionManager::capturar_pantalla(target) {
        Ok(path) => {
            let b64 = VisionManager::preparar_base64(&path).ok();
            Json(VisionResponse {
                status: "ok".to_string(),
                path: Some(path.to_string_lossy().to_string()),
                base64: b64,
                message: Some(format!("Captura exitosa de {}", target)),
            })
        }
        Err(err) => Json(VisionResponse {
            status: "error".to_string(),
            path: None,
            base64: None,
            message: Some(format!("Fallo en visión: {}", err)),
        }),
    }
}
