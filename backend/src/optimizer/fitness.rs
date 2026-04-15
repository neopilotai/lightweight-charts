use crate::backtest::engine::run_backtest;
use crate::backtest::types::BacktestResult;
use crate::indicator::engine::IndicatorEngine;
use crate::models::candle::Candle;
use crate::optimizer::genome::{genome_to_strategy, Genome};

pub fn fitness(
    genome: &Genome,
    candles: &[Candle],
    indicator: IndicatorEngine,
) -> (f64, BacktestResult) {
    let (buy, sell) = genome_to_strategy(genome);
    let result = run_backtest(candles, indicator, &buy, &sell);
    let score = score(&result);
    (score, result)
}

pub fn score(result: &BacktestResult) -> f64 {
    result.total_profit * 0.6 + result.win_rate * 0.3 - result.max_drawdown * 0.2
}
