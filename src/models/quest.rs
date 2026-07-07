use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Quest {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub quest_type: String,
    pub target_count: i32,
    pub reward_exp: i32,
    pub reward_coins: i32,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct UserQuest {
    pub id: Uuid,
    pub user_id: String,
    pub quest_id: Uuid,
    pub progress: i32,
    pub is_completed: bool,
    pub is_claimed: bool,
    pub created_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct UserQuestResponse {
    pub id: Uuid,
    pub user_id: String,
    pub quest_id: Uuid,
    pub progress: i32,
    pub is_completed: bool,
    pub is_claimed: bool,
    pub title: String,
    pub description: Option<String>,
    pub quest_type: String,
    pub target_count: i32,
    pub reward_exp: i32,
    pub reward_coins: i32,
}
