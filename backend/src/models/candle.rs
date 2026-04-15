// src/models/candle.rs
use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct Candle {
    pub time: u64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub rsi: Option<f64>,
    pub ema12: Option<f64>,
    pub ema26: Option<f64>,
    pub macd: Option<f64>,
    pub signal: Option<f64>,
    pub histogram: Option<f64>,
}
