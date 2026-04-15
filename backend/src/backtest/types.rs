use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Serialize)]
pub struct Trade {
    pub entry_price: f64,
    pub exit_price: f64,
    pub profit: f64,
    pub side: Side,
    pub entry_time: u64,
    pub exit_time: u64,
}

#[derive(Debug, Clone)]
pub struct BacktestResult {
    pub total_trades: usize,
    pub win_rate: f64,
    pub total_profit: f64,
    pub max_drawdown: f64,
    pub trades: Vec<Trade>,
}