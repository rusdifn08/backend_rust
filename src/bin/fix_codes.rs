use rand::Rng;
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

    let users = sqlx::query!("SELECT id, friend_code FROM users")
        .fetch_all(&pool)
        .await?;

    let mut rng = rand::thread_rng();
    let mut updated_count = 0;

    for user in users {
        if !user.friend_code.chars().all(char::is_numeric) || user.friend_code.len() != 6 {
            let new_code = format!("{:06}", rng.gen_range(100000..999999));
            sqlx::query!(
                "UPDATE users SET friend_code = $1 WHERE id = $2",
                new_code,
                user.id
            )
            .execute(&pool)
            .await?;
            updated_count += 1;
            println!("Updated user {} with new code {}", user.id, new_code);
        }
    }

    println!(
        "Successfully updated {} users with old alphanumeric codes.",
        updated_count
    );
    Ok(())
}
