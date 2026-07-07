use crate::api::chat::AppState;
use crate::repositories::social_repo::SocialRepo;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct CreateActivityRequest {
    pub user_id: Uuid,
    pub action_type: String,
    pub description: String,
}

pub async fn get_feed(State(state): State<AppState>) -> impl IntoResponse {
    match SocialRepo::get_feed(&state.pool, 50).await {
        Ok(feed) => (StatusCode::OK, Json(json!({ "feed": feed }))),
        Err(e) => {
            eprintln!("Error fetching feed: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal server error" })),
            )
        }
    }
}

pub async fn get_leaderboard(State(state): State<AppState>) -> impl IntoResponse {
    match SocialRepo::get_leaderboard(&state.pool, 50).await {
        Ok(leaderboard) => (StatusCode::OK, Json(json!({ "leaderboard": leaderboard }))),
        Err(e) => {
            eprintln!("Error fetching leaderboard: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal server error" })),
            )
        }
    }
}

pub async fn create_activity(
    State(state): State<AppState>,
    Json(payload): Json<CreateActivityRequest>,
) -> impl IntoResponse {
    match SocialRepo::create_activity(
        &state.pool,
        payload.user_id,
        &payload.action_type,
        &payload.description,
    )
    .await
    {
        Ok(activity) => (StatusCode::CREATED, Json(activity)).into_response(),
        Err(e) => {
            eprintln!("Error creating activity: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal server error" })),
            )
                .into_response()
        }
    }
}
