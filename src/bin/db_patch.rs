use sqlx::postgres::PgPoolOptions;
use std::env;
use std::fs;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    println!("Reading borders from Assets folder...");

    for tier in 1..=10 {
        let file_path = PathBuf::from(format!("src/Assets/tier{}.png", tier));
        
        if file_path.exists() {
            let img_bytes = fs::read(&file_path)?;
            
            sqlx::query(
                "INSERT INTO tier_borders (tier, image_data) VALUES ($1, $2) ON CONFLICT (tier) DO UPDATE SET image_data = EXCLUDED.image_data"
            )
            .bind(tier)
            .bind(img_bytes)
            .execute(&pool)
            .await?;
            
            println!("Uploaded tier{}.png to database", tier);
        } else {
            println!("File tier{}.png not found, skipping.", tier);
        }
    }

    println!("Done seeding borders!");
    Ok(())
}
