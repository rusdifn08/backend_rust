use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Friend {
    pub id: Uuid,
    pub user_id_1: Uuid,
    pub user_id_2: Uuid,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct ChatMessage {
    pub id: Uuid,
    pub sender_id: Uuid,
    pub receiver_id: Uuid,
    pub content: String,
    pub message_type: String,
    pub created_at: DateTime<Utc>,
    pub is_read: bool,
}

#[derive(Debug, Deserialize)]
pub struct AddFriendReq {
    pub sender_id: String,
    pub friend_code: String,
}

#[derive(Debug, Deserialize)]
pub struct AcceptFriendReq {
    pub sender_id: String,
    pub requester_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WsMessage {
    pub id: String,
    pub sender_id: String,
    pub receiver_id: String,
    pub content: String,
    pub message_type: String,
}
