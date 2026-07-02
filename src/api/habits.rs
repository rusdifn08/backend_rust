use axum::{extract::{State, Path}, http::StatusCode, Json};
use sqlx::PgPool;
use uuid::Uuid;
use crate::models::habit::{Habit, CreateHabitReq};

pub async fn get_habits(State(pool): State<PgPool>) -> Result<Json<Vec<Habit>>, (StatusCode, String)> {
    let habits = sqlx::query_as::<_, Habit>("SELECT * FROM habits ORDER BY created_at DESC")
        .fetch_all(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(habits))
}

pub async fn create_habit(
    State(pool): State<PgPool>,
    Json(req): Json<CreateHabitReq>,
) -> Result<(StatusCode, Json<Habit>), (StatusCode, String)> {
    let habit = sqlx::query_as::<_, Habit>(
        r#"
        INSERT INTO habits (title, time, icon, color, streak, category) 
        VALUES ($1, $2, $3, $4, $5, $6) 
        RETURNING *
        "#
    )
    .bind(req.title)
    .bind(req.time)
    .bind(req.icon)
    .bind(req.color)
    .bind(req.streak)
    .bind(req.category)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok((StatusCode::CREATED, Json(habit)))
}

pub async fn toggle_habit(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<Json<Habit>, (StatusCode, String)> {
    let habit = sqlx::query_as::<_, Habit>(
        r#"
        UPDATE habits 
        SET is_completed = NOT is_completed, updated_at = NOW() 
        WHERE id = $1 
        RETURNING *
        "#
    )
    .bind(id)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(habit))
}

pub async fn delete_habit(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    sqlx::query("DELETE FROM habits WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}
