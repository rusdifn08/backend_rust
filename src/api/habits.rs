use crate::models::habit::{CreateHabitLogReq, CreateHabitReq, Habit, HabitLog, UpdateHabitReq};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn get_habits(
    Path(user_id): Path<Uuid>,
    State(pool): State<PgPool>,
) -> Result<Json<Vec<Habit>>, (StatusCode, String)> {
    let habits = sqlx::query_as::<_, Habit>(
        "SELECT * FROM habits WHERE user_id = $1 ORDER BY created_at DESC",
    )
    .bind(user_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(habits))
}

pub async fn create_habit(
    State(pool): State<PgPool>,
    Json(req): Json<CreateHabitReq>,
) -> Result<(StatusCode, Json<Habit>), (StatusCode, String)> {
    let user_uuid = Uuid::parse_str(&req.user_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid UUID".into()))?;
    let habit_uuid = match req.id {
        Some(id) => Uuid::parse_str(&id)
            .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid habit UUID".into()))?,
        None => Uuid::new_v4(),
    };
    let habit = sqlx::query_as::<_, Habit>(
        r#"
        INSERT INTO habits (id, user_id, title, time, icon, color, streak, category, description, frequency) 
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        ON CONFLICT (id) DO UPDATE
        SET title = EXCLUDED.title,
            time = EXCLUDED.time,
            icon = EXCLUDED.icon,
            color = EXCLUDED.color,
            category = EXCLUDED.category,
            description = EXCLUDED.description,
            frequency = EXCLUDED.frequency,
            updated_at = NOW()
        RETURNING *
        "#
    )
    .bind(habit_uuid)
    .bind(user_uuid)
    .bind(req.title)
    .bind(req.time.unwrap_or_else(|| "Anytime".to_string()))
    .bind(req.icon)
    .bind(req.color)
    .bind(req.streak.unwrap_or(0))
    .bind(req.category)
    .bind(req.description.or(req.subtitle))
    .bind(req.frequency.unwrap_or_else(|| {
        req.target_days
            .map(|days| format!("{}x", days))
            .unwrap_or_else(|| "daily".to_string())
    }))
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok((StatusCode::CREATED, Json(habit)))
}

pub async fn toggle_habit(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<Json<Habit>, (StatusCode, String)> {
    let habit_before = sqlx::query_as::<_, Habit>("SELECT * FROM habits WHERE id = $1")
        .bind(id)
        .fetch_one(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let _ = sqlx::query("INSERT INTO habit_logs (habit_id, user_id, note) VALUES ($1, $2, $3)")
        .bind(id)
        .bind(habit_before.user_id)
        .bind("Checked from toggle")
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let habit = sqlx::query_as::<_, Habit>(
        r#"
        UPDATE habits 
        SET is_completed = TRUE, streak = streak + 1, updated_at = NOW() 
        WHERE id = $1 
        RETURNING *
        "#,
    )
    .bind(id)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(habit))
}

pub async fn delete_habit(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    sqlx::query("DELETE FROM habits WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_habit_logs(
    Path(habit_id): Path<Uuid>,
    State(pool): State<PgPool>,
) -> Result<Json<Vec<HabitLog>>, (StatusCode, String)> {
    let logs = sqlx::query_as::<_, HabitLog>(
        "SELECT * FROM habit_logs WHERE habit_id = $1 ORDER BY created_at DESC",
    )
    .bind(habit_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(logs))
}

pub async fn add_habit_log(
    State(pool): State<PgPool>,
    Json(req): Json<CreateHabitLogReq>,
) -> Result<(StatusCode, Json<HabitLog>), (StatusCode, String)> {
    let habit_uuid = Uuid::parse_str(&req.habit_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid UUID".into()))?;
    let user_uuid = req
        .user_id
        .map(|id| Uuid::parse_str(&id).unwrap_or(Uuid::default()));

    let log = sqlx::query_as::<_, HabitLog>(
        r#"
        INSERT INTO habit_logs (habit_id, user_id, note) 
        VALUES ($1, $2, $3) 
        RETURNING *
        "#,
    )
    .bind(habit_uuid)
    .bind(user_uuid)
    .bind(req.note)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Update streak on the habit when a new log is added
    let _ = sqlx::query("UPDATE habits SET streak = streak + 1, is_completed = TRUE, updated_at = NOW() WHERE id = $1")
        .bind(habit_uuid)
        .execute(&pool)
        .await;

    if let Some(uid) = user_uuid {
        // Gamification Reward
        let _ = crate::services::gamification_service::GamificationService::add_reward(
            &pool, uid, 15, 10, // 15 EXP, 10 Coins
        )
        .await;

        // Fetch habit title
        if let Ok(habit_data) = sqlx::query!("SELECT title FROM habits WHERE id = $1", habit_uuid)
            .fetch_one(&pool)
            .await
        {
            // Social Feed
            let desc = format!("Completed a habit: {}", habit_data.title);
            let _ = crate::repositories::social_repo::SocialRepo::create_activity(
                &pool,
                uid,
                "HABIT_COMPLETED",
                &desc,
            )
            .await;
        }
    }

    Ok((StatusCode::CREATED, Json(log)))
}

pub async fn update_habit(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateHabitReq>,
) -> Result<Json<Habit>, (StatusCode, String)> {
    let habit = sqlx::query_as::<_, Habit>(
        r#"
        UPDATE habits 
        SET title = $1,
            description = $2,
            category = $3,
            frequency = $4,
            color = $5,
            icon = COALESCE($6, icon),
            updated_at = NOW() 
        WHERE id = $7
        RETURNING *
        "#,
    )
    .bind(req.title)
    .bind(req.subtitle)
    .bind(req.category)
    .bind(req.target_days.map(|days| format!("{}x", days)))
    .bind(req.color)
    .bind(req.icon)
    .bind(id)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(habit))
}
