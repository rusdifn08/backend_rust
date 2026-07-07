use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use sqlx::PgPool;
use uuid::Uuid;
use crate::models::squad::{Squad, CreateSquadReq, SquadMemberResponse};

pub async fn create_squad(
    State(pool): State<PgPool>,
    Json(req): Json<CreateSquadReq>,
) -> Result<Json<Squad>, (StatusCode, String)> {
    let mut tx = pool.begin().await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let squad = sqlx::query_as::<_, Squad>(
        r#"
        INSERT INTO squads (name, description) 
        VALUES ($1, $2) 
        RETURNING *
        "#
    )
    .bind(&req.name)
    .bind(&req.description)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    sqlx::query(
        r#"
        INSERT INTO squad_members (squad_id, user_id, role) 
        VALUES ($1, $2, 'leader')
        "#
    )
    .bind(squad.id)
    .bind(&req.user_id)
    .execute(&mut *tx)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    tx.commit().await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(squad))
}

pub async fn get_squad_leaderboard(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<SquadMemberResponse>>, (StatusCode, String)> {
    let members = sqlx::query_as::<_, SquadMemberResponse>(
        r#"
        SELECT 
            sm.id, sm.squad_id, sm.user_id, sm.role, sm.joined_at,
            u.username, u.avatar_url
        FROM squad_members sm
        JOIN users u ON sm.user_id = u.id
        WHERE sm.squad_id = $1
        ORDER BY u.created_at ASC
        "#
    )
    .bind(id)
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(members))
}
