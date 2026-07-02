mod api;
mod models;
mod utils;

use axum::{routing::{get, post, put, delete}, Router};
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
        .route("/api/todos", get(api::todos::get_todos).post(api::todos::create_todo))
        .route("/api/todos/:id", put(api::todos::toggle_todo).delete(api::todos::delete_todo))
        // Habit Routes
        .route("/api/habits", get(api::habits::get_habits).post(api::habits::create_habit))
        .route("/api/habits/:id", put(api::habits::toggle_habit).delete(api::habits::delete_habit))
        // Transaction Routes
        .route("/api/transactions", get(api::transactions::get_transactions).post(api::transactions::create_transaction))
        .route("/api/transactions/:id", delete(api::transactions::delete_transaction))
        // Note Routes
        .route("/api/notes", get(api::notes::get_notes).post(api::notes::create_note))
        .route("/api/notes/:id", delete(api::notes::delete_note))
        // Focus Routes
        .route("/api/focus", get(api::focus::get_focus_sessions).post(api::focus::create_focus_session))
        // Chat Routes
        .route("/api/friends", post(api::chat::add_friend))
        .route("/api/friends/accept", post(api::chat::accept_friend))
        .route("/api/friends/:id", get(api::chat::get_friends))
        .route("/api/friends/search/:code", get(api::chat::search_friend))
        .route("/api/chat/history/:user1/:user2", get(api::chat::get_chat_history).delete(api::chat::delete_chat_history))
        .route("/api/ws/chat/:user_id", get(api::chat::ws_handler))
        // Avatars Routes
        .route("/api/avatars", get(api::avatars::get_avatars))
        // Upload Routes
        .route("/api/upload", post(api::upload::upload_file))
        .route("/api/files/:id", get(api::upload::get_file))
        .layer(cors)
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .layer(axum::middleware::from_fn(api::analytics::analytics_middleware))
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
