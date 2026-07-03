use axum::{
    extract::{State, Path},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;
use crate::api::chat::AppState;

#[derive(Serialize, sqlx::FromRow)]
pub struct ShopItem {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub price: i32,
    pub category: String, // mapped from enum
    pub image_url: Option<String>,
    pub max_purchases: Option<i32>,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct InventoryItem {
    pub id: Uuid,
    pub user_id: Uuid,
    pub item_id: Uuid,
    pub quantity: i32,
    pub is_equipped: bool,
    pub acquired_at: Option<chrono::DateTime<chrono::Utc>>,
    // Joined fields
    pub name: String,
    pub category: String,
    pub image_url: Option<String>,
}

#[derive(Deserialize)]
pub struct BuyRequest {
    pub user_id: Uuid,
    pub item_id: Uuid,
    pub quantity: i32,
}

pub async fn get_shop_items(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let items = sqlx::query_as::<_, ShopItem>(
        r#"
        SELECT id, name, description, price, category::text as "category", image_url, max_purchases
        FROM shop_items
        ORDER BY category, price ASC
        "#
    )
    .fetch_all(&state.pool)
    .await;

    match items {
        Ok(data) => (StatusCode::OK, Json(json!({ "success": true, "data": data }))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": e.to_string() }))).into_response(),
    }
}

pub async fn get_inventory(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> impl IntoResponse {
    let inventory = sqlx::query_as::<_, InventoryItem>(
        r#"
        SELECT i.id, i.user_id, i.item_id, i.quantity, i.is_equipped, i.acquired_at,
               s.name, s.category::text as "category", s.image_url
        FROM user_inventory i
        JOIN shop_items s ON i.item_id = s.id
        WHERE i.user_id = $1
        ORDER BY s.category, s.name ASC
        "#
    )
    .bind(user_id)
    .fetch_all(&state.pool)
    .await;

    match inventory {
        Ok(data) => (StatusCode::OK, Json(json!({ "success": true, "data": data }))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": e.to_string() }))).into_response(),
    }
}

pub async fn buy_item(
    State(state): State<AppState>,
    Json(payload): Json<BuyRequest>,
) -> impl IntoResponse {
    let mut tx = match state.pool.begin().await {
        Ok(tx) => tx,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": e.to_string() }))).into_response(),
    };

    // Check item exists and price
    #[derive(sqlx::FromRow)]
    struct ShopItemPrice { price: i32, max_purchases: Option<i32> }
    
    let item = match sqlx::query_as::<_, ShopItemPrice>(r#"SELECT price, max_purchases FROM shop_items WHERE id = $1"#)
        .bind(payload.item_id)
        .fetch_optional(&mut *tx).await {
        Ok(Some(i)) => i,
        Ok(None) => return (StatusCode::NOT_FOUND, Json(json!({ "success": false, "error": "Item not found" }))).into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": e.to_string() }))).into_response(),
    };

    let total_cost = item.price * payload.quantity;

    // Check user coins
    #[derive(sqlx::FromRow)]
    struct UserStatsCoins { coins: i32 }
    let stats = match sqlx::query_as::<_, UserStatsCoins>(r#"SELECT coins FROM user_stats WHERE user_id = $1"#)
        .bind(payload.user_id)
        .fetch_optional(&mut *tx).await {
        Ok(Some(s)) => s,
        Ok(None) => return (StatusCode::NOT_FOUND, Json(json!({ "success": false, "error": "User stats not found" }))).into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": e.to_string() }))).into_response(),
    };

    if stats.coins < total_cost {
        return (StatusCode::BAD_REQUEST, Json(json!({ "success": false, "error": "Not enough coins" }))).into_response();
    }

    // Check max_purchases and current inventory
    #[derive(sqlx::FromRow)]
    struct InvQty { quantity: i32 }
    let current_inv = sqlx::query_as::<_, InvQty>(r#"SELECT quantity FROM user_inventory WHERE user_id = $1 AND item_id = $2"#)
        .bind(payload.user_id)
        .bind(payload.item_id)
        .fetch_optional(&mut *tx)
        .await
        .unwrap_or(None);

    if let Some(max_p) = item.max_purchases {
        let current_qty = current_inv.as_ref().map(|i| i.quantity).unwrap_or(0);
        if current_qty + payload.quantity > max_p {
            return (StatusCode::BAD_REQUEST, Json(json!({ "success": false, "error": "Purchase limit reached" }))).into_response();
        }
    }

    // Deduct coins
    let _ = sqlx::query(r#"UPDATE user_stats SET coins = coins - $1 WHERE user_id = $2"#)
        .bind(total_cost)
        .bind(payload.user_id)
        .execute(&mut *tx)
        .await;

    // Add to inventory
    if current_inv.is_some() {
        let _ = sqlx::query(r#"UPDATE user_inventory SET quantity = quantity + $1 WHERE user_id = $2 AND item_id = $3"#)
            .bind(payload.quantity)
            .bind(payload.user_id)
            .bind(payload.item_id)
            .execute(&mut *tx)
            .await;
    } else {
        let _ = sqlx::query(r#"INSERT INTO user_inventory (user_id, item_id, quantity) VALUES ($1, $2, $3)"#)
            .bind(payload.user_id)
            .bind(payload.item_id)
            .bind(payload.quantity)
            .execute(&mut *tx)
            .await;
    }

    if tx.commit().await.is_err() {
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Transaction failed" }))).into_response();
    }

    (StatusCode::OK, Json(json!({ "success": true, "message": "Purchase successful" }))).into_response()
}

pub async fn equip_item(
    State(state): State<AppState>,
    Path(inventory_id): Path<Uuid>,
) -> impl IntoResponse {
    let mut tx = match state.pool.begin().await {
        Ok(tx) => tx,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": e.to_string() }))).into_response(),
    };

    #[derive(sqlx::FromRow)]
    struct EquipItemRow { user_id: Uuid, is_equipped: bool, category: String }
    let inv = match sqlx::query_as::<_, EquipItemRow>(
        r#"
        SELECT i.user_id, i.is_equipped, s.category::text as "category"
        FROM user_inventory i
        JOIN shop_items s ON i.item_id = s.id
        WHERE i.id = $1
        "#
    )
    .bind(inventory_id)
    .fetch_optional(&mut *tx)
    .await {
        Ok(Some(i)) => i,
        Ok(None) => return (StatusCode::NOT_FOUND, Json(json!({ "success": false, "error": "Inventory item not found" }))).into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": e.to_string() }))).into_response(),
    };

    // If equipping, unequip others in the same category
    if !inv.is_equipped {
        let _ = sqlx::query(
            r#"
            UPDATE user_inventory
            SET is_equipped = false
            WHERE user_id = $1 AND id IN (
                SELECT i.id FROM user_inventory i JOIN shop_items s ON i.item_id = s.id WHERE s.category::text = $2
            )
            "#
        )
        .bind(inv.user_id)
        .bind(inv.category)
        .execute(&mut *tx)
        .await;
    }

    // Toggle equipped state
    let _ = sqlx::query(
        r#"
        UPDATE user_inventory
        SET is_equipped = $1
        WHERE id = $2
        "#
    )
    .bind(!inv.is_equipped)
    .bind(inventory_id)
    .execute(&mut *tx)
    .await;

    if tx.commit().await.is_err() {
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Transaction failed" }))).into_response();
    }

    (StatusCode::OK, Json(json!({ "success": true, "message": "Equipped state updated" }))).into_response()
}
