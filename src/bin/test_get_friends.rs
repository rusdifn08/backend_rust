use sqlx::postgres::PgPoolOptions;
use std::env;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    let row = sqlx::query(
        "SELECT u.id, u.username, f.status, u.avatar_url, g.tier, g.exp
        FROM friends f
        JOIN users u ON u.id = CASE WHEN f.user_id_1 = '00000000-0000-0000-0000-000000000000' THEN f.user_id_2 ELSE f.user_id_1 END
        LEFT JOIN gamification_stats g ON u.id = g.user_id"
    )
    .fetch_optional(&pool)
    .await;
    
    match row {
        Ok(_) => println!("Query succeeded!"),
        Err(e) => println!("Query failed: {}", e),
    }

    Ok(())
}

