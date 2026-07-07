use crate::models::todo::{CreateTodoReq, Todo, UpdateTodoReq};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn get_todos(
    Path(user_id): Path<Uuid>,
    State(pool): State<PgPool>,
) -> Result<Json<Vec<Todo>>, (StatusCode, String)> {
    let todos = sqlx::query_as::<_, Todo>("SELECT * FROM todos WHERE user_id = $1 AND (is_completed = false OR DATE(updated_at AT TIME ZONE 'UTC') >= CURRENT_DATE) ORDER BY created_at DESC")
        .bind(user_id)
        .fetch_all(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(todos))
}

pub async fn create_todo(
    State(pool): State<PgPool>,
    Json(req): Json<CreateTodoReq>,
) -> Result<(StatusCode, Json<Todo>), (StatusCode, String)> {
    let user_uuid = Uuid::parse_str(&req.user_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid UUID".into()))?;
    let todo = sqlx::query_as::<_, Todo>(
        r#"
        INSERT INTO todos (user_id, title, subtitle, category, color, icon) 
        VALUES ($1, $2, $3, $4, $5, $6) 
        RETURNING *
        "#,
    )
    .bind(user_uuid)
    .bind(req.title)
    .bind(req.subtitle)
    .bind(req.category)
    .bind(req.color)
    .bind(req.icon)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok((StatusCode::CREATED, Json(todo)))
}

pub async fn update_todo(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateTodoReq>,
) -> Result<Json<Todo>, (StatusCode, String)> {
    let todo = sqlx::query_as::<_, Todo>(
        r#"
        UPDATE todos 
        SET title = $1, subtitle = $2, category = $3, color = $4, icon = $5, updated_at = NOW() 
        WHERE id = $6 
        RETURNING *
        "#,
    )
    .bind(req.title)
    .bind(req.subtitle)
    .bind(req.category)
    .bind(req.color)
    .bind(req.icon)
    .bind(id)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(todo))
}

pub async fn toggle_todo(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<Json<Todo>, (StatusCode, String)> {
    let todo = sqlx::query_as::<_, Todo>(
        r#"
        UPDATE todos 
        SET is_completed = NOT is_completed, updated_at = NOW() 
        WHERE id = $1 
        RETURNING *
        "#,
    )
    .bind(id)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if todo.is_completed {
        if let Some(uid) = todo.user_id {
            // Gamification Reward
            let _ = crate::services::gamification_service::GamificationService::add_reward(
                &pool, uid, 10, 5, // 10 EXP, 5 Coins
            )
            .await;

            // Social Feed
            let desc = format!("Completed a task: {}", todo.title);
            let _ = crate::repositories::social_repo::SocialRepo::create_activity(
                &pool,
                uid,
                "TODO_COMPLETED",
                &desc,
            )
            .await;
        }
    }

    Ok(Json(todo))
}

pub async fn delete_todo(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    sqlx::query("DELETE FROM todos WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}
