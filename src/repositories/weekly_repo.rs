use sqlx::PgPool;
use uuid::Uuid;
use crate::models::weekly::WeeklyTask;

pub struct WeeklyRepo;

impl WeeklyRepo {
    pub async fn get_tasks(pool: &PgPool, user_id: Uuid) -> Result<Vec<WeeklyTask>, sqlx::Error> {
        let rows = sqlx::query_as::<_, WeeklyTask>(
            r#"
            SELECT * FROM weekly_tasks
            WHERE user_id = $1
            ORDER BY day_of_week ASC, created_at DESC
            "#
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;
        
        Ok(rows)
    }

    pub async fn create_task(
        pool: &PgPool,
        user_id: Uuid,
        title: &str,
        description: Option<&str>,
        day_of_week: i32,
    ) -> Result<WeeklyTask, sqlx::Error> {
        let row = sqlx::query_as::<_, WeeklyTask>(
            r#"
            INSERT INTO weekly_tasks (user_id, title, description, day_of_week)
            VALUES ($1, $2, $3, $4)
            RETURNING *
            "#
        )
        .bind(user_id)
        .bind(title)
        .bind(description)
        .bind(day_of_week)
        .fetch_one(pool)
        .await?;

        Ok(row)
    }

    pub async fn toggle_task(
        pool: &PgPool,
        id: Uuid,
    ) -> Result<WeeklyTask, sqlx::Error> {
        let row = sqlx::query_as::<_, WeeklyTask>(
            r#"
            UPDATE weekly_tasks
            SET is_completed = NOT is_completed, updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#
        )
        .bind(id)
        .fetch_one(pool)
        .await?;

        Ok(row)
    }

    pub async fn delete_task(
        pool: &PgPool,
        id: Uuid,
    ) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(
            r#"
            DELETE FROM weekly_tasks WHERE id = $1
            "#
        )
        .bind(id)
        .execute(pool)
        .await?;

        Ok(result.rows_affected())
    }
}
