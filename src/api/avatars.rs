use axum::http::StatusCode;
use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};

use crate::api::chat::AppState;

#[derive(Serialize, Deserialize, sqlx::FromRow)]
pub struct Avatar {
    pub id: i32,
    pub name: String,
    pub url: String,
}

pub async fn get_avatars(
    State(state): State<AppState>,
) -> Result<Json<Vec<Avatar>>, (StatusCode, String)> {
    let avatars = sqlx::query_as::<_, Avatar>("SELECT id, name, url FROM avatars ORDER BY id")
        .fetch_all(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(avatars))
}
