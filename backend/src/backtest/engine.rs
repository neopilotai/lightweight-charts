use crate::indicator::engine::IndicatorEngine;
use crate::backtest::executor::Executor;
use crate::backtest::strategy::check_signal;
use crate::backtest::metrics::compute_metrics;
use crate::backtest::types::*;
use crate::models::candle::Candle;

pub fn run_backtest(
    candles: &[Candle],
    mut indicator: IndicatorEngine,
    buy_condition: &str,
    sell_condition: &str,
) -> BacktestResult {
    let mut executor = Executor::new(1000.0);

    for candle in candles {
        let mut values = indicator.update(candle);

        // include price
        values.insert("close".into(), candle.close);

        if check_signal(buy_condition, &values) {
            executor.on_signal("BUY", candle.close, candle.time);
        }

        if check_signal(sell_condition, &values) {
            executor.on_signal("SELL", candle.close, candle.time);
        }
    }

    let (win_rate, total_profit) = compute_metrics(&executor.trades);

    BacktestResult {
        total_trades: executor.trades.len(),
        win_rate,
        total_profit,
        max_drawdown: 0.0, // upgrade later
        trades: executor.trades,
    }
}