use crate::models::note::{CreateNoteReq, Note, UpdateNoteReq};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn get_notes(
    Path(user_id): Path<Uuid>,
    State(pool): State<PgPool>,
) -> Result<Json<Vec<Note>>, (StatusCode, String)> {
    let notes = sqlx::query_as::<_, Note>(
        "SELECT * FROM notes WHERE user_id = $1 ORDER BY created_at DESC",
    )
    .bind(user_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(notes))
}

pub async fn create_note(
    State(pool): State<PgPool>,
    Json(req): Json<CreateNoteReq>,
) -> Result<(StatusCode, Json<Note>), (StatusCode, String)> {
    let user_uuid = Uuid::parse_str(&req.user_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid UUID".into()))?;
    let note = sqlx::query_as::<_, Note>(
        r#"
        INSERT INTO notes (user_id, title, content, date, tag, color, deadline) 
        VALUES ($1, $2, $3, $4, $5, $6, $7) 
        RETURNING *
        "#,
    )
    .bind(user_uuid)
    .bind(req.title)
    .bind(req.content)
    .bind(req.date)
    .bind(req.tag)
    .bind(req.color)
    .bind(req.deadline)
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

pub async fn update_note(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateNoteReq>,
) -> Result<Json<Note>, (StatusCode, String)> {
    let note = sqlx::query_as::<_, Note>(
        r#"
        UPDATE notes 
        SET title = , content = , date = , tag = , color = , deadline = , updated_at = NOW() 
        WHERE id =  
        RETURNING *
        "#,
    )
    .bind(req.title)
    .bind(req.content)
    .bind(req.date)
    .bind(req.tag)
    .bind(req.color)
    .bind(req.deadline)
    .bind(id)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(note))
}
