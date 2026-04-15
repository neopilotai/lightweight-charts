// src/routes/auth.rs
use axum::{
    routing::post,
    Router,
    Json,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub expires_in: u64,
}

#[derive(Debug, Serialize)]
pub struct AuthError {
    pub error: String,
}

pub async fn login(Json(body): Json<LoginRequest>) -> Json<Result<LoginResponse, AuthError>> {
    // Simple authentication - in production, verify against database
    // For demo: accept any username with password "admin"
    if body.password == "admin" {
        match crate::auth::create_token(&body.username) {
            Ok(token) => Json(Ok(LoginResponse {
                token,
                expires_in: 86400,
            })),
            Err(e) => Json(Err(AuthError {
                error: format!("Failed to create token: {}", e),
            })),
        }
    } else {
        Json(Err(AuthError {
            error: "Invalid credentials".to_string(),
        }))
    }
}

pub fn create_auth_router() -> Router {
    Router::new()
        .route("/auth/token", post(login))
}