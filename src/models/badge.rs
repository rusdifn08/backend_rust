use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Badge {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub icon: String,
    pub criteria_type: String,
    pub criteria_value: i32,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct UserBadge {
    pub id: Uuid,
    pub user_id: String,
    pub badge_id: Uuid,
    pub earned_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct UserBadgeResponse {
    pub id: Uuid,
    pub user_id: String,
    pub badge_id: Uuid,
    pub earned_at: Option<DateTime<Utc>>,
    pub name: String,
    pub description: Option<String>,
    pub icon: String,
}
