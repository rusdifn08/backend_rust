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
use serde::Serialize;
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use uuid::Uuid;

use crate::models::chat::{AcceptFriendReq, AddFriendReq, ChatMessage, WsMessage};
use tracing::{error, info, warn};

// Store active connections: User ID -> Connection ID -> Sender Channel.
// A user may be logged in from more than one device/session.
pub type ChatState = Arc<Mutex<HashMap<String, HashMap<String, mpsc::UnboundedSender<WsMessage>>>>>;

use crate::api::analytics::{AnalyticsState, WsEventLog};

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
    let receiver = sqlx::query("SELECT id FROM users WHERE friend_code = $1")
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
    pub avatar_url: Option<String>,
    pub tier: Option<i32>,
    pub exp: Option<i32>,
}

pub async fn get_friends(
    Path(user_id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<Json<Vec<FriendResponse>>, (StatusCode, String)> {
    let friends = sqlx::query_as::<sqlx::Postgres, FriendResponse>(
        r#"
        SELECT u.id, u.username, f.status, u.avatar_url, g.tier, g.exp
        FROM friends f
        JOIN users u ON u.id = CASE WHEN f.user_id_1 = $1 THEN f.user_id_2 ELSE f.user_id_1 END
        LEFT JOIN user_stats g ON u.id = g.user_id
        WHERE (f.user_id_1 = $1 OR f.user_id_2 = $1)
        "#,
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
        SELECT u.id, u.username, 'search' as status, u.avatar_url, g.tier, g.exp
        FROM users u
        LEFT JOIN user_stats g ON u.id = g.user_id
        WHERE u.friend_code = $1
        "#,
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
        "#,
    )
    .bind(user1)
    .bind(user2)
    .fetch_all(&state.pool)
    .await
    .map_err(|e: sqlx::Error| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(messages))
}

// REST: Delete Chat History
pub async fn delete_chat_history(
    Path((user1, user2)): Path<(Uuid, Uuid)>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let _ = sqlx::query(
        r#"
        DELETE FROM chat_messages 
        WHERE (sender_id = $1 AND receiver_id = $2) 
           OR (sender_id = $2 AND receiver_id = $1)
        "#,
    )
    .bind(user1)
    .bind(user2)
    .execute(&state.pool)
    .await
    .map_err(|e: sqlx::Error| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok((StatusCode::OK, "Chat history deleted".to_string()))
}

// REST: Delete Single Message
pub async fn delete_single_message(
    Path(message_id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let _ = sqlx::query("DELETE FROM chat_messages WHERE id = $1")
        .bind(message_id)
        .execute(&state.pool)
        .await
        .map_err(|e: sqlx::Error| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok((StatusCode::OK, "Message deleted".to_string()))
}

// WebSocket handler
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    Path(user_id): Path<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, user_id, state))
}

async fn emit_ws_event(
    state: &AppState,
    event: &str,
    user_id: Option<String>,
    peer_id: Option<String>,
    message_id: Option<String>,
    detail: impl Into<String>,
) {
    state
        .analytics_state
        .emit_ws_event(
            event,
            WsEventLog {
                event: event.to_string(),
                user_id,
                peer_id,
                message_id,
                detail: detail.into(),
            },
        )
        .await;
}

pub async fn send_ws_to_user(chat_state: &ChatState, user_id: &str, msg: WsMessage) -> usize {
    let mut delivered = 0;
    let mut state = chat_state.lock().await;

    if let Some(connections) = state.get_mut(user_id) {
        connections.retain(|_, tx| {
            let sent = tx.send(msg.clone()).is_ok();
            if sent {
                delivered += 1;
            }
            sent
        });

        if connections.is_empty() {
            state.remove(user_id);
        }
    }

    delivered
}

fn ws_status_message(source: &WsMessage, status: &str, detail: &str) -> WsMessage {
    WsMessage {
        id: Uuid::new_v4().to_string(),
        sender_id: source.receiver_id.clone(),
        receiver_id: source.sender_id.clone(),
        content: format!("{}:{}", status, detail),
        message_type: "delivery_ack".to_string(),
    }
}

async fn send_to_connection(tx: &mpsc::UnboundedSender<WsMessage>, msg: WsMessage) {
    let _ = tx.send(msg);
}

async fn handle_socket(socket: WebSocket, user_id: String, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    let (tx, mut rx) = mpsc::unbounded_channel::<WsMessage>();
    let connection_id = Uuid::new_v4().to_string();

    {
        let mut chat_state = state.chat_state.lock().await;
        chat_state
            .entry(user_id.clone())
            .or_default()
            .insert(connection_id.clone(), tx.clone());
    }

    info!("User {} connected to chat ({})", user_id, connection_id);
    emit_ws_event(
        &state,
        "connected",
        Some(user_id.clone()),
        None,
        None,
        format!("connection {}", connection_id),
    )
    .await;

    // Task to send messages from channel to websocket client
    let mut send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            match serde_json::to_string(&msg) {
                Ok(msg_str) => {
                    if sender.send(Message::Text(msg_str)).await.is_err() {
                        break;
                    }
                }
                Err(err) => {
                    error!("Failed to serialize websocket message: {}", err);
                    break;
                }
            }
        }
    });

    // Task to receive messages from websocket client
    let state_clone = state.clone();
    let user_id_for_recv = user_id.clone();
    let tx_for_ack = tx.clone();

    let mut recv_task = tokio::spawn(async move {
        while let Some(result) = receiver.next().await {
            let text = match result {
                Ok(Message::Text(text)) => text,
                Ok(Message::Close(_)) => break,
                Ok(_) => continue,
                Err(err) => {
                    warn!("WebSocket receive error for {}: {}", user_id_for_recv, err);
                    emit_ws_event(
                        &state_clone,
                        "error",
                        Some(user_id_for_recv.clone()),
                        None,
                        None,
                        format!("receive error: {}", err),
                    )
                    .await;
                    break;
                }
            };

            let msg = match serde_json::from_str::<WsMessage>(&text) {
                Ok(msg) => msg,
                Err(err) => {
                    warn!(
                        "Invalid websocket payload from {}: {}",
                        user_id_for_recv, err
                    );
                    emit_ws_event(
                        &state_clone,
                        "error",
                        Some(user_id_for_recv.clone()),
                        None,
                        None,
                        format!("invalid payload: {}", err),
                    )
                    .await;
                    continue;
                }
            };

            if msg.sender_id != user_id_for_recv {
                warn!(
                    "Rejected websocket message: path user {} tried sender_id {}",
                    user_id_for_recv, msg.sender_id
                );
                send_to_connection(
                    &tx_for_ack,
                    ws_status_message(&msg, "error", "sender_id_mismatch"),
                )
                .await;
                emit_ws_event(
                    &state_clone,
                    "error",
                    Some(user_id_for_recv.clone()),
                    Some(msg.receiver_id.clone()),
                    Some(msg.id.clone()),
                    "sender_id mismatch",
                )
                .await;
                continue;
            }

            emit_ws_event(
                &state_clone,
                "message_received",
                Some(msg.sender_id.clone()),
                Some(msg.receiver_id.clone()),
                Some(msg.id.clone()),
                format!("type {}", msg.message_type),
            )
            .await;

            let pool = &state_clone.pool;

            if msg.message_type == "read_receipt" {
                if let Ok(msg_uuid) = Uuid::parse_str(&msg.content) {
                    if let Err(err) =
                        sqlx::query("UPDATE chat_messages SET is_read = TRUE WHERE id = $1")
                            .bind(msg_uuid)
                            .execute(pool)
                            .await
                    {
                        warn!("Failed to mark message {} as read: {}", msg.content, err);
                        emit_ws_event(
                            &state_clone,
                            "error",
                            Some(msg.sender_id.clone()),
                            Some(msg.receiver_id.clone()),
                            Some(msg.content.clone()),
                            format!("read receipt DB error: {}", err),
                        )
                        .await;
                    }
                }

                let delivered =
                    send_ws_to_user(&state_clone.chat_state, &msg.receiver_id, msg.clone()).await;
                let event = if delivered > 0 {
                    "message_delivered"
                } else {
                    "message_offline"
                };
                emit_ws_event(
                    &state_clone,
                    event,
                    Some(msg.sender_id.clone()),
                    Some(msg.receiver_id.clone()),
                    Some(msg.content.clone()),
                    format!("read receipt delivered to {} connection(s)", delivered),
                )
                .await;
                continue;
            }

            let sender_uuid = match Uuid::parse_str(&msg.sender_id) {
                Ok(id) => id,
                Err(err) => {
                    send_to_connection(
                        &tx_for_ack,
                        ws_status_message(&msg, "error", "invalid_sender_id"),
                    )
                    .await;
                    emit_ws_event(
                        &state_clone,
                        "error",
                        Some(user_id_for_recv.clone()),
                        Some(msg.receiver_id.clone()),
                        Some(msg.id.clone()),
                        format!("invalid sender id: {}", err),
                    )
                    .await;
                    continue;
                }
            };

            let receiver_uuid = match Uuid::parse_str(&msg.receiver_id) {
                Ok(id) => id,
                Err(err) => {
                    send_to_connection(
                        &tx_for_ack,
                        ws_status_message(&msg, "error", "invalid_receiver_id"),
                    )
                    .await;
                    emit_ws_event(
                        &state_clone,
                        "error",
                        Some(msg.sender_id.clone()),
                        Some(msg.receiver_id.clone()),
                        Some(msg.id.clone()),
                        format!("invalid receiver id: {}", err),
                    )
                    .await;
                    continue;
                }
            };

            let msg_uuid = Uuid::parse_str(&msg.id).unwrap_or_else(|_| Uuid::new_v4());

            let insert_result = sqlx::query(
                r#"
                INSERT INTO chat_messages (id, sender_id, receiver_id, content, message_type, is_read)
                VALUES ($1, $2, $3, $4, $5, FALSE)
                ON CONFLICT (id) DO UPDATE
                SET content = EXCLUDED.content,
                    message_type = EXCLUDED.message_type
                "#
            )
            .bind(msg_uuid)
            .bind(sender_uuid)
            .bind(receiver_uuid)
            .bind(msg.content.clone())
            .bind(msg.message_type.clone())
            .execute(pool)
            .await;

            if let Err(err) = insert_result {
                error!("Failed to save chat message {}: {}", msg.id, err);
                send_to_connection(
                    &tx_for_ack,
                    ws_status_message(&msg, "error", "db_insert_failed"),
                )
                .await;
                emit_ws_event(
                    &state_clone,
                    "error",
                    Some(msg.sender_id.clone()),
                    Some(msg.receiver_id.clone()),
                    Some(msg.id.clone()),
                    format!("DB insert failed: {}", err),
                )
                .await;
                continue;
            }

            let delivered =
                send_ws_to_user(&state_clone.chat_state, &msg.receiver_id, msg.clone()).await;
            let (event, detail) = if delivered > 0 {
                (
                    "message_delivered",
                    format!("delivered to {} connection(s)", delivered),
                )
            } else {
                (
                    "message_offline",
                    "receiver offline; saved to DB".to_string(),
                )
            };

            send_to_connection(&tx_for_ack, ws_status_message(&msg, "stored", &detail)).await;
            emit_ws_event(
                &state_clone,
                event,
                Some(msg.sender_id.clone()),
                Some(msg.receiver_id.clone()),
                Some(msg.id.clone()),
                detail,
            )
            .await;
        }
    });

    // Wait for either task to finish
    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    };

    {
        let mut chat_state = state.chat_state.lock().await;
        if let Some(connections) = chat_state.get_mut(&user_id) {
            connections.remove(&connection_id);
            if connections.is_empty() {
                chat_state.remove(&user_id);
            }
        }
    }

    info!(
        "User {} disconnected from chat ({})",
        user_id, connection_id
    );
    emit_ws_event(
        &state,
        "disconnected",
        Some(user_id),
        None,
        None,
        format!("connection {}", connection_id),
    )
    .await;
}
