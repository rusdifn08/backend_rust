use dotenvy::dotenv;
use sqlx::PgPool;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env");

    let pool = PgPool::connect(&database_url).await?;

    // Clear existing (optional, but good for resetting)
    sqlx::query("DELETE FROM shop_items").execute(&pool).await?;

    let items = vec![
        // Consumables
        (
            "Freeze Ticket",
            "Protects your streak for 1 day.",
            50,
            "Consumable",
            None,
            None,
        ),
        (
            "Double EXP Potion",
            "Double EXP gain for 24 hours.",
            150,
            "Consumable",
            None,
            None,
        ),
        // Emblems
        (
            "Master Emblem",
            "Show off your mastery.",
            500,
            "Emblem",
            None,
            Some(1),
        ),
        (
            "Dragon Emblem",
            "The ultimate symbol of power.",
            1000,
            "Emblem",
            None,
            Some(1),
        ),
        (
            "Phoenix Emblem",
            "Rise from the ashes.",
            1500,
            "Emblem",
            None,
            Some(1),
        ),
        // Pets
        (
            "Pixel Dog",
            "A loyal companion on your productivity journey.",
            1000,
            "Pet",
            Some("https://rust-labs.onrender.com/api/assets/store-dog.gif"),
            Some(1),
        ),
        (
            "Cyber Cat",
            "Meow. Bleep. Bloop.",
            2000,
            "Pet",
            Some("https://rust-labs.onrender.com/api/assets/store-cat.gif"),
            Some(1),
        ),
        (
            "Golden Penguin",
            "Legendary pet that watches over your habits.",
            5000,
            "Pet",
            Some("https://rust-labs.onrender.com/api/assets/store-penguin.gif"),
            Some(1),
        ),
        // Themes
        (
            "Dark Mode Pro",
            "Ultra dark aesthetic for the night owls.",
            1000,
            "Theme",
            None,
            Some(1),
        ),
        (
            "Sakura Pink",
            "A beautiful, serene pink aesthetic.",
            1200,
            "Theme",
            None,
            Some(1),
        ),
        (
            "Neon Cyberpunk",
            "Vibrant neon colors on a dark background.",
            1500,
            "Theme",
            None,
            Some(1),
        ),
        (
            "Ocean Breeze",
            "Calming blue and teal gradients.",
            1200,
            "Theme",
            None,
            Some(1),
        ),
        // UI Custom
        (
            "Retro Font",
            "Changes your app font to a retro 8-bit style.",
            800,
            "UICustom",
            None,
            Some(1),
        ),
        (
            "Glassmorphism Pro",
            "Enhanced glass effects for all cards.",
            1500,
            "UICustom",
            None,
            Some(1),
        ),
    ];

    for (name, desc, price, category, url, max_p) in items {
        sqlx::query(
            "INSERT INTO shop_items (name, description, price, category, image_url, max_purchases) VALUES ($1, $2, $3, $4::item_category, $5, $6)"
        )
        .bind(name)
        .bind(desc)
        .bind(price)
        .bind(category)
        .bind(url)
        .bind(max_p)
        .execute(&pool)
        .await?;

        println!("Inserted {}", name);
    }

    println!("Shop successfully seeded!");

    Ok(())
}
