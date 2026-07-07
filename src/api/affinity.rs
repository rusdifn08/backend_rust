use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post, put},
    Json, Router,
};
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use crate::api::chat::AppState;
use crate::models::affinity::{
    AffinityConnection, AffinityInteractReq, AffinityRequestReq, AffinityRespondReq,
};
use crate::models::chat::WsMessage;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/request", post(request_affinity))
        .route("/respond/:id", put(respond_affinity))
        .route("/user/:user_id", get(get_user_affinities))
        .route("/interact", post(interact_affinity))
}

async fn check_affinity_limit(
    pool: &PgPool,
    user_id: Uuid,
    affinity_type: &str,
) -> Result<bool, sqlx::Error> {
    let limit = match affinity_type {
        "partner" => 1,
        "bro" => 4,
        "bestie" => 4,
        "confidant" => 5,
        _ => return Ok(false),
    };

    let count: (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*) FROM affinity_connections
        WHERE (requester_id = $1 OR receiver_id = $1)
        AND affinity_type = $2
        AND status = 'accepted'
        "#,
    )
    .bind(user_id)
    .bind(affinity_type)
    .fetch_one(pool)
    .await?;

    Ok(count.0 < limit)
}

pub async fn request_affinity(
    State(state): State<AppState>,
    Json(req): Json<AffinityRequestReq>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // 1. Check Limits for both users
    let sender_ok = check_affinity_limit(&state.pool, req.requester_id, &req.affinity_type)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    let receiver_ok = check_affinity_limit(&state.pool, req.receiver_id, &req.affinity_type)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if !sender_ok || !receiver_ok {
        return Err((
            StatusCode::BAD_REQUEST,
            "Slot for this affinity type is full".to_string(),
        ));
    }

    // 2. Insert Request
    let result = sqlx::query(
        r#"
        INSERT INTO affinity_connections (requester_id, receiver_id, affinity_type, status)
        VALUES ($1, $2, $3, 'pending')
        ON CONFLICT (requester_id, receiver_id) DO NOTHING
        RETURNING id
        "#,
    )
    .bind(req.requester_id)
    .bind(req.receiver_id)
    .bind(&req.affinity_type)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e: sqlx::Error| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if result.is_none() {
        return Err((StatusCode::CONFLICT, "Affinity connection already exists".to_string()));
    }

    // 3. Send Real-time WS Notification
    let ws_msg = WsMessage {
        id: Uuid::new_v4().to_string(),
        sender_id: req.requester_id.to_string(),
        receiver_id: req.receiver_id.to_string(),
        content: req.affinity_type,
        message_type: "affinity_request".to_string(),
    };

    let chat_state = state.chat_state.lock().await;
    if let Some(tx) = chat_state.get(&req.receiver_id.to_string()) {
        let _ = tx.send(ws_msg);
    }

    Ok((StatusCode::OK, "Affinity request sent".to_string()))
}

pub async fn respond_affinity(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
    Json(req): Json<AffinityRespondReq>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    if req.status == "accepted" {
        // Double check limits before accepting
        let conn = sqlx::query_as::<_, AffinityConnection>(
            "SELECT * FROM affinity_connections WHERE id = $1 AND status = 'pending'"
        )
        .bind(id)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        if let Some(c) = conn {
            let sender_ok = check_affinity_limit(&state.pool, c.requester_id, &c.affinity_type)
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
            let receiver_ok = check_affinity_limit(&state.pool, c.receiver_id, &c.affinity_type)
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

            if !sender_ok || !receiver_ok {
                return Err((
                    StatusCode::BAD_REQUEST,
                    "Slot for this affinity type is full".to_string(),
                ));
            }
        } else {
            return Err((StatusCode::NOT_FOUND, "Pending request not found".to_string()));
        }

        let _ = sqlx::query(
            "UPDATE affinity_connections SET status = 'accepted' WHERE id = $1",
        )
        .bind(id)
        .execute(&state.pool)
        .await
        .map_err(|e: sqlx::Error| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        Ok((StatusCode::OK, "Affinity accepted".to_string()))
    } else {
        let _ = sqlx::query(
            "DELETE FROM affinity_connections WHERE id = $1",
        )
        .bind(id)
        .execute(&state.pool)
        .await
        .map_err(|e: sqlx::Error| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        Ok((StatusCode::OK, "Affinity rejected".to_string()))
    }
}

pub async fn get_user_affinities(
    Path(user_id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<Json<Vec<AffinityConnection>>, (StatusCode, String)> {
    let connections = sqlx::query_as::<_, AffinityConnection>(
        r#"
        SELECT * FROM affinity_connections 
        WHERE requester_id = $1 OR receiver_id = $1
        "#
    )
    .bind(user_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(connections))
}

pub async fn interact_affinity(
    State(state): State<AppState>,
    Json(req): Json<AffinityInteractReq>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // We just emit WS message for interactivity
    let ws_msg = WsMessage {
        id: Uuid::new_v4().to_string(),
        sender_id: req.sender_id.to_string(),
        receiver_id: req.receiver_id.to_string(),
        content: req.action,
        message_type: "affinity_interaction".to_string(),
    };

    let chat_state = state.chat_state.lock().await;
    if let Some(tx) = chat_state.get(&req.receiver_id.to_string()) {
        let _ = tx.send(ws_msg);
    }

    Ok((StatusCode::OK, "Interaction sent".to_string()))
}
