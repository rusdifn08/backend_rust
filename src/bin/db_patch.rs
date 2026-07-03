use sqlx::postgres::PgPoolOptions;
use std::env;
use std::io::Cursor;
use image::{RgbaImage, Rgba};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    println!("Generating and inserting borders...");

    let colors = vec![
        (1, Rgba([205, 127, 50, 255])), // Bronze
        (2, Rgba([192, 192, 192, 255])), // Silver
        (3, Rgba([255, 215, 0, 255])),  // Gold
        (4, Rgba([138, 43, 226, 255])), // Diamond/Purple
    ];

    for (tier, color) in colors {
        let size = 200;
        let mut img = RgbaImage::new(size, size);
        
        let center = size as f32 / 2.0;
        let radius = (size as f32 / 2.0) - 10.0;
        let thickness = 10.0;

        for x in 0..size {
            for y in 0..size {
                let dx = x as f32 - center;
                let dy = y as f32 - center;
                let dist = (dx * dx + dy * dy).sqrt();

                if dist > radius - thickness && dist < radius + thickness {
                    img.put_pixel(x, y, color);
                } else {
                    img.put_pixel(x, y, Rgba([0, 0, 0, 0]));
                }
            }
        }

        let mut buf = Cursor::new(Vec::new());
        img.write_to(&mut buf, image::ImageFormat::Png)?;
        let img_bytes = buf.into_inner();

        sqlx::query(
            "INSERT INTO tier_borders (tier, image_data) VALUES ($1, $2) ON CONFLICT (tier) DO UPDATE SET image_data = EXCLUDED.image_data"
        )
        .bind(tier)
        .bind(img_bytes)
        .execute(&pool)
        .await?;
        
        println!("Inserted border for tier {}", tier);
    }

    println!("Done seeding borders!");
    Ok(())
}
