use sqlx::PgPool;
use std::fs;
use std::path::Path;
use dotenvy::dotenv;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPool::connect(&database_url).await?;

    println!("Running database migrations...");
    sqlx::migrate!("./migrations").run(&pool).await?;

    println!("Seeding tier borders into the database...");

    for tier in 1..=10 {
        let path_str = format!("src/Assets/tier{}.png", tier);
        let path = Path::new(&path_str);

        if path.exists() {
            let image_data = fs::read(path)?;
            
            sqlx::query(
                r#"
                INSERT INTO tier_borders (tier, image_data)
                VALUES ($1, $2)
                ON CONFLICT (tier) DO UPDATE SET image_data = EXCLUDED.image_data, updated_at = NOW()
                "#
            )
            .bind(tier as i32)
            .bind(image_data.clone())
            .execute(&pool)
            .await?;
            
            println!("Successfully seeded tier {}: {} bytes", tier, image_data.len());
        } else {
            println!("Warning: Border image not found at {:?}", path);
        }
    }

    // Set rusdifn08 to Gamemaster status
    println!("Setting rusdifn08 as Gamemaster (Tier 10)...");
    let res = sqlx::query(
        r#"
        UPDATE user_stats 
        SET exp = 1000000, coins = 99999
        WHERE user_id = (SELECT id FROM users WHERE username = 'rusdifn08')
        "#
    ).execute(&pool).await?;
    
    println!("Rows updated for gamemaster: {}", res.rows_affected());

    println!("Done!");
    Ok(())
}
