// src/main.rs
use axum::{
    routing::{get, post},
    Router,
    extract::ws::WebSocketUpgrade,
    response::IntoResponse,
};
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use std::sync::Arc;
use dashmap::DashMap;
use std::collections::VecDeque;
use parking_lot::RwLock;

mod models;
mod routes;
mod ws;
mod services;
mod channels;
mod trading;

use routes::market::get_candles;
use routes::trading::{TradingState, create_strategy_simple, list_strategies as list_strats, get_signals};
use ws::handler::handle_socket;
use channels::{LockFreeChannel, MarketData};
use trading::SignalGenerator;

const MAX_CANDLES: usize = 500;
const QUEUE_CAPACITY: usize = 10000;

#[derive(Clone)]
pub struct AppState {
    pub candles_cache: Arc<DashMap<String, VecDeque<crate::models::candle::Candle>>>,
    pub market_channel: Arc<LockFreeChannel>,
}

#[tokio::main]
async fn main() {
    // Create lock-free market data channel
    let market_channel = Arc::new(LockFreeChannel::new(QUEUE_CAPACITY));

    // Create app state with DashMap
    let state = AppState {
        candles_cache: Arc::new(DashMap::new()),
        market_channel: Arc::clone(&market_channel),
    };

    // Create trading state
    let trading_state = TradingState::new();

    // Start Binance WebSocket listener for multiple symbols
    let state_clone = state.clone();
    tokio::spawn(async move {
        ws::binance_listener::start_binance_listener(state_clone).await;
    });

    let app = Router::new()
        .route("/api/candles", get({
            let state = state.clone();
            move |query| get_candles(query, state)
        }))
        .route("/api/trading/strategies", axum::routing::post({
            let ts = trading_state.clone();
            move |body| create_strategy_simple(axum::extract::State(ts), body)
        }))
        .route("/api/trading/strategies/list", get({
            let ts = trading_state.clone();
            move || list_strats(axum::extract::State(ts))
        }))
        .route("/api/trading/signals", get({
            let ts = trading_state.clone();
            move || get_signals(axum::extract::State(ts))
        }))
        .route("/ws", get({
            let state = state.clone();
            move |ws: WebSocketUpgrade, query| async move { ws_handler(ws, query, state).await }
        }))
        .layer(CorsLayer::permissive());

    println!("Server running at http://localhost:3000 [HFT Mode - Lock-Free]");

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
    state: AppState
) -> impl IntoResponse {
    let symbol = params.get("symbol").cloned().unwrap_or_else(|| "btcusdt".to_string());
    ws.on_upgrade(move |socket| handle_socket(socket, symbol, state))
}