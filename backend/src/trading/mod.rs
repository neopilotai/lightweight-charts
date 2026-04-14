// src/trading/mod.rs
pub mod backtest;
pub mod engine;
pub mod signals;
pub mod strategy;

pub use backtest::{BacktestEngine, BacktestResult, BacktestTrade};
pub use engine::{PortfolioStats, TradingEngine};
pub use signals::SignalGenerator;
pub use strategy::{Strategy, StrategyConfig, StrategyManager, StrategyType};
