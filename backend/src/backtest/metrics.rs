use crate::backtest::types::*;

pub fn compute_metrics(trades: &[Trade]) -> (f64, f64) {
    let total = trades.len() as f64;
    if total == 0.0 {
        return (0.0, 0.0);
    }

    let wins = trades.iter().filter(|t| t.profit > 0.0).count() as f64;
    let win_rate = wins / total * 100.0;

    let total_profit: f64 = trades.iter().map(|t| t.profit).sum();

    (win_rate, total_profit)
}