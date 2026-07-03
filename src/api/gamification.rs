use axum::{
    extract::{State, Path},
    routing::{get, post},
    Json, Router,
    http::StatusCode,
};
use sqlx::PgPool;
use uuid::Uuid;
use crate::models::user::User;
use crate::models::gamification::{GamificationProfileResponse, BuyFreezeTicketResponse};
use crate::repositories::gamification_repo::GamificationRepo;
use crate::services::gamification_service::GamificationService;
use crate::api::chat::AppState;
use serde_json::json;
use axum::response::IntoResponse;

pub async fn get_profile(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<GamificationProfileResponse>, (StatusCode, String)> {
    let stats = GamificationRepo::get_stats(&state.pool, user_id).await?;
    let achievements = GamificationRepo::get_achievements(&state.pool, user_id).await?;

    Ok(Json(GamificationProfileResponse {
        stats,
        achievements,
    }))
}

pub async fn get_border(
    State(state): State<AppState>,
    Path(tier): Path<i32>,
) -> impl IntoResponse {
    let row = sqlx::query!("SELECT image_data FROM tier_borders WHERE tier = $1", tier)
        .fetch_one(&state.pool)
        .await;

    match row {
        Ok(record) => {
            let mut headers = axum::http::HeaderMap::new();
            headers.insert(axum::http::header::CONTENT_TYPE, "image/png".parse().unwrap());
            headers.insert(axum::http::header::CACHE_CONTROL, "public, max-age=86400".parse().unwrap());
            (StatusCode::OK, headers, record.image_data).into_response()
        },
        Err(_) => (StatusCode::NOT_FOUND, "Border not found").into_response(),
    }
}

pub async fn buy_freeze_ticket(
    State(pool): State<PgPool>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<BuyFreezeTicketResponse>, (StatusCode, String)> {
    let response = GamificationService::buy_freeze_ticket(&pool, user_id).await?;
    Ok(Json(response))
}
