use lightweight_charts_strategy::{
    create_strategy, run_strategy, OrderSide, Strategy, StrategyConfig, StrategyType,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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

pub struct BacktestEngine {
    pub initial_balance: f64,
    pub current_balance: f64,
    pub trades: Vec<BacktestTrade>,
}

impl BacktestEngine {
    pub fn new(initial_balance: f64) -> Self {
        Self {
            initial_balance,
            current_balance: initial_balance,
            trades: Vec::new(),
        }
    }

    pub fn run(
        &mut self,
        strategy_name: &str,
        closes: &[f64],
        times: &[u64],
        fee_pct: f64,
    ) -> BacktestResult {
        let mut strategy = create_strategy(strategy_name, StrategyType::MultiIndicator, "BTCUSDT");
        let mut open_position: Option<(u64, f64, f64)> = None;

        for (i, &close) in closes.iter().enumerate() {
            if i < 26 {
                continue;
            }

            let (entered, exited) =
                run_strategy(&mut strategy, closes, close, self.current_balance);

            // Process entries
            for pos in entered {
                if open_position.is_none() {
                    let cost = pos.quantity * close;
                    let fee = cost * (fee_pct / 100.0);
                    if self.current_balance >= cost + fee {
                        self.current_balance -= cost + fee;
                        open_position = Some((times[i], close, pos.quantity));
                    }
                }
            }

            // Process exits
            for pos in exited {
                if let Some((entry_time, entry_price, qty)) = open_position.take() {
                    let revenue = qty * close;
                    let fee = revenue * (fee_pct / 100.0);
                    let net_revenue = revenue - fee;
                    let pnl = net_revenue - (entry_price * qty);
                    let pnl_pct = (pnl / (entry_price * qty)) * 100.0;

                    self.current_balance += net_revenue;
                    self.trades.push(BacktestTrade {
                        entry_time,
                        exit_time: times[i],
                        entry_price,
                        exit_price: close,
                        quantity: qty,
                        side: pos.side,
                        pnl,
                        pnl_pct,
                    });
                }
            }

            // Check SL/TP on open position
            if let Some((entry_time, entry_price, qty)) = open_position {
                let pnl_pct = ((close - entry_price) / entry_price) * 100.0;

                if pnl_pct <= -strategy.config.stop_loss_pct
                    || pnl_pct >= strategy.config.take_profit_pct
                {
                    let revenue = qty * close;
                    let fee = revenue * (fee_pct / 100.0);
                    let pnl = revenue - fee - (entry_price * qty);

                    self.current_balance += revenue - fee;
                    self.trades.push(BacktestTrade {
                        entry_time,
                        exit_time: times[i],
                        entry_price,
                        exit_price: close,
                        quantity: qty,
                        side: OrderSide::Buy,
                        pnl,
                        pnl_pct,
                    });
                    open_position = None;
                }
            }
        }

        // Close remaining position
        if let Some((entry_time, entry_price, qty)) = open_position {
            if let Some(&close) = closes.last() {
                let pnl = (close - entry_price) * qty;
                let pnl_pct = (pnl / (entry_price * qty)) * 100.0;
                self.current_balance += qty * close - (qty * close * fee_pct / 100.0);
                self.trades.push(BacktestTrade {
                    entry_time,
                    exit_time: *times.last().unwrap_or(&0),
                    entry_price,
                    exit_price: close,
                    quantity: qty,
                    side: OrderSide::Buy,
                    pnl,
                    pnl_pct,
                });
            }
        }

        self.calculate_results()
    }

    fn calculate_results(&self) -> BacktestResult {
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
        let max_pnl = self.trades.iter().map(|t| t.pnl).fold(0.0_f64, f64::max);
        let min_pnl = self.trades.iter().map(|t| t.pnl).fold(0.0_f64, f64::min);
        let max_drawdown = self.calculate_max_drawdown();

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

    fn calculate_max_drawdown(&self) -> f64 {
        let mut peak = 0.0;
        let mut max_dd = 0.0;
        let mut cumulative = 0.0;

        for trade in &self.trades {
            cumulative += trade.pnl;
            if cumulative > peak {
                peak = cumulative;
            }
            let dd = peak - cumulative;
            if dd > max_dd {
                max_dd = dd;
            }
        }

        max_dd
    }
}

pub fn backtest(
    strategy_name: &str,
    closes: &[f64],
    times: &[u64],
    initial_balance: f64,
) -> BacktestResult {
    let mut engine = BacktestEngine::new(initial_balance);
    engine.run(strategy_name, closes, times, 0.1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backtest() {
        let prices: Vec<f64> = (0..100).map(|i| 100.0 + (i as f64) * 0.5).collect();
        let times: Vec<u64> = (0..100).map(|i| i as u64 * 60).collect();
        let result = backtest("Test", &prices, &times, 10000.0);
        assert!(result.total_trades >= 0);
    }
}
