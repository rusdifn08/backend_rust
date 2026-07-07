use crate::models::focus::{CreateFocusSessionReq, FocusSession};
use axum::{extract::State, http::StatusCode, Json};
use sqlx::PgPool;

pub async fn get_focus_sessions(
    State(pool): State<PgPool>,
) -> Result<Json<Vec<FocusSession>>, (StatusCode, String)> {
    let sessions = sqlx::query_as::<_, FocusSession>(
        "SELECT * FROM focus_sessions ORDER BY completed_at DESC",
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(sessions))
}

pub async fn create_focus_session(
    State(pool): State<PgPool>,
    Json(req): Json<CreateFocusSessionReq>,
) -> Result<(StatusCode, Json<FocusSession>), (StatusCode, String)> {
    let session = sqlx::query_as::<_, FocusSession>(
        r#"
        INSERT INTO focus_sessions (duration_minutes, task_name) 
        VALUES ($1, $2) 
        RETURNING *
        "#,
    )
    .bind(req.duration_minutes)
    .bind(req.task_name)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok((StatusCode::CREATED, Json(session)))
}
