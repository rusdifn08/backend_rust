use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct AppVersion {
    pub id: Uuid,
    pub version_code: i32,
    pub version_name: String,
    pub download_url: String,
    pub release_notes: Option<String>,
    pub is_mandatory: bool,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct CreateAppVersion {
    pub version_code: i32,
    pub version_name: String,
    pub download_url: String,
    pub release_notes: Option<String>,
    pub is_mandatory: Option<bool>,
}
