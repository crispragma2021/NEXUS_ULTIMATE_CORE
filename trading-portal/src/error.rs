use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] rusqlite::Error),
    
    #[error("Internal server error: {0}")]
    InternalError(#[from] anyhow::Error),
    
    #[error("Bad request: {0}")]
    BadRequest(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::DatabaseError(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
            AppError::InternalError(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
            AppError::BadRequest(err) => (StatusCode::BAD_REQUEST, err),
        };

        let body = Json(json!({
            "status": "error",
            "mensaje": error_message
        }));

        (status, body).into_response()
    }
}
