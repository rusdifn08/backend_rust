use crate::models::gamification::{UserAchievement, UserStats};
use axum::http::StatusCode;
use sqlx::PgPool;
use uuid::Uuid;

pub struct GamificationRepo;

impl GamificationRepo {
    // Get or Create user stats
    pub async fn get_stats(
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<UserStats, (StatusCode, String)> {
        // Try to select
        let stats = sqlx::query_as::<_, UserStats>("SELECT * FROM user_stats WHERE user_id = $1")
            .bind(user_id)
            .fetch_optional(pool)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        if let Some(s) = stats {
            // Update last_active
            sqlx::query("UPDATE user_stats SET last_active = CURRENT_TIMESTAMP WHERE user_id = $1")
                .bind(user_id)
                .execute(pool)
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

            return Ok(s);
        }

        // Create if not exists
        let new_stats = sqlx::query_as::<_, UserStats>(
            r#"
            INSERT INTO user_stats (user_id, exp, coins, tier, freeze_tickets)
            VALUES ($1, 0, 0, 1, 0)
            RETURNING *
            "#,
        )
        .bind(user_id)
        .fetch_one(pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        Ok(new_stats)
    }

    pub async fn get_achievements(
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<Vec<UserAchievement>, (StatusCode, String)> {
        let achievements = sqlx::query_as::<_, UserAchievement>(
            "SELECT * FROM user_achievements WHERE user_id = $1 ORDER BY unlocked_at DESC",
        )
        .bind(user_id)
        .fetch_all(pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        Ok(achievements)
    }

    pub async fn update_stats(
        pool: &PgPool,
        user_id: Uuid,
        exp: i32,
        coins: i32,
        tier: i32,
        freeze_tickets: i32,
    ) -> Result<UserStats, (StatusCode, String)> {
        let stats = sqlx::query_as::<_, UserStats>(
            r#"
            UPDATE user_stats
            SET exp = $1, coins = $2, tier = $3, freeze_tickets = $4, last_active = CURRENT_TIMESTAMP
            WHERE user_id = $5
            RETURNING *
            "#
        )
        .bind(exp)
        .bind(coins)
        .bind(tier)
        .bind(freeze_tickets)
        .bind(user_id)
        .fetch_one(pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        Ok(stats)
    }

    #[allow(dead_code)]
    pub async fn unlock_achievement(
        pool: &PgPool,
        user_id: Uuid,
        badge_id: &str,
    ) -> Result<bool, (StatusCode, String)> {
        // Attempt insert, on conflict do nothing
        let result = sqlx::query(
            r#"
            INSERT INTO user_achievements (user_id, badge_id)
            VALUES ($1, $2)
            ON CONFLICT (user_id, badge_id) DO NOTHING
            "#,
        )
        .bind(user_id)
        .bind(badge_id)
        .execute(pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        Ok(result.rows_affected() > 0)
    }
}
