use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, State,
    },
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use futures::{sink::SinkExt, stream::StreamExt};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use uuid::Uuid;

use crate::models::chat::{AddFriendReq, AcceptFriendReq, ChatMessage, WsMessage};
use crate::models::user::User;
use tracing::{error, info};

// Store active connections: User ID -> Sender Channel
pub type ChatState = Arc<Mutex<HashMap<String, mpsc::UnboundedSender<WsMessage>>>>;

use crate::api::analytics::AnalyticsState;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub chat_state: ChatState,
    pub analytics_state: AnalyticsState,
}

impl axum::extract::FromRef<AppState> for PgPool {
    fn from_ref(app_state: &AppState) -> PgPool {
        app_state.pool.clone()
    }
}

impl axum::extract::FromRef<AppState> for AnalyticsState {
    fn from_ref(app_state: &AppState) -> AnalyticsState {
        app_state.analytics_state.clone()
    }
}

pub async fn add_friend(
    State(state): State<AppState>,
    Json(req): Json<AddFriendReq>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // 1. Get the current user from auth context (Mock for now, using dummy or we need sender_id)
    // Actually, AddFriendReq needs a sender_id for MVP if no JWT extraction
    // Let's assume req.sender_id is provided in MVP
    
    // Find receiver by friend_code
    use sqlx::Row;
    let receiver = sqlx::query(
        "SELECT id FROM users WHERE friend_code = $1"
    )
    .bind(&req.friend_code)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e: sqlx::Error| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let receiver_id: Uuid = match receiver {
        Some(r) => r.get("id"),
        None => return Err((StatusCode::NOT_FOUND, "Friend code not found".to_string())),
    };

    let sender_uuid = Uuid::parse_str(&req.sender_id).unwrap_or_default();

    // Insert pending request
    let _ = sqlx::query!(
        r#"
        INSERT INTO friends (user_id_1, user_id_2, status)
        VALUES ($1, $2, 'pending')
        ON CONFLICT DO NOTHING
        "#,
        sender_uuid,
        receiver_id
    )
    .execute(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok((StatusCode::OK, "Friend request sent".to_string()))
}

pub async fn accept_friend(
    State(state): State<AppState>,
    Json(req): Json<AcceptFriendReq>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let sender_uuid = Uuid::parse_str(&req.sender_id).unwrap_or_default();
    let requester_uuid = Uuid::parse_str(&req.requester_id).unwrap_or_default();

    let _ = sqlx::query!(
        r#"
        UPDATE friends 
        SET status = 'accepted'
        WHERE (user_id_1 = $1 AND user_id_2 = $2)
           OR (user_id_1 = $2 AND user_id_2 = $1)
        "#,
        requester_uuid,
        sender_uuid
    )
    .execute(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok((StatusCode::OK, "Friend request accepted".to_string()))
}

// REST: Get Friends
#[derive(Serialize, sqlx::FromRow)]
pub struct FriendResponse {
    pub id: Uuid,
    pub username: String,
    pub status: String,
}

pub async fn get_friends(
    Path(user_id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<Json<Vec<FriendResponse>>, (StatusCode, String)> {
    let friends = sqlx::query_as::<sqlx::Postgres, FriendResponse>(
        r#"
        SELECT u.id, u.username, f.status
        FROM friends f
        JOIN users u ON u.id = CASE WHEN f.user_id_1 = $1 THEN f.user_id_2 ELSE f.user_id_1 END
        WHERE (f.user_id_1 = $1 OR f.user_id_2 = $1)
        "#
    )
    .bind(user_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|e: sqlx::Error| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(friends))
}

pub async fn search_friend(
    Path(code): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<FriendResponse>, (StatusCode, String)> {
    let user = sqlx::query_as::<sqlx::Postgres, FriendResponse>(
        r#"
        SELECT id, username, 'search' as status 
        FROM users 
        WHERE friend_code = $1
        "#
    )
    .bind(code)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e: sqlx::Error| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    match user {
        Some(u) => Ok(Json(u)),
        None => Err((StatusCode::NOT_FOUND, "User not found".to_string())),
    }
}

pub async fn get_chat_history(
    Path((user1, user2)): Path<(Uuid, Uuid)>,
    State(state): State<AppState>,
) -> Result<Json<Vec<ChatMessage>>, (StatusCode, String)> {
    let messages = sqlx::query_as::<_, ChatMessage>(
        r#"
        SELECT * FROM chat_messages 
        WHERE ((sender_id = $1 AND receiver_id = $2) 
           OR (sender_id = $2 AND receiver_id = $1))
           AND created_at >= NOW() - INTERVAL '24 hours'
        ORDER BY created_at ASC
        "#
    )
    .bind(user1)
    .bind(user2)
    .fetch_all(&state.pool)
    .await
    .map_err(|e: sqlx::Error| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(messages))
}

// WebSocket handler
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    Path(user_id): Path<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, user_id, state))
}

async fn handle_socket(socket: WebSocket, user_id: String, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    let (tx, mut rx) = mpsc::unbounded_channel::<WsMessage>();

    // Register user
    state.chat_state.lock().await.insert(user_id.clone(), tx);
    info!("User {} connected to chat", user_id);

    // Task to send messages from channel to websocket client
    let mut send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            let msg_str = serde_json::to_string(&msg).unwrap();
            if sender.send(Message::Text(msg_str)).await.is_err() {
                break;
            }
        }
    });

    // Task to receive messages from websocket client
    let state_clone = state.clone();
    
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(Message::Text(text))) = receiver.next().await {
            if let Ok(msg) = serde_json::from_str::<WsMessage>(&text) {
                // Save to DB
                let pool = &state_clone.pool;
                let sender_uuid = Uuid::parse_str(&msg.sender_id).unwrap_or_default();
                let receiver_uuid = Uuid::parse_str(&msg.receiver_id).unwrap_or_default();
                
                let _ = sqlx::query(
                    r#"
                    INSERT INTO chat_messages (sender_id, receiver_id, content, message_type)
                    VALUES ($1, $2, $3, $4)
                    "#
                )
                .bind(sender_uuid)
                .bind(receiver_uuid)
                .bind(msg.content.clone())
                .bind(msg.message_type.clone())
                .execute(pool)
                .await;

                // Send to receiver if online
                let chat_state = state_clone.chat_state.lock().await;
                if let Some(receiver_tx) = chat_state.get(&msg.receiver_id) {
                    let _ = receiver_tx.send(msg);
                }
            }
        }
    });

    // Wait for either task to finish
    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    };

    // Unregister user
    state.chat_state.lock().await.remove(&user_id);
    info!("User {} disconnected from chat", user_id);
}
