use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct UserStats {
    pub user_id: Uuid,
    pub exp: i32,
    pub coins: i32,
    pub tier: i32,
    pub freeze_tickets: i32,
    pub last_active: Option<DateTime<Utc>>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct UserAchievement {
    pub id: Uuid,
    pub user_id: Uuid,
    pub badge_id: String,
    pub unlocked_at: Option<DateTime<Utc>>,
}

// Request and Response DTOs
#[derive(Debug, Serialize)]
pub struct BuyFreezeTicketResponse {
    pub success: bool,
    pub message: String,
    pub remaining_coins: i32,
    pub total_tickets: i32,
}

#[derive(Debug, Serialize)]
pub struct GamificationProfileResponse {
    pub stats: UserStats,
    pub achievements: Vec<UserAchievement>,
}
