use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct AffinityConnection {
    pub id: Uuid,
    pub requester_id: Uuid,
    pub receiver_id: Uuid,
    pub affinity_type: String, // partner, bro, bestie, confidant
    pub status: String,        // pending, accepted
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct AffinityRequestReq {
    pub requester_id: Uuid,
    pub receiver_id: Uuid,
    pub affinity_type: String,
}

#[derive(Debug, Deserialize)]
pub struct AffinityRespondReq {
    pub status: String, // accepted, rejected
}

#[derive(Debug, Deserialize)]
pub struct AffinityInteractReq {
    pub sender_id: Uuid,
    pub receiver_id: Uuid,
    pub action: String, // e.g., "drink_water", "poop", "sleep", "thinking"
}
