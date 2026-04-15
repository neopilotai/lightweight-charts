// src/routes/trading.rs
use axum::{
    extract::{Query, State, Json},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::Mutex;

use crate::AppState;
use crate::trading::{
    StrategyConfig, StrategyManager, StrategyType,
};
use crate::models::orders::Signal;

#[derive(Clone)]
pub struct TradingState {
    pub strategy_manager: Arc<Mutex<StrategyManager>>,
    pub signals: Arc<Mutex<Vec<Signal>>>,
}

impl TradingState {
    pub fn new() -> Self {
        TradingState {
            strategy_manager: Arc::new(Mutex::new(StrategyManager::new())),
            signals: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

// Request/Response types
#[derive(Debug, Deserialize)]
pub struct CreateStrategyRequest {
    pub name: String,
    pub strategy_type: String,
    pub symbol: String,
    pub risk_percent: Option<f64>,
    pub stop_loss_pct: Option<f64>,
    pub take_profit_pct: Option<f64>,
    pub max_positions: Option<usize>,
}

impl CreateStrategyRequest {
    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() || self.name.len() > 100 {
            return Err("name must be between 1 and 100 characters".to_string());
        }
        
        if let Some(risk) = self.risk_percent {
            if risk < 0.1 || risk > 100.0 {
                return Err("risk_percent must be between 0.1 and 100.0".to_string());
            }
        }
        
        if self.symbol.is_empty() || self.symbol.len() > 20 {
            return Err("symbol must be between 1 and 20 characters".to_string());
        }
        
        if !self.symbol.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit()) {
            return Err("symbol must contain only uppercase letters and digits".to_string());
        }
        
        if let Some(max_pos) = self.max_positions {
            if max_pos < 1 || max_pos > 10 {
                return Err("max_positions must be between 1 and 10".to_string());
            }
        }
        
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateStrategyRequest {
    pub risk_percent: Option<f64>,
    pub stop_loss_pct: Option<f64>,
    pub take_profit_pct: Option<f64>,
    pub max_positions: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct StrategyResponse {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub config: StrategyConfig,
}

#[derive(Debug, Deserialize)]
pub struct BacktestRequest {
    pub strategy_id: String,
    pub symbol: String,
    pub initial_balance: f64,
}

#[derive(Debug, Serialize)]
pub struct BacktestResponse {
    pub total_trades: usize,
    pub winning_trades: usize,
    pub losing_trades: usize,
    pub win_rate: f64,
    pub total_pnl: f64,
    pub total_return_pct: f64,
    pub avg_pnl: f64,
    pub max_pnl: f64,
    pub min_pnl: f64,
    pub max_drawdown: f64,
    pub final_balance: f64,
}

#[derive(Debug, Serialize)]
pub struct SignalResponse {
    pub symbol: String,
    pub signal_type: String,
    pub confidence: f64,
    pub timestamp: i64,
}

// Handlers
pub async fn create_strategy(
    State(trading_state): State<TradingState>,
    Json(req): Json<CreateStrategyRequest>,
) -> Json<StrategyResponse> {
    let strategy_type = match req.strategy_type.as_str() {
        "moving_average_crossover" => StrategyType::MovingAverageCrossover,
        "rsi_momentum" => StrategyType::RSIMomentum,
        "macd_crossover" => StrategyType::MACDCrossover,
        "multi_indicator" => StrategyType::MultiIndicator,
        _ => StrategyType::Custom(req.strategy_type.clone()),
    };

    let mut config = StrategyConfig::new(req.name.clone(), strategy_type, req.symbol.clone());

    if let Some(risk) = req.risk_percent {
        config.risk_percent = risk;
    }
    if let Some(stop_loss) = req.stop_loss_pct {
        config.stop_loss_pct = stop_loss;
    }
    if let Some(take_profit) = req.take_profit_pct {
        config.take_profit_pct = take_profit;
    }
    if let Some(max_pos) = req.max_positions {
        config.max_positions = max_pos;
    }

    let id = config.id.clone();
    trading_state
        .strategy_manager
        .lock()
        .add_strategy(config.clone());

    Json(StrategyResponse {
        id,
        name: req.name,
        enabled: true,
        config,
    })
}

pub async fn list_strategies(
    State(trading_state): State<TradingState>,
) -> Json<Vec<StrategyConfig>> {
    let strategies = trading_state
        .strategy_manager
        .lock()
        .list_strategies();
    Json(strategies)
}

pub async fn get_strategy(
    State(trading_state): State<TradingState>,
    Query(params): Query<HashMap<String, String>>,
) -> Json<Option<StrategyConfig>> {
    if let Some(id) = params.get("id") {
        if let Some(strategy) = trading_state.strategy_manager.lock().get_strategy(id) {
            return Json(Some(strategy.config.clone()));
        }
    }
    Json(None)
}

pub async fn enable_strategy(
    State(trading_state): State<TradingState>,
    Query(params): Query<HashMap<String, String>>,
) -> Json<serde_json::Value> {
    if let Some(id) = params.get("id") {
        trading_state
            .strategy_manager
            .lock()
            .enable_strategy(id);
        return Json(serde_json::json!({"status": "enabled"}));
    }
    Json(serde_json::json!({"error": "No ID provided"}))
}

pub async fn disable_strategy(
    State(trading_state): State<TradingState>,
    Query(params): Query<HashMap<String, String>>,
) -> Json<serde_json::Value> {
    if let Some(id) = params.get("id") {
        trading_state
            .strategy_manager
            .lock()
            .disable_strategy(id);
        return Json(serde_json::json!({"status": "disabled"}));
    }
    Json(serde_json::json!({"error": "No ID provided"}))
}

pub async fn delete_strategy(
    State(trading_state): State<TradingState>,
    Query(params): Query<HashMap<String, String>>,
) -> Json<serde_json::Value> {
    if let Some(id) = params.get("id") {
        trading_state
            .strategy_manager
            .lock()
            .remove_strategy(id);
        return Json(serde_json::json!({"status": "deleted"}));
    }
    Json(serde_json::json!({"error": "No ID provided"}))
}

pub async fn get_strategy_stats(
    State(trading_state): State<TradingState>,
) -> Json<Vec<(String, usize, usize, f64, f64)>> {
    let stats = trading_state
        .strategy_manager
        .lock()
        .get_all_stats();
    Json(stats)
}

pub async fn run_backtest(
    State(_trading_state): State<TradingState>,
    State(_app_state): State<AppState>,
    Json(_req): Json<BacktestRequest>,
) -> Json<BacktestResponse> {
    // Placeholder - in production would use actual candle data
    let result = BacktestResponse {
        total_trades: 0,
        winning_trades: 0,
        losing_trades: 0,
        win_rate: 0.0,
        total_pnl: 0.0,
        total_return_pct: 0.0,
        avg_pnl: 0.0,
        max_pnl: 0.0,
        min_pnl: 0.0,
        max_drawdown: 0.0,
        final_balance: 0.0,
    };
    Json(result)
}

pub async fn get_signals(
    State(trading_state): State<TradingState>,
) -> Json<Vec<Signal>> {
    let signals = trading_state.signals.lock().clone();
    Json(signals)
}

pub fn create_router(trading_state: TradingState) -> Router<TradingState> {
    Router::new()
        .route("/strategies", post(create_strategy).get(list_strategies))
        .route("/strategies/get", get(get_strategy))
        .route("/strategies/enable", post(enable_strategy))
        .route("/strategies/disable", post(disable_strategy))
        .route("/strategies/delete", post(delete_strategy))
        .route("/strategies/stats", get(get_strategy_stats))
        .route("/signals", get(get_signals))
        .with_state(trading_state)
}

// Simpler version that works with current AppState
// Simpler version that works with current AppState
pub async fn create_strategy_simple(
    State(trading_state): State<TradingState>,
    req: axum::extract::Json<CreateStrategyRequest>,
) -> Json<StrategyResponse> {    
    // Validate request
    if let Err(e) = req.0.validate() {
        panic!("Invalid request: {}", e);
    }
    
    let strategy_type = match req.0.strategy_type.as_str() {
        "moving_average_crossover" => StrategyType::MovingAverageCrossover,
        "rsi_momentum" => StrategyType::RSIMomentum,
        "macd_crossover" => StrategyType::MACDCrossover,
        "multi_indicator" => StrategyType::MultiIndicator,
        _ => StrategyType::Custom(req.0.strategy_type.clone()),
    };

    let mut config = StrategyConfig::new(req.0.name.clone(), strategy_type, req.0.symbol.clone());

    if let Some(risk) = req.0.risk_percent {
        config.risk_percent = risk;
    }
    if let Some(stop_loss) = req.0.stop_loss_pct {
        config.stop_loss_pct = stop_loss;
    }
    if let Some(take_profit) = req.0.take_profit_pct {
        config.take_profit_pct = take_profit;
    }
    if let Some(max_pos) = req.0.max_positions {
        config.max_positions = max_pos;
    }

    let id = config.id.clone();
    trading_state
        .strategy_manager
        .lock()
        .add_strategy(config.clone());

    Json(StrategyResponse {
        id,
        name: req.0.name,
        enabled: true,
        config,
    })
}
