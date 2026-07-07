use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Habit {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub title: String,
    pub time: String,
    pub icon: String,
    pub color: String,
    pub streak: i32,
    pub category: String,
    pub is_completed: bool,
    pub description: Option<String>,
    pub frequency: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct HabitLog {
    pub id: Uuid,
    pub habit_id: Uuid,
    pub user_id: Option<Uuid>,
    pub note: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateHabitReq {
    pub user_id: String,
    pub title: String,
    pub time: String,
    pub icon: String,
    pub color: String,
    pub streak: i32,
    pub category: String,
    pub description: Option<String>,
    pub frequency: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateHabitLogReq {
    pub habit_id: String,
    pub user_id: Option<String>,
    pub note: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
pub struct UpdateHabitReq {
    pub title: String,
    pub subtitle: String,
    pub category: String,
    pub target_days: i32,
    pub color: String,
    pub icon: Option<String>,
}
