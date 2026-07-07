use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use sqlx::PgPool;
use uuid::Uuid;
use crate::models::quest::{Quest, UserQuest, UserQuestResponse};

pub async fn get_daily_quests(
    State(pool): State<PgPool>,
    Path(user_id): Path<String>,
) -> Result<Json<Vec<UserQuestResponse>>, (StatusCode, String)> {
    // Generate or fetch daily quests for user.
    // For simplicity, we just fetch from user_quests joined with quests
    let quests = sqlx::query_as::<_, UserQuestResponse>(
        r#"
        SELECT 
            uq.id, uq.user_id, uq.quest_id, uq.progress, uq.is_completed, uq.is_claimed,
            q.title, q.description, q.quest_type, q.target_count, q.reward_exp, q.reward_coins
        FROM user_quests uq
        JOIN quests q ON uq.quest_id = q.id
        WHERE uq.user_id = $1 AND q.quest_type = 'daily'
        "#
    )
    .bind(user_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(quests))
}

pub async fn claim_quest_reward(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<Json<UserQuest>, (StatusCode, String)> {
    // Start transaction to claim and reward
    let mut tx = pool.begin().await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let uq = sqlx::query_as::<_, UserQuest>(
        r#"
        UPDATE user_quests 
        SET is_claimed = TRUE 
        WHERE id = $1 AND is_completed = TRUE AND is_claimed = FALSE
        RETURNING *
        "#
    )
    .bind(id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if let Some(uq) = uq {
        let q = sqlx::query_as::<_, Quest>(
            r#"SELECT * FROM quests WHERE id = $1"#
        )
        .bind(uq.quest_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        let _ = crate::services::gamification_service::GamificationService::add_reward(
            &pool,
            uuid::Uuid::parse_str(&uq.user_id).unwrap_or_default(),
            q.reward_exp,
            q.reward_coins,
        ).await;

        tx.commit().await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        Ok(Json(uq))
    } else {
        Err((StatusCode::BAD_REQUEST, "Quest not completed, already claimed, or not found".to_string()))
    }
}
