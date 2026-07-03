mod api;
mod models;
mod utils;
mod repositories;
mod services;
use axum::{routing::{get, post, put, delete}, Router, response::IntoResponse};
use sqlx::postgres::PgPoolOptions;
use std::env;
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables
    dotenvy::dotenv().ok();
    
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("Starting Mobile Productivity Backend...");

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env");

    println!("Connecting to Supabase PostgreSQL...");
    
    // Set up connection pool
    let pool = PgPoolOptions::new()
        .max_connections(20)
        .min_connections(5)
        .idle_timeout(std::time::Duration::from_secs(3600))
        .connect(&database_url)
        .await?;

    println!("Running database migrations...");
    sqlx::migrate!("./migrations").run(&pool).await?;
    println!("✅ Migrations applied successfully!");

    println!("✅ Successfully connected to Supabase PostgreSQL!");

    // Setup AppState for Chat
    let chat_state: api::chat::ChatState = std::sync::Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new()));
    let app_state = api::chat::AppState {
        pool: pool.clone(),
        chat_state,
        analytics_state: api::analytics::AnalyticsState::new(),
    };

    // Setup CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Build our application with routes
    let app = Router::new()
        // Dashboard
        .route("/", get(|| async { axum::response::Html(include_str!("dashboard.html")) }))
        .route("/api/analytics/stream", get(api::analytics::stream_analytics))
        // Auth Routes
        .route("/api/auth/register", post(api::auth::register))
        .route("/api/auth/login", post(api::auth::login))
        .route("/api/auth/profile/:id", put(api::auth::update_profile))
        // Todo Routes
        .route("/api/todos/user/:user_id", get(api::todos::get_todos))
        .route("/api/todos", post(api::todos::create_todo))
        .route("/api/todos/:id", put(api::todos::toggle_todo).delete(api::todos::delete_todo))
        // Habit Routes
        .route("/api/habits/user/:user_id", get(api::habits::get_habits))
        .route("/api/habits", post(api::habits::create_habit))
        .route("/api/habits/:id", put(api::habits::toggle_habit).delete(api::habits::delete_habit))
        .route("/api/habits/:id/logs", get(api::habits::get_habit_logs))
        .route("/api/habits/logs", post(api::habits::add_habit_log))
        // Transaction Routes
        .route("/api/transactions/user/:user_id", get(api::transactions::get_transactions))
        .route("/api/transactions", post(api::transactions::create_transaction))
        .route("/api/transactions/:id", delete(api::transactions::delete_transaction))
        // Note Routes
        .route("/api/notes/user/:user_id", get(api::notes::get_notes))
        .route("/api/notes", post(api::notes::create_note))
        .route("/api/notes/:id", delete(api::notes::delete_note))
        // Focus Routes
        .route("/api/focus", get(api::focus::get_focus_sessions).post(api::focus::create_focus_session))
        // Chat Routes
        .route("/api/friends", post(api::chat::add_friend))
        .route("/api/friends/accept", post(api::chat::accept_friend))
        .route("/api/friends/:id", get(api::chat::get_friends))
        .route("/api/friends/search/:code", get(api::chat::search_friend))
        .route("/api/chat/history/:user1/:user2", get(api::chat::get_chat_history).delete(api::chat::delete_chat_history))
        .route("/api/chat/message/:id", delete(api::chat::delete_single_message))
        .route("/api/ws/chat/:user_id", get(api::chat::ws_handler))
        // Avatars Routes
        .route("/api/avatars", get(api::avatars::get_avatars))
        // Upload Routes
        .route("/api/upload", post(api::upload::upload_file))
        .route("/api/files/:id", get(api::upload::get_file))
        // Gamification Routes
        .route("/api/gamification/profile/:user_id", get(api::gamification::get_profile))
        .route("/api/gamification/shop/buy-freeze/:user_id", post(api::gamification::buy_freeze_ticket))
        .route("/api/gamification/border/:tier", get(api::gamification::get_border))
        // Social Routes
        .route("/api/social/feed", get(api::social::get_feed))
        .route("/api/social/leaderboard", get(api::social::get_leaderboard))
        .route("/api/social/activity", post(api::social::create_activity))
        // Weekly Planner Routes
        .route("/api/weekly/user/:user_id", get(api::weekly::get_weekly_tasks))
        .route("/api/weekly", post(api::weekly::create_weekly_task))
        .route("/api/weekly/:id", put(api::weekly::toggle_weekly_task).delete(api::weekly::delete_weekly_task))
        // Shop Routes
        .route("/api/shop/items", get(api::shop::get_shop_items))
        .route("/api/shop/inventory/:user_id", get(api::shop::get_inventory))
        .route("/api/shop/buy", post(api::shop::buy_item))
        .route("/api/shop/equip/:inventory_id", post(api::shop::equip_item))
        
        // System OTA
        .route("/api/system/ota/latest", get(api::system::get_latest_ota))
        // Serve static assets from src/Assets
        .route("/api/assets/:filename", get(|axum::extract::Path(filename): axum::extract::Path<String>| async move {
            let path = format!("src/Assets/{}", filename);
            match tokio::fs::read(&path).await {
                Ok(bytes) => {
                    let mut headers = axum::http::HeaderMap::new();
                    let ct = if filename.ends_with(".gif") { "image/gif" } else if filename.ends_with(".png") { "image/png" } else { "application/octet-stream" };
                    headers.insert(axum::http::header::CONTENT_TYPE, ct.parse().unwrap());
                    headers.insert(axum::http::header::CACHE_CONTROL, "public, max-age=86400".parse().unwrap());
                    (axum::http::StatusCode::OK, headers, bytes).into_response()
                },
                Err(_) => (axum::http::StatusCode::NOT_FOUND, "Asset not found").into_response(),
            }
        }))
        // KPI Analytics Route
        .route("/api/analytics/kpi/:user_id", get(api::analytics::get_kpi))
        .layer(cors)
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .layer(axum::middleware::from_fn(api::analytics::analytics_middleware))
        .layer(axum::extract::DefaultBodyLimit::max(20 * 1024 * 1024))
        .layer(axum::Extension(app_state.analytics_state.clone()))
        .with_state(app_state);

    // Fetch PORT from environment, default to 5050
    let port = env::var("PORT")
        .unwrap_or_else(|_| "5050".to_string())
        .parse::<u16>()
        .unwrap_or(5050);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    println!("🚀 Server listening on http://0.0.0.0:{}", port);
    
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
