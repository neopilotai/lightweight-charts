// src/main.rs
use axum::{
    routing::get,
    Router,
    extract::ws::WebSocketUpgrade,
    response::IntoResponse,
};
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use std::sync::Arc;
use dashmap::DashMap;
use std::collections::VecDeque;

mod models;
mod routes;
mod ws;
mod services;
mod trading;
mod channels;
mod metrics;
mod middleware;
mod auth;

use routes::market::get_candles;
use routes::trading::{TradingState, create_strategy_simple, list_strategies as list_strats, get_signals};
use routes::health::{create_health_router, HealthState};
use routes::auth::create_auth_router;
use ws::handler::handle_socket;
use channels::MarketData;
use tokio::signal;

const MAX_CANDLES: usize = 500;

#[derive(Clone)]
pub struct AppState {
    pub candles_cache: Arc<DashMap<String, VecDeque<crate::models::candle::Candle>>>,
    pub client_senders: Arc<tokio::sync::Mutex<Vec<tokio::sync::mpsc::UnboundedSender<Arc<MarketData>>>>>,
    pub sequence_tracker: Arc<parking_lot::Mutex<DashMap<String, u64>>>,
    pub health_state: crate::routes::health::HealthState,
    pub shutdown_signal: Arc<parking_lot::Mutex<bool>>,
}

#[tokio::main]
async fn main() {
    // Initialize tracing for structured logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into())
        )
        .init();

    tracing::info!("Starting lightweight-charts-backend v{}", env!("CARGO_PKG_VERSION"));

    // Initialize Prometheus metrics
    if let Err(e) = metrics::init_metrics() {
        tracing::error!("Failed to initialize metrics: {}", e);
    }
    tracing::info!("Metrics initialized");

    // Create health state
    let health_state = HealthState::new();
    let health_state_clone = health_state.clone();

    // Create app state with DashMap and sequence tracker
    let state = AppState {
        candles_cache: Arc::new(DashMap::new()),
        client_senders: Arc::new(tokio::sync::Mutex::new(Vec::new())),
        sequence_tracker: Arc::new(parking_lot::Mutex::new(DashMap::new())),
        health_state: health_state_clone,
        shutdown_signal: Arc::new(parking_lot::Mutex::new(false)),
    };

    // Create trading state
    let trading_state = TradingState::new();

    // Start Binance WebSocket listener for multiple symbols
    let state_clone = state.clone();
    tokio::spawn(async move {
        ws::binance_listener::start_binance_listener(state_clone).await;
    });

    let app = Router::new()
        .merge(create_health_router(health_state))
        .merge(create_auth_router())
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

    tracing::info!("Server running at http://localhost:3000");

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    
    // Set up signal handlers for graceful shutdown
    let shutdown = state.shutdown_signal.clone();
    tokio::spawn(async move {
        let sig = signal::ctrl_c().await;
        if sig.is_ok() {
            tracing::info!("Received Ctrl-C, shutting down gracefully...");
            *shutdown.lock() = true;
        }
    });
    
    let shutdown_check = state.shutdown_signal.clone();
    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            // Wait for shutdown signal
            while !*shutdown_check.lock() {
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
            tracing::info!("Shutting down server...");
        })
        .await
        .unwrap();
    
    tracing::info!("Server shutdown complete");
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
    state: AppState
) -> impl IntoResponse {
    let symbol = params.get("symbol").cloned().unwrap_or_else(|| "btcusdt".to_string());
    ws.on_upgrade(move |socket| handle_socket(socket, symbol, state))
}