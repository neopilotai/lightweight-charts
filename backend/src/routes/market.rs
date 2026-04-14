// src/routes/market.rs
use axum::{Json, extract::Query};
use crate::models::candle::Candle;
use crate::services::data_service::get_historical_candles;
use crate::AppState;
use crate::MAX_CANDLES;
use serde::Deserialize;
use std::collections::VecDeque;

#[derive(Deserialize)]
pub struct MarketQuery {
    pub symbol: Option<String>,
}

pub async fn get_candles(Query(params): Query<MarketQuery>, state: AppState) -> Json<Vec<Candle>> {
    let symbol = params.symbol.unwrap_or_else(|| "btcusdt".to_string()).to_lowercase();
    
    // Check cache first
    if let Some(candles) = state.candles_cache.get(&symbol) {
        return Json(candles.iter().cloned().collect());
    }
    
    // Fetch from API and cache
    match get_historical_candles(&symbol).await {
        Ok(mut candles_vec) => {
            let mut deque: VecDeque<Candle> = candles_vec.into_iter().collect();
            // Limit to MAX_CANDLES
            while deque.len() > MAX_CANDLES {
                deque.pop_front();
            }
            let result: Vec<Candle> = deque.iter().cloned().collect();
            state.candles_cache.insert(symbol, deque);
            Json(result)
        }
        Err(_) => Json(vec![]),
    }
}