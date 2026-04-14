// src/trading/engine.rs
use crate::models::orders::{Order, OrderSide, Position, Trade, TradeResult};
use dashmap::DashMap;
use std::sync::Arc;

/// Trading engine - manages positions and P&L
pub struct TradingEngine {
    positions: Arc<DashMap<String, Vec<Position>>>,
    closed_trades: Arc<DashMap<String, Vec<TradeResult>>>,
    account_balance: Arc<parking_lot::Mutex<f64>>,
    total_fees: Arc<parking_lot::Mutex<f64>>,
}

impl TradingEngine {
    pub fn new(initial_balance: f64) -> Self {
        TradingEngine {
            positions: Arc::new(DashMap::new()),
            closed_trades: Arc::new(DashMap::new()),
            account_balance: Arc::new(parking_lot::Mutex::new(initial_balance)),
            total_fees: Arc::new(parking_lot::Mutex::new(0.0)),
        }
    }

    /// Execute a buy order
    pub fn execute_buy_order(
        &self,
        symbol: String,
        quantity: f64,
        price: f64,
        fee_pct: f64,
    ) -> Result<Trade, String> {
        let cost = quantity * price;
        let fee = cost * (fee_pct / 100.0);
        let total_cost = cost + fee;

        let mut balance = self.account_balance.lock();
        if *balance < total_cost {
            return Err("Insufficient balance".to_string());
        }

        *balance -= total_cost;
        *self.total_fees.lock() += fee;

        let trade = Trade::new(symbol.clone(), OrderSide::Buy, quantity, price);

        // Add to positions
        self.positions
            .entry(symbol.clone())
            .or_insert_with(Vec::new)
            .push(Position::new(symbol, quantity, price, OrderSide::Buy));

        Ok(trade)
    }

    /// Execute a sell order (close position)
    pub fn execute_sell_order(
        &self,
        symbol: String,
        quantity: f64,
        price: f64,
        entry_price: f64,
        fee_pct: f64,
    ) -> Result<TradeResult, String> {
        let revenue = quantity * price;
        let fee = revenue * (fee_pct / 100.0);
        let net_revenue = revenue - fee;

        let mut balance = self.account_balance.lock();
        *balance += net_revenue;
        *self.total_fees.lock() += fee;

        let pnl = (price - entry_price) * quantity;
        let pnl_pct = ((price - entry_price) / entry_price) * 100.0;

        let result = TradeResult::new(
            chrono::Utc::now().timestamp(),
            chrono::Utc::now().timestamp(),
            entry_price,
            price,
            quantity,
            OrderSide::Sell,
            symbol.clone(),
        );

        // Record closed trade
        self.closed_trades
            .entry(symbol)
            .or_insert_with(Vec::new)
            .push(result.clone());

        Ok(result)
    }

    /// Update position prices
    pub fn update_prices(&self, symbol: &str, new_price: f64) {
        if let Some(mut positions) = self.positions.get_mut(symbol) {
            for pos in positions.iter_mut() {
                pos.update_price(new_price);
            }
        }
    }

    /// Get current positions for symbol
    pub fn get_positions(&self, symbol: &str) -> Vec<Position> {
        self.positions
            .get(symbol)
            .map(|p| p.clone())
            .unwrap_or_default()
    }

    /// Get closed trades for symbol
    pub fn get_closed_trades(&self, symbol: &str) -> Vec<TradeResult> {
        self.closed_trades
            .get(symbol)
            .map(|t| t.clone())
            .unwrap_or_default()
    }

    /// Get account balance
    pub fn get_balance(&self) -> f64 {
        *self.account_balance.lock()
    }

    /// Get total unrealized P&L
    pub fn get_unrealized_pnl(&self) -> f64 {
        let mut total = 0.0;
        for entry in self.positions.iter() {
            for position in entry.value() {
                total += position.unrealized_pnl();
            }
        }
        total
    }

    /// Get total realized P&L
    pub fn get_realized_pnl(&self) -> f64 {
        let mut total = 0.0;
        for entry in self.closed_trades.iter() {
            for trade in entry.value() {
                total += trade.pnl;
            }
        }
        total
    }

    /// Close all positions at given price
    pub fn close_all_positions(&self, prices: &dashmap::DashMap<String, f64>) -> Vec<TradeResult> {
        let mut results = Vec::new();

        for mut positions in self.positions.iter_mut() {
            let symbol = positions.key().clone();
            if let Some(price) = prices.get(&symbol).map(|p| *p) {
                while let Some(pos) = positions.pop() {
                    if let Ok(result) = self.execute_sell_order(
                        symbol.clone(),
                        pos.quantity,
                        price,
                        pos.entry_price,
                        0.1, // 0.1% fee
                    ) {
                        results.push(result);
                    }
                }
            }
        }

        results
    }

    /// Get portfolio statistics
    pub fn get_stats(&self) -> PortfolioStats {
        let closed_trades: Vec<TradeResult> = self
            .closed_trades
            .iter()
            .flat_map(|entry| entry.value().clone())
            .collect();

        let total_trades = closed_trades.len();
        let winning_trades = closed_trades.iter().filter(|t| t.is_winning()).count();
        let losing_trades = total_trades - winning_trades;

        let total_pnl: f64 = closed_trades.iter().map(|t| t.pnl).sum();
        let total_invested: f64 = closed_trades.iter().map(|t| t.entry_price * t.quantity).sum();

        let total_return_pct = if total_invested > 0.0 {
            (total_pnl / total_invested) * 100.0
        } else {
            0.0
        };

        let win_rate = if total_trades > 0 {
            (winning_trades as f64 / total_trades as f64) * 100.0
        } else {
            0.0
        };

        let total_wins: f64 = closed_trades
            .iter()
            .filter(|t| t.is_winning())
            .map(|t| t.pnl)
            .sum();
        let total_losses: f64 = closed_trades
            .iter()
            .filter(|t| !t.is_winning())
            .map(|t| t.pnl.abs())
            .sum();

        let avg_win = if winning_trades > 0 {
            total_wins / winning_trades as f64
        } else {
            0.0
        };

        let avg_loss = if losing_trades > 0 {
            total_losses / losing_trades as f64
        } else {
            0.0
        };

        let profit_factor = if total_losses > 0.0 {
            total_wins / total_losses
        } else {
            0.0
        };

        let max_drawdown = Self::calculate_max_drawdown(&closed_trades);
        let sharpe_ratio = Self::calculate_sharpe_ratio(&closed_trades);

        PortfolioStats {
            total_trades,
            winning_trades,
            losing_trades,
            win_rate,
            total_pnl,
            total_return_pct,
            max_drawdown,
            sharpe_ratio,
            avg_win,
            avg_loss,
            profit_factor,
        }
    }

    fn calculate_max_drawdown(trades: &[TradeResult]) -> f64 {
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

    fn calculate_sharpe_ratio(trades: &[TradeResult]) -> f64 {
        if trades.len() < 2 {
            return 0.0;
        }

        let returns: Vec<f64> = trades.iter().map(|t| t.pnl).collect();
        let mean: f64 = returns.iter().sum::<f64>() / returns.len() as f64;

        let variance: f64 = returns.iter().map(|r| (r - mean).powi(2)).sum::<f64>()
            / returns.len() as f64;
        let std_dev = variance.sqrt();

        if std_dev == 0.0 {
            return 0.0;
        }

        // Annual Sharpe (assuming 1 trade per day)
        let daily_sharpe = mean / std_dev;
        daily_sharpe * (252.0_f64.sqrt())
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct PortfolioStats {
    pub total_trades: usize,
    pub winning_trades: usize,
    pub losing_trades: usize,
    pub win_rate: f64,
    pub total_pnl: f64,
    pub total_return_pct: f64,
    pub max_drawdown: f64,
    pub sharpe_ratio: f64,
    pub avg_win: f64,
    pub avg_loss: f64,
    pub profit_factor: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_initialization() {
        let engine = TradingEngine::new(10000.0);
        assert_eq!(engine.get_balance(), 10000.0);
    }

    #[test]
    fn test_buy_order() {
        let engine = TradingEngine::new(10000.0);
        let result = engine.execute_buy_order("BTC".to_string(), 0.1, 50000.0, 0.1);
        assert!(result.is_ok());
        assert!(engine.get_balance() < 10000.0);
    }
}
