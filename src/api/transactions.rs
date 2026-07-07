use crate::models::transaction::{CreateTransactionReq, Transaction};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn get_transactions(
    State(pool): State<PgPool>,
    Path(user_id): Path<String>,
) -> Result<Json<Vec<Transaction>>, (StatusCode, String)> {
    let parsed_uuid = Uuid::parse_str(&user_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid UUID format".to_string()))?;
    let txs = sqlx::query_as::<_, Transaction>(
        "SELECT * FROM transactions WHERE user_id = $1 ORDER BY created_at DESC",
    )
    .bind(parsed_uuid)
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(txs))
}

pub async fn create_transaction(
    State(pool): State<PgPool>,
    Json(req): Json<CreateTransactionReq>,
) -> Result<(StatusCode, Json<Transaction>), (StatusCode, String)> {
    let user_uuid = req
        .user_id
        .map(|id| Uuid::parse_str(&id).unwrap_or(Uuid::default()));

    let tx = sqlx::query_as::<_, Transaction>(
        r#"
        INSERT INTO transactions (user_id, title, date, amount, icon, color, is_expense) 
        VALUES ($1, $2, $3, $4, $5, $6, $7) 
        RETURNING *
        "#,
    )
    .bind(user_uuid)
    .bind(req.title)
    .bind(req.date)
    .bind(req.amount)
    .bind(req.icon)
    .bind(req.color)
    .bind(req.is_expense)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if let Some(uid) = user_uuid {
        let _ = crate::services::gamification_service::GamificationService::add_reward(
            &pool, uid, 5, 2, // 5 EXP, 2 Coins
        )
        .await;
    }

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
