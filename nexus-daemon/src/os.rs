use axum::{
    response::Json,
    routing::post,
    Router,
};
use serde::{Deserialize, Serialize};
use tracing::info;
use nexus_agent::OSControlManager;

#[derive(Deserialize)]
pub struct OSActionRequest {
    pub action: String, // "click", "type", "key", "adb"
    pub x: Option<u32>,
    pub y: Option<u32>,
    pub text: Option<String>,
    pub key: Option<String>,
    pub params: Option<Vec<String>>,
}

#[derive(Serialize)]
pub struct OSActionResponse {
    pub status: String,
    pub message: String,
}

pub fn router() -> Router {
    Router::new().route("/action", post(ejecutar_accion_os_handler))
}

async fn ejecutar_accion_os_handler(
    Json(payload): Json<OSActionRequest>,
) -> Json<OSActionResponse> {
    info!("⌨️ [NEXUS-DAEMON-OS] Ejecutando acción OS: {}", payload.action);

    let res = match payload.action.as_str() {
        "click" => {
            let x = payload.x.unwrap_or(0);
            let y = payload.y.unwrap_or(0);
            OSControlManager::clic_escritorio(x, y)
        }
        "type" => {
            let txt = payload.text.as_deref().unwrap_or("");
            OSControlManager::escribir_escritorio(txt)
        }
        "key" => {
            let k = payload.key.as_deref().unwrap_or("Return");
            OSControlManager::tecla_escritorio(k)
        }
        "adb" => {
            let act = payload.text.as_deref().unwrap_or("dispositivos");
            let empty_vec = vec![];
            let params_slice: Vec<&str> = payload
                .params
                .as_ref()
                .unwrap_or(&empty_vec)
                .iter()
                .map(|s| s.as_str())
                .collect();
            OSControlManager::ejecutar_adb(act, &params_slice)
        }
        _ => Err(anyhow::anyhow!("Acción OS no válida: {}", payload.action)),
    };

    match res {
        Ok(msg) => Json(OSActionResponse {
            status: "ok".to_string(),
            message: msg,
        }),
        Err(err) => Json(OSActionResponse {
            status: "error".to_string(),
            message: format!("Fallo en OSControlManager: {}", err),
        }),
    }
}
