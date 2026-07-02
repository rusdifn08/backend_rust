use axum::{extract::{State, Path}, http::StatusCode, Json};
use sqlx::PgPool;
use uuid::Uuid;
use crate::models::transaction::{Transaction, CreateTransactionReq};

pub async fn get_transactions(State(pool): State<PgPool>) -> Result<Json<Vec<Transaction>>, (StatusCode, String)> {
    let txs = sqlx::query_as::<_, Transaction>("SELECT * FROM transactions ORDER BY created_at DESC")
        .fetch_all(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(txs))
}

pub async fn create_transaction(
    State(pool): State<PgPool>,
    Json(req): Json<CreateTransactionReq>,
) -> Result<(StatusCode, Json<Transaction>), (StatusCode, String)> {
    let tx = sqlx::query_as::<_, Transaction>(
        r#"
        INSERT INTO transactions (title, date, amount, icon, color, is_expense) 
        VALUES ($1, $2, $3, $4, $5, $6) 
        RETURNING *
        "#
    )
    .bind(req.title)
    .bind(req.date)
    .bind(req.amount)
    .bind(req.icon)
    .bind(req.color)
    .bind(req.is_expense)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok((StatusCode::CREATED, Json(tx)))
}

pub async fn delete_transaction(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    sqlx::query("DELETE FROM transactions WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}
