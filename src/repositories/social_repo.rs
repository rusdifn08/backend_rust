use sqlx::PgPool;
use uuid::Uuid;
use crate::models::social::{SocialActivity, LeaderboardEntry};

pub struct SocialRepo;

impl SocialRepo {
    pub async fn create_activity(
        pool: &PgPool,
        user_id: Uuid,
        action_type: &str,
        description: &str,
    ) -> Result<SocialActivity, sqlx::Error> {
        let row = sqlx::query_as::<_, SocialActivity>(
            r#"
            INSERT INTO social_activities (user_id, action_type, description)
            VALUES ($1, $2, $3)
            RETURNING id, user_id, action_type, description, created_at, 
            NULL as username, NULL as avatar_url
            "#
        )
        .bind(user_id)
        .bind(action_type)
        .bind(description)
        .fetch_one(pool)
        .await?;

        Ok(row)
    }

    pub async fn get_feed(pool: &PgPool, limit: i64) -> Result<Vec<SocialActivity>, sqlx::Error> {
        // Fetch recent activities from all users
        let rows = sqlx::query_as::<_, SocialActivity>(
            r#"
            SELECT a.id, a.user_id, a.action_type, a.description, a.created_at,
                   u.username, u.avatar_url
            FROM social_activities a
            JOIN users u ON a.user_id = u.id
            ORDER BY a.created_at DESC
            LIMIT $1
            "#
        )
        .bind(limit)
        .fetch_all(pool)
        .await?;

        Ok(rows)
    }

    pub async fn get_leaderboard(pool: &PgPool, limit: i64) -> Result<Vec<LeaderboardEntry>, sqlx::Error> {
        let rows = sqlx::query_as::<_, LeaderboardEntry>(
            r#"
            SELECT u.id as user_id, u.username, u.avatar_url, s.tier, s.exp
            FROM users u
            JOIN user_stats s ON u.id = s.user_id
            ORDER BY s.exp DESC
            LIMIT $1
            "#
        )
        .bind(limit)
        .fetch_all(pool)
        .await?;

        Ok(rows)
    }
}
