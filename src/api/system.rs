use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use sqlx::PgPool;

use crate::{
    api::chat::AppState,
    models::system::{AppVersion, CreateAppVersion},
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/version/latest", get(get_latest_version))
        .route("/version", post(publish_new_version))
}

async fn get_latest_version(
    State(state): State<AppState>,
) -> Result<Json<AppVersion>, (StatusCode, String)> {
    let result = sqlx::query_as::<_, AppVersion>(
        "SELECT * FROM app_versions ORDER BY version_code DESC LIMIT 1",
    )
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    match result {
        Some(version) => Ok(Json(version)),
        None => Err((StatusCode::NOT_FOUND, "No app versions found".to_string())),
    }
}

async fn publish_new_version(
    State(state): State<AppState>,
    Json(payload): Json<CreateAppVersion>,
) -> Result<Json<AppVersion>, (StatusCode, String)> {
    // Delete all previous versions to keep only the latest one
    sqlx::query("DELETE FROM app_versions")
        .execute(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Insert the new version
    let new_version = sqlx::query_as::<_, AppVersion>(
        r#"
        INSERT INTO app_versions (version_code, version_name, download_url, release_notes, is_mandatory)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING *
        "#
    )
    .bind(payload.version_code)
    .bind(payload.version_name)
    .bind(payload.download_url)
    .bind(payload.release_notes)
    .bind(payload.is_mandatory.unwrap_or(false))
    .fetch_one(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(new_version))
}
