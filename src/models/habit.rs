use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Habit {
    pub id: Uuid,
    pub title: String,
    pub time: String,
    pub icon: String,
    pub color: String,
    pub streak: i32,
    pub category: String,
    pub is_completed: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateHabitReq {
    pub title: String,
    pub time: String,
    pub icon: String,
    pub color: String,
    pub streak: i32,
    pub category: String,
}
