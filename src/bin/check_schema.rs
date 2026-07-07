use sqlx::postgres::PgPoolOptions;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    // Get schema of gamification_stats
    let rows: Vec<(String, String)> = sqlx::query_as(
        "SELECT column_name, data_type FROM information_schema.columns WHERE table_name = 'gamification_stats'"
    )
    .fetch_all(&pool)
    .await?;

    println!("gamification_stats columns:");
    for (name, dtype) in rows {
        println!("{}: {}", name, dtype);
    }

    // Get schema of users
    let rows2: Vec<(String, String)> = sqlx::query_as(
        "SELECT column_name, data_type FROM information_schema.columns WHERE table_name = 'users'",
    )
    .fetch_all(&pool)
    .await?;

    println!("users columns:");
    for (name, dtype) in rows2 {
        println!("{}: {}", name, dtype);
    }

    Ok(())
}
