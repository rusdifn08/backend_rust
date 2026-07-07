use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Todo {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub title: String,
    pub subtitle: String,
    pub category: String,
    pub color: String,
    pub icon: Option<String>,
    pub is_completed: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateTodoReq {
    pub user_id: String,
    pub title: String,
    pub subtitle: String,
    pub category: String,
    pub color: String,
    pub icon: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTodoReq {
    pub title: String,
    pub subtitle: String,
    pub category: String,
    pub color: String,
    pub icon: Option<String>,
}
