use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use sqlx::PgPool;
use crate::models::badge::UserBadgeResponse;

pub async fn get_user_badges(
    State(pool): State<PgPool>,
    Path(user_id): Path<String>,
) -> Result<Json<Vec<UserBadgeResponse>>, (StatusCode, String)> {
    let badges = sqlx::query_as::<_, UserBadgeResponse>(
        r#"
        SELECT 
            ub.id, ub.user_id, ub.badge_id, ub.earned_at,
            b.name, b.description, b.icon
        FROM user_badges ub
        JOIN badges b ON ub.badge_id = b.id
        WHERE ub.user_id = $1
        ORDER BY ub.earned_at DESC
        "#
    )
    .bind(user_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(badges))
}
