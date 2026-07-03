use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Transaction {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub title: String,
    pub date: String,
    pub amount: f64,
    pub icon: String,
    pub color: String,
    pub is_expense: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateTransactionReq {
    pub user_id: Option<String>,
    pub title: String,
    pub date: String,
    pub amount: f64,
    pub icon: String,
    pub color: String,
    pub is_expense: bool,
}
