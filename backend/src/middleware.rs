// src/middleware.rs
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

#[derive(Clone)]
pub struct RateLimiter {
    requests: Arc<Mutex<HashMap<String, Vec<Instant>>>>,
    max_requests: usize,
    window_secs: u64,
}

impl RateLimiter {
    pub fn new(max_requests: usize, window_secs: u64) -> Self {
        Self {
            requests: Arc::new(Mutex::new(HashMap::new())),
            max_requests,
            window_secs,
        }
    }

    pub fn check(&self, key: &str) -> bool {
        let mut requests = self.requests.lock();
        let now = Instant::now();
        let window_start = now - std::time::Duration::from_secs(self.window_secs);

        let timestamps = requests.entry(key.to_string()).or_insert_with(Vec::new);

        timestamps.retain(|t| *t > window_start);

        if timestamps.len() >= self.max_requests {
            return false;
        }

        timestamps.push(now);
        true
    }

    pub fn check_ip(&self, ip: &str) -> bool {
        self.check(ip)
    }
}

pub fn rate_limit_response() -> Response {
    tracing::warn!("Rate limit exceeded");
    (
        StatusCode::TOO_MANY_REQUESTS,
        [("Retry-After", "1")],
        "Too many requests",
    )
        .into_response()
}
