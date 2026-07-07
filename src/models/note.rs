use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Note {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub title: String,
    pub content: String,
    pub date: String,
    pub tag: String,
    pub color: String,
    pub deadline: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateNoteReq {
    pub user_id: String,
    pub title: String,
    pub content: String,
    pub date: String,
    pub tag: String,
    pub color: String,
    pub deadline: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
pub struct UpdateNoteReq {
    pub title: String,
    pub content: String,
    pub date: String,
    pub tag: String,
    pub color: String,
    pub deadline: Option<String>,
}
