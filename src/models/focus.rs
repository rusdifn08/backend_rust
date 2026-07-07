use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct FocusSession {
    pub id: Uuid,
    pub duration_minutes: i32,
    pub task_name: String,
    pub completed_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateFocusSessionReq {
    pub duration_minutes: i32,
    pub task_name: String,
}
