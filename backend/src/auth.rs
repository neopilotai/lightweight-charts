// src/auth.rs
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

const JWT_SECRET: &[u8] = b"trading_chart_secret_key_change_in_production";
const TOKEN_EXPIRATION_SECS: u64 = 86400; // 24 hours

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub user_id: String,
    pub exp: u64,
    pub iat: u64,
}

impl Claims {
    pub fn new(user_id: String) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            sub: user_id.clone(),
            user_id,
            exp: now + TOKEN_EXPIRATION_SECS,
            iat: now,
        }
    }

    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        now > self.exp
    }
}

pub fn create_token(user_id: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let claims = Claims::new(user_id.to_string());
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(JWT_SECRET),
    )
}

pub fn verify_token(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(JWT_SECRET),
        &Validation::default(),
    )?;

    if token_data.claims.is_expired() {
        return Err(jsonwebtoken::errors::Error::from(
            jsonwebtoken::errors::ErrorKind::InvalidToken,
        ));
    }

    Ok(token_data.claims)
}

pub fn extract_user_id(authorization: Option<axum::http::HeaderValue>) -> Option<String> {
    let auth = authorization?;
    let token_str = auth.to_str().ok()?;
    if token_str.starts_with("Bearer ") {
        let token = &token_str[7..];
        verify_token(token).ok().map(|c| c.user_id)
    } else {
        None
    }
}
