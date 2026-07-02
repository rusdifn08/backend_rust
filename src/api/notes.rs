use axum::{extract::{State, Path}, http::StatusCode, Json};
use sqlx::PgPool;
use uuid::Uuid;
use crate::models::note::{Note, CreateNoteReq};

pub async fn get_notes(State(pool): State<PgPool>) -> Result<Json<Vec<Note>>, (StatusCode, String)> {
    let notes = sqlx::query_as::<_, Note>("SELECT * FROM notes ORDER BY created_at DESC")
        .fetch_all(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(notes))
}

pub async fn create_note(
    State(pool): State<PgPool>,
    Json(req): Json<CreateNoteReq>,
) -> Result<(StatusCode, Json<Note>), (StatusCode, String)> {
    let note = sqlx::query_as::<_, Note>(
        r#"
        INSERT INTO notes (title, content, date, tag, color) 
        VALUES ($1, $2, $3, $4, $5) 
        RETURNING *
        "#
    )
    .bind(req.title)
    .bind(req.content)
    .bind(req.date)
    .bind(req.tag)
    .bind(req.color)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok((StatusCode::CREATED, Json(note)))
}

pub async fn delete_note(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    sqlx::query("DELETE FROM notes WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}
