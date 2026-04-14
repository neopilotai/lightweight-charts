// src/trading/backtest.rs
use crate::models::candle::Candle;
use crate::models::orders::{OrderSide, SignalType};
use crate::trading::engine::PortfolioStats;
use crate::trading::signals::SignalGenerator;
use crate::trading::strategy::{Strategy, StrategyConfig, StrategyType};
use std::collections::VecDeque;

/// Backtesting engine
pub struct BacktestEngine {
    pub initial_balance: f64,
    pub current_balance: f64,
    pub trades: Vec<BacktestTrade>,
    pub stats: Option<PortfolioStats>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct BacktestTrade {
    pub entry_time: u64,
    pub exit_time: u64,
    pub entry_price: f64,
    pub exit_price: f64,
    pub quantity: f64,
    pub side: OrderSide,
    pub pnl: f64,
    pub pnl_pct: f64,
}

impl BacktestEngine {
    pub fn new(initial_balance: f64) -> Self {
        BacktestEngine {
            initial_balance,
            current_balance: initial_balance,
            trades: Vec::new(),
            stats: None,
        }
    }

    /// Run backtest on historical data
    pub fn backtest(
        &mut self,
        strategy_config: StrategyConfig,
        candles: &VecDeque<Candle>,
    ) -> BacktestResult {
        let mut strategy = Strategy::new(strategy_config.clone());
        let mut open_position: Option<(u64, f64, f64)> = None; // (entry_time, entry_price, qty)

        // Process each candle
        for (i, candle) in candles.iter().enumerate() {
            // Generate signal
            let signal = if i >= 26 {
                // Need 26 candles for MACD
                let recent_candles: VecDeque<_> = candles
                    .iter()
                    .skip(i.saturating_sub(100))
                    .take(i - (i.saturating_sub(100)) + 1)
                    .cloned()
                    .collect();
                SignalGenerator::generate_signal(
                    &strategy_config.symbol,
                    &recent_candles,
                    candle.close,
                )
            } else {
                None
            };

            // Execute strategy
            let orders = strategy.update(
                signal.as_ref().map(|s| s.signal_type.clone()),
                candle.close,
                self.current_balance,
            );

            // Process orders and positions
            for order in orders {
                match order.side {
                    OrderSide::Buy => {
                        if open_position.is_none() {
                            let qty = strategy
                                .calculate_position_size(self.current_balance, candle.close);
                            let cost = qty * candle.close;
                            let fee = cost * 0.001; // 0.1% fee

                            if self.current_balance >= (cost + fee) {
                                self.current_balance -= cost + fee;
                                open_position = Some((candle.time, candle.close, qty));
                            }
                        }
                    }
                    OrderSide::Sell => {
                        if let Some((entry_time, entry_price, qty)) = open_position.take() {
                            let revenue = qty * candle.close;
                            let fee = revenue * 0.001; // 0.1% fee
                            let net_revenue = revenue - fee;

                            let pnl = net_revenue - (entry_price * qty);
                            let pnl_pct = (pnl / (entry_price * qty)) * 100.0;

                            self.current_balance += net_revenue;
                            self.trades.push(BacktestTrade {
                                entry_time,
                                exit_time: candle.time,
                                entry_price,
                                exit_price: candle.close,
                                quantity: qty,
                                side: OrderSide::Buy,
                                pnl,
                                pnl_pct,
                            });

                            strategy.record_trade(entry_price, candle.close, qty);
                        }
                    }
                }
            }

            // Check stop-loss/take-profit
            if let Some((entry_time, entry_price, qty)) = open_position {
                let current_pnl_pct = ((candle.close - entry_price) / entry_price) * 100.0;

                if current_pnl_pct <= -strategy_config.stop_loss_pct {
                    let pnl = (candle.close - entry_price) * qty;
                    self.current_balance += qty * candle.close - (qty * candle.close * 0.001);
                    self.trades.push(BacktestTrade {
                        entry_time,
                        exit_time: candle.time,
                        entry_price,
                        exit_price: candle.close,
                        quantity: qty,
                        side: OrderSide::Buy,
                        pnl,
                        pnl_pct: current_pnl_pct,
                    });
                    open_position = None;
                } else if current_pnl_pct >= strategy_config.take_profit_pct {
                    let pnl = (candle.close - entry_price) * qty;
                    self.current_balance += qty * candle.close - (qty * candle.close * 0.001);
                    self.trades.push(BacktestTrade {
                        entry_time,
                        exit_time: candle.time,
                        entry_price,
                        exit_price: candle.close,
                        quantity: qty,
                        side: OrderSide::Buy,
                        pnl,
                        pnl_pct: current_pnl_pct,
                    });
                    open_position = None;
                }
            }
        }

        // Close remaining position
        if let Some((entry_time, entry_price, qty)) = open_position {
            if let Some(last_candle) = candles.back() {
                let pnl = (last_candle.close - entry_price) * qty;
                let pnl_pct = (pnl / (entry_price * qty)) * 100.0;

                self.current_balance += qty * last_candle.close - (qty * last_candle.close * 0.001);
                self.trades.push(BacktestTrade {
                    entry_time,
                    exit_time: last_candle.time,
                    entry_price,
                    exit_price: last_candle.close,
                    quantity: qty,
                    side: OrderSide::Buy,
                    pnl,
                    pnl_pct,
                });
            }
        }

        self.calculate_stats()
    }

    fn calculate_stats(&mut self) -> BacktestResult {
        let total_trades = self.trades.len();
        let winning_trades = self.trades.iter().filter(|t| t.pnl > 0.0).count();
        let losing_trades = total_trades - winning_trades;

        let total_pnl: f64 = self.trades.iter().map(|t| t.pnl).sum();
        let total_return_pct =
            ((self.current_balance - self.initial_balance) / self.initial_balance) * 100.0;

        let win_rate = if total_trades > 0 {
            (winning_trades as f64 / total_trades as f64) * 100.0
        } else {
            0.0
        };

        let avg_pnl = if total_trades > 0 {
            total_pnl / total_trades as f64
        } else {
            0.0
        };

        let max_pnl = self.trades.iter().map(|t| t.pnl).fold(0.0, f64::max);
        let min_pnl = self.trades.iter().map(|t| t.pnl).fold(0.0, f64::min);

        let max_drawdown = Self::calculate_max_drawdown(&self.trades);

        BacktestResult {
            total_trades,
            winning_trades,
            losing_trades,
            win_rate,
            total_pnl,
            total_return_pct,
            avg_pnl,
            max_pnl,
            min_pnl,
            max_drawdown,
            final_balance: self.current_balance,
        }
    }

    fn calculate_max_drawdown(trades: &[BacktestTrade]) -> f64 {
        if trades.is_empty() {
            return 0.0;
        }

        let mut peak = 0.0;
        let mut max_dd = 0.0;
        let mut cumulative = 0.0;

        for trade in trades {
            cumulative += trade.pnl;
            if cumulative > peak {
                peak = cumulative;
            }
            let drawdown = peak - cumulative;
            if drawdown > max_dd {
                max_dd = drawdown;
            }
        }

        max_dd
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct BacktestResult {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backtest_engine_creation() {
        let engine = BacktestEngine::new(10000.0);
        assert_eq!(engine.initial_balance, 10000.0);
        assert_eq!(engine.current_balance, 10000.0);
        assert_eq!(engine.trades.len(), 0);
    }
}
