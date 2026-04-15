// src/routes/market.rs
use axum::{Json, extract::Query, routing::get, Router};
use crate::models::candle::Candle;
use crate::services::data_service::get_historical_candles;
use crate::AppState;
use crate::MAX_CANDLES;
use crate::models::indicators::IndicatorParams;
use serde::Deserialize;
use std::collections::VecDeque;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct MarketQuery {
    pub symbol: Option<String>,
    pub interval: Option<String>,
}

#[derive(Deserialize)]
pub struct IndicatorQuery {
    pub symbol: Option<String>,
    pub interval: Option<String>,
    pub rsi_period: Option<usize>,
    pub macd_fast: Option<usize>,
    pub macd_slow: Option<usize>,
    pub bb_period: Option<usize>,
    pub bb_std: Option<f64>,
}

pub async fn get_candles(Query(params): Query<MarketQuery>, state: AppState) -> Json<Vec<Candle>> {
    let symbol = params.symbol.unwrap_or_else(|| "btcusdt".to_string()).to_lowercase();
    let interval = params.interval.unwrap_or_else(|| "1m".to_string());
    
    // Check rate limit per endpoint
    let client_ip = "default"; // Would be extracted from request in production
    
    // Check cache first (include interval in cache key)
    let cache_key = format!("{}:{}", symbol, interval);
    if let Some(candles) = state.candles_cache.get(&cache_key) {
        return Json(candles.iter().cloned().collect());
    }
    
    // Fetch from API and cache
    match get_historical_candles(&symbol, &interval).await {
        Ok(candles_vec) => {
            let mut deque: VecDeque<Candle> = candles_vec.into_iter().collect();
            // Limit to MAX_CANDLES
            while deque.len() > MAX_CANDLES {
                deque.pop_front();
            }
            let result: Vec<Candle> = deque.iter().cloned().collect();
            state.candles_cache.insert(cache_key, deque);
            Json(result)
        }
        Err(_) => Json(vec![]),
    }
}

pub async fn get_candles_rate_limited(
    Query(params): Query<MarketQuery>,
    state: AppState,
    axum::extract::ConnectInfo(addr): axum::extract::ConnectInfo<std::net::SocketAddr>,
) -> Result<Json<Vec<Candle>>, axum::http::StatusCode> {
    let ip = addr.ip().to_string();
    
    // Check rate limit: 1000 req/sec for candles endpoint
    if !state.candles_rate_limiter.check_ip(&ip) {
        tracing::warn!("Rate limit exceeded for {} on /api/candles", ip);
        return Err(axum::http::StatusCode::TOO_MANY_REQUESTS);
    }
    
    let symbol = params.symbol.unwrap_or_else(|| "btcusdt".to_string()).to_lowercase();
    let interval = params.interval.unwrap_or_else(|| "1m".to_string());
    
    // Check cache first (include interval in cache key)
    let cache_key = format!("{}:{}", symbol, interval);
    if let Some(candles) = state.candles_cache.get(&cache_key) {
        return Ok(Json(candles.iter().cloned().collect()));
    }
    
    match get_historical_candles(&symbol, &interval).await {
        Ok(candles_vec) => {
            let mut deque: VecDeque<Candle> = candles_vec.into_iter().collect();
            while deque.len() > MAX_CANDLES {
                deque.pop_front();
            }
            let result: Vec<Candle> = deque.iter().cloned().collect();
            state.candles_cache.insert(cache_key, deque);
            Ok(Json(result))
        }
        Err(_) => Ok(Json(vec![])),
    }
}

pub async fn get_candles_custom_indicators(
    Query(params): Query<IndicatorQuery>,
    state: AppState,
) -> Result<Json<Vec<Candle>>, axum::http::StatusCode> {
    let symbol = params.symbol.unwrap_or_else(|| "btcusdt".to_string()).to_lowercase();
    let interval = params.interval.unwrap_or_else(|| "1m".to_string());
    
    let mut params = IndicatorParams::default();
    if let Some(rsi) = params.rsi_period { params.rsi_period = rsi; }
    if let Some(fast) = params.macd_fast { params.macd_fast = fast; }
    if let Some(slow) = params.macd_slow { params.macd_slow = slow; }
    if let Some(period) = params.bb_period { params.bb_period = period; }
    if let Some(std) = params.bb_std { params.bb_std = std; }
    
    let cache_key = format!("{}:{}:custom", symbol, interval);
    
    if let Some(candles) = state.candles_cache.get(&cache_key) {
        return Ok(Json(candles.iter().cloned().collect()));
    }
    
    match get_historical_candles(&symbol, &interval).await {
        Ok(mut candles_vec) => {
            crate::models::indicators::calculate_indicators_with_params(&mut candles_vec, &params);
            let mut deque: VecDeque<Candle> = candles_vec.into_iter().collect();
            while deque.len() > MAX_CANDLES {
                deque.pop_front();
            }
            let result: Vec<Candle> = deque.iter().cloned().collect();
            state.candles_cache.insert(cache_key, deque);
            Ok(Json(result))
        }
        Err(_) => Ok(Json(vec![])),
    }
}