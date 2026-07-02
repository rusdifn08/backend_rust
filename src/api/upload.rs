use axum::{
    extract::{Multipart, Path, State},
    http::{header, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::Serialize;
use uuid::Uuid;

use crate::api::chat::AppState; // Re-use AppState from chat for DB pool

#[derive(Serialize)]
pub struct UploadResponse {
    pub url: String,
}

pub async fn upload_file(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let mut data_bytes = Vec::new();
    let mut mime_type = String::from("application/octet-stream");

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?
    {
        let name = field.name().unwrap_or("").to_string();
        if name == "file" {
            if let Some(ct) = field.content_type() {
                mime_type = ct.to_string();
            }
            let data = field
                .bytes()
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
            data_bytes = data.to_vec();
        }
    }

    if data_bytes.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "No file uploaded".to_string()));
    }

    let id = Uuid::new_v4();

    sqlx::query(
        "INSERT INTO file_uploads (id, mime_type, data) VALUES ($1, $2, $3)"
    )
    .bind(id)
    .bind(&mime_type)
    .bind(&data_bytes)
    .execute(&state.pool)
    .await
    .map_err(|e: sqlx::Error| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let url = format!("http://localhost:5050/api/files/{}", id);
    Ok((StatusCode::CREATED, Json(UploadResponse { url })))
}

use sqlx::Row;

pub async fn get_file(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let record = sqlx::query(
        "SELECT mime_type, data FROM file_uploads WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e: sqlx::Error| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    match record {
        Some(r) => {
            let mime_type: String = r.get("mime_type");
            let data: Vec<u8> = r.get("data");
            let headers = [(header::CONTENT_TYPE, mime_type)];
            Ok((headers, data))
        }
        None => Err((StatusCode::NOT_FOUND, "File not found".to_string())),
    }
}
