use axum::{extract::{State, Path}, http::StatusCode, Json};
use sqlx::PgPool;
use uuid::Uuid;
use crate::models::todo::{Todo, CreateTodoReq};

pub async fn get_todos(State(pool): State<PgPool>) -> Result<Json<Vec<Todo>>, (StatusCode, String)> {
    let todos = sqlx::query_as::<_, Todo>("SELECT * FROM todos ORDER BY created_at DESC")
        .fetch_all(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(todos))
}

pub async fn create_todo(
    State(pool): State<PgPool>,
    Json(req): Json<CreateTodoReq>,
) -> Result<(StatusCode, Json<Todo>), (StatusCode, String)> {
    let todo = sqlx::query_as::<_, Todo>(
        r#"
        INSERT INTO todos (title, subtitle, category, color) 
        VALUES ($1, $2, $3, $4) 
        RETURNING *
        "#
    )
    .bind(req.title)
    .bind(req.subtitle)
    .bind(req.category)
    .bind(req.color)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok((StatusCode::CREATED, Json(todo)))
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
        "#
    )
    .bind(id)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

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
