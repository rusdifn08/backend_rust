use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use tracing::{info, error};
use crate::models::user::User;
use crate::utils::jwt::create_jwt;

#[derive(Deserialize, Debug)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Deserialize, Debug)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: User,
}

pub async fn register(
    State(pool): State<PgPool>,
    Json(payload): Json<RegisterRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    info!("API CALL: [POST] /api/auth/register - Attempting to register user: {}", payload.email);
    
    let salt = SaltString::generate(&mut OsRng);
    
    // Use optimized params for fast hashing
    let params = argon2::Params::new(4096, 2, 1, None).unwrap();
    let argon2 = Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params);
    
    let password_hash = argon2.hash_password(payload.password.as_bytes(), &salt)
        .map_err(|e| {
            error!("Failed to hash password for {}: {}", payload.email, e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?
        .to_string();

    let friend_code = {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        format!("{:06}", rng.gen_range(100000..999999))
    };

    let user = sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (username, email, password_hash, friend_code)
        VALUES ($1, $2, $3, $4)
        RETURNING *
        "#
    )
    .bind(&payload.username)
    .bind(&payload.email)
    .bind(password_hash)
    .bind(friend_code)
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        if e.to_string().contains("duplicate key") {
            error!("Registration failed: Email or username already exists for {}", payload.email);
            (StatusCode::CONFLICT, "Email or username already exists".to_string())
        } else {
            error!("Database error during registration: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        }
    })?;

    let token = create_jwt(&user.id.to_string()).unwrap();
    info!("✅ Registration successful for: {}", payload.email);

    Ok((StatusCode::CREATED, Json(AuthResponse { token, user })))
}

pub async fn login(
    State(pool): State<PgPool>,
    Json(payload): Json<LoginRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    info!("API CALL: [POST] /api/auth/login - Login attempt for: {}", payload.username);

    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = $1")
        .bind(&payload.username)
        .fetch_optional(&pool)
        .await
        .map_err(|e| {
            error!("Database error during login query: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?;

    if let Some(user) = user {
        let parsed_hash = PasswordHash::new(&user.password_hash)
            .map_err(|e| {
                error!("Hash parsing error for user {}: {}", payload.username, e);
                (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
            })?;
            
        // Use optimized params for fast verification
        let params = argon2::Params::new(4096, 2, 1, None).unwrap();
        let argon2_verifier = Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params);

        if argon2_verifier.verify_password(payload.password.as_bytes(), &parsed_hash).is_ok() {
            let token = create_jwt(&user.id.to_string()).unwrap();
            info!("✅ Login successful for: {}", payload.username);
            return Ok((StatusCode::OK, Json(AuthResponse { token, user })));
        } else {
            error!("❌ Login failed: Incorrect password for {}", payload.username);
        }
    } else {
        error!("❌ Login failed: User not found for {}", payload.username);
    }

    Err((StatusCode::UNAUTHORIZED, "Invalid username or password".to_string()))
}

#[derive(Deserialize, Debug)]
pub struct UpdateProfileRequest {
    pub username: String,
    pub avatar_url: Option<String>,
}

pub async fn update_profile(
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
    State(pool): State<PgPool>,
    Json(payload): Json<UpdateProfileRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    
    // Delete old avatar if it's changing and was locally uploaded
    if let Ok(Some(old_user)) = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1").bind(id).fetch_optional(&pool).await {
        if let Some(old_url) = old_user.avatar_url {
            if let Some(new_url) = &payload.avatar_url {
                if &old_url != new_url && old_url.contains("/api/files/") {
                    if let Some(file_id_str) = old_url.split("/api/files/").last() {
                        if let Ok(file_uuid) = uuid::Uuid::parse_str(file_id_str) {
                            let _ = sqlx::query("DELETE FROM file_uploads WHERE id = $1").bind(file_uuid).execute(&pool).await;
                        }
                    }
                }
            }
        }
    }

    let user = sqlx::query_as::<_, User>(
        r#"
        UPDATE users
        SET username = $1, avatar_url = $2
        WHERE id = $3
        RETURNING *
        "#
    )
    .bind(&payload.username)
    .bind(&payload.avatar_url)
    .bind(id)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok((StatusCode::OK, Json(user)))
}
