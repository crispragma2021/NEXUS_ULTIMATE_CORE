use axum::{
    extract::Query,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::fs;

pub fn router() -> Router {
    Router::new()
        .route("/read", get(read_file))
        .route("/write", post(write_file))
        .route("/list", get(list_dir))
}

#[derive(Deserialize)]
pub struct ReadQuery {
    path: String,
}

#[derive(Serialize)]
pub struct ReadResponse {
    content: String,
}

async fn read_file(Query(q): Query<ReadQuery>) -> Result<Json<ReadResponse>, (StatusCode, String)> {
    let content = fs::read_to_string(&q.path)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(ReadResponse { content }))
}

#[derive(Deserialize)]
pub struct WriteRequest {
    path: String,
    content: String,
}

async fn write_file(Json(payload): Json<WriteRequest>) -> Result<StatusCode, (StatusCode, String)> {
    fs::write(&payload.path, &payload.content)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(StatusCode::OK)
}

#[derive(Deserialize)]
pub struct ListQuery {
    path: String,
}

#[derive(Serialize)]
pub struct FileEntry {
    name: String,
    is_dir: bool,
}

#[derive(Serialize)]
pub struct ListResponse {
    files: Vec<FileEntry>,
}

async fn list_dir(Query(q): Query<ListQuery>) -> Result<Json<ListResponse>, (StatusCode, String)> {
    let mut entries = fs::read_dir(&q.path)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut files = Vec::new();
    while let Some(entry) = entries
        .next_entry()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    {
        let metadata = entry
            .metadata()
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        files.push(FileEntry {
            name: entry.file_name().to_string_lossy().into_owned(),
            is_dir: metadata.is_dir(),
        });
    }
    
    // Sort directories first, then alphabetical
    files.sort_by(|a, b| {
        b.is_dir.cmp(&a.is_dir).then_with(|| a.name.cmp(&b.name))
    });

    Ok(Json(ListResponse { files }))
}
