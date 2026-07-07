use sqlx::PgPool;
use std::env;

#[derive(sqlx::FromRow)]
struct UserRow {
    id: uuid::Uuid,
    username: String,
    email: String,
    friend_code: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPool::connect(&database_url).await?;

    let users = sqlx::query_as::<sqlx::Postgres, UserRow>(
        "SELECT id, username, email, friend_code FROM users",
    )
    .fetch_all(&pool)
    .await?;

    println!("--- LIST OF USERS ---");
    for user in users {
        println!(
            "Name: {} | Code: {} | ID: {}",
            user.username, user.friend_code, user.id
        );
    }
    println!("---------------------");

    Ok(())
}
