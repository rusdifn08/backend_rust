use axum::{
    extract::{State, Path},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;
use uuid::Uuid;
use crate::api::chat::AppState;
use crate::repositories::weekly_repo::WeeklyRepo;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct CreateWeeklyRequest {
    pub user_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub day_of_week: i32,
}

pub async fn get_weekly_tasks(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> impl IntoResponse {
    match WeeklyRepo::get_tasks(&state.pool, user_id).await {
        Ok(tasks) => (StatusCode::OK, Json(json!({ "tasks": tasks }))),
        Err(e) => {
            eprintln!("Error fetching weekly tasks: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Internal server error" })))
        }
    }
}

pub async fn create_weekly_task(
    State(state): State<AppState>,
    Json(payload): Json<CreateWeeklyRequest>,
) -> impl IntoResponse {
    match WeeklyRepo::create_task(&state.pool, payload.user_id, &payload.title, payload.description.as_deref(), payload.day_of_week).await {
        Ok(task) => (StatusCode::CREATED, Json(task)).into_response(),
        Err(e) => {
            eprintln!("Error creating weekly task: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Internal server error" }))).into_response()
        }
    }
}

pub async fn toggle_weekly_task(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match WeeklyRepo::toggle_task(&state.pool, id).await {
        Ok(task) => {
            if task.is_completed {
                // Gamification Reward
                let _ = crate::services::gamification_service::GamificationService::add_reward(
                    &state.pool, task.user_id, 20, 10 // 20 EXP, 10 Coins
                ).await;
                
                // Social Feed
                let desc = format!("Completed a weekly task: {}", task.title);
                let _ = crate::repositories::social_repo::SocialRepo::create_activity(
                    &state.pool, task.user_id, "TASK_COMPLETED", &desc
                ).await;
            }
            (StatusCode::OK, Json(task)).into_response()
        },
        Err(e) => {
            eprintln!("Error toggling weekly task: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Internal server error" }))).into_response()
        }
    }
}

pub async fn delete_weekly_task(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match WeeklyRepo::delete_task(&state.pool, id).await {
        Ok(_) => (StatusCode::OK, Json(json!({ "success": true }))).into_response(),
        Err(e) => {
            eprintln!("Error deleting weekly task: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Internal server error" }))).into_response()
        }
    }
}
