use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Squad {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub level: i32,
    pub exp: i32,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct SquadMember {
    pub id: Uuid,
    pub squad_id: Uuid,
    pub user_id: String,
    pub role: String,
    pub joined_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateSquadReq {
    pub name: String,
    pub description: Option<String>,
    pub user_id: String, // creator
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct SquadMemberResponse {
    pub id: Uuid,
    pub squad_id: Uuid,
    pub user_id: String,
    pub username: String,
    pub avatar_url: Option<String>,
    pub role: String,
    pub joined_at: Option<DateTime<Utc>>,
}
