use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct SocialActivity {
    pub id: Uuid,
    pub user_id: Uuid,
    pub action_type: String,
    pub description: String,
    pub created_at: Option<DateTime<Utc>>,
    // Joined fields from users table for display
    pub username: Option<String>,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct LeaderboardEntry {
    pub user_id: Uuid,
    pub username: String,
    pub avatar_url: Option<String>,
    pub tier: i32,
    pub exp: i32,
}
