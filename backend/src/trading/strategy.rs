// src/trading/strategy.rs
use crate::models::candle::Candle;
use crate::models::orders::{Order, OrderSide, OrderType, Position, SignalType};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum StrategyType {
    MovingAverageCrossover,
    RSIMomentum,
    MACDCrossover,
    MultiIndicator,
    Custom(String),
}

/// Strategy configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StrategyConfig {
    pub id: String,
    pub name: String,
    pub strategy_type: StrategyType,
    pub symbol: String,
    pub enabled: bool,
    pub risk_percent: f64,    // % of portfolio per trade
    pub stop_loss_pct: f64,   // Stop loss percentage
    pub take_profit_pct: f64, // Take profit percentage
    pub max_positions: usize, // Max concurrent positions
    pub position_size: f64,   // Fixed position size (if 0, use risk_percent)
}

impl StrategyConfig {
    pub fn new(name: String, strategy_type: StrategyType, symbol: String) -> Self {
        StrategyConfig {
            id: format!("strategy_{}", chrono::Utc::now().timestamp()),
            name,
            strategy_type,
            symbol,
            enabled: true,
            risk_percent: 2.0,
            stop_loss_pct: 2.0,
            take_profit_pct: 5.0,
            max_positions: 1,
            position_size: 0.0,
        }
    }
}

/// Strategy state machine
pub struct Strategy {
    pub config: StrategyConfig,
    pub positions: Vec<Position>,
    pub pending_orders: Vec<Order>,
    pub closed_trades: Vec<(f64, f64, f64)>, // (entry_price, exit_price, pnl)
    pub total_pnl: f64,
    pub win_count: usize,
    pub loss_count: usize,
}

impl Strategy {
    pub fn new(config: StrategyConfig) -> Self {
        Strategy {
            config,
            positions: Vec::new(),
            pending_orders: Vec::new(),
            closed_trades: Vec::new(),
            total_pnl: 0.0,
            win_count: 0,
            loss_count: 0,
        }
    }

    /// Check if strategy should enter a position
    pub fn should_enter(&self, signal_type: &SignalType, current_positions: usize) -> bool {
        if !self.config.enabled {
            return false;
        }

        if current_positions >= self.config.max_positions {
            return false;
        }

        matches!(
            signal_type,
            SignalType::BuySignal
                | SignalType::StrongBuy
                | SignalType::SellSignal
                | SignalType::StrongSell
        )
    }

    /// Check if strategy should exit position
    pub fn should_exit(&self, position: &Position, current_price: f64) -> Option<String> {
        let pnl_pct = position.unrealized_pnl_pct();

        // Stop loss
        if pnl_pct <= -self.config.stop_loss_pct {
            return Some("Stop loss hit".to_string());
        }

        // Take profit
        if pnl_pct >= self.config.take_profit_pct {
            return Some("Take profit hit".to_string());
        }

        None
    }

    /// Calculate position size based on strategy
    pub fn calculate_position_size(&self, account_balance: f64, entry_price: f64) -> f64 {
        if self.config.position_size > 0.0 {
            return self.config.position_size;
        }

        let risk_amount = account_balance * (self.config.risk_percent / 100.0);
        let stop_distance = entry_price * (self.config.stop_loss_pct / 100.0);

        (risk_amount / stop_distance).floor()
    }

    /// Generate entry order
    pub fn generate_entry_order(
        &self,
        signal_type: &SignalType,
        current_price: f64,
        quantity: f64,
    ) -> Order {
        let side = match signal_type {
            SignalType::BuySignal | SignalType::StrongBuy => OrderSide::Buy,
            SignalType::SellSignal | SignalType::StrongSell => OrderSide::Sell,
            _ => OrderSide::Buy,
        };

        Order::new(
            self.config.symbol.clone(),
            OrderType::Market,
            side,
            quantity,
            current_price,
        )
    }

    /// Generate exit order
    pub fn generate_exit_order(&self, position: &Position, current_price: f64) -> Order {
        let exit_side = match position.side {
            OrderSide::Buy => OrderSide::Sell,
            OrderSide::Sell => OrderSide::Buy,
        };

        Order::new(
            position.symbol.clone(),
            OrderType::Market,
            exit_side,
            position.quantity,
            current_price,
        )
    }

    /// Execute strategy logic
    pub fn update(
        &mut self,
        signal_type: Option<SignalType>,
        current_price: f64,
        account_balance: f64,
    ) -> Vec<Order> {
        let mut orders = Vec::new();

        // First, collect positions that should exit (based on stored data)
        let exit_reasons: Vec<bool> = self
            .positions
            .iter()
            .map(|p| {
                let pnl_pct = p.unrealized_pnl_pct();
                pnl_pct <= -self.config.stop_loss_pct || pnl_pct >= self.config.take_profit_pct
            })
            .collect();

        // Update prices and create exit orders
        let exit_orders: Vec<Order> = self
            .positions
            .iter()
            .enumerate()
            .filter(|(i, _)| exit_reasons[*i])
            .map(|(_, position)| self.generate_exit_order(position, current_price))
            .collect();

        // Rebuild positions without the exited ones
        let mut new_positions = Vec::new();
        for (i, pos) in self.positions.iter().enumerate() {
            if exit_reasons[i] {
                continue;
            }
            let mut new_pos = pos.clone();
            new_pos.update_price(current_price);
            new_positions.push(new_pos);
        }
        self.positions = new_positions;

        // Add exit orders
        for order in exit_orders {
            orders.push(order);
        }

        // Check entries with signals
        if let Some(signal) = signal_type {
            if self.should_enter(&signal, self.positions.len()) {
                let qty = self.calculate_position_size(account_balance, current_price);
                let entry_order = self.generate_entry_order(&signal, current_price, qty);

                let side = entry_order.side.clone();
                self.positions.push(Position::new(
                    self.config.symbol.clone(),
                    qty,
                    current_price,
                    side,
                ));

                orders.push(entry_order);
            }
        }

        orders
    }

    /// Record closed trade
    pub fn record_trade(&mut self, entry_price: f64, exit_price: f64, quantity: f64) {
        let pnl = (exit_price - entry_price) * quantity;
        self.closed_trades.push((entry_price, exit_price, pnl));
        self.total_pnl += pnl;

        if pnl > 0.0 {
            self.win_count += 1;
        } else {
            self.loss_count += 1;
        }
    }

    /// Get statistics
    pub fn stats(&self) -> (usize, usize, f64, f64) {
        let total_trades = self.win_count + self.loss_count;
        let win_rate = if total_trades > 0 {
            (self.win_count as f64 / total_trades as f64) * 100.0
        } else {
            0.0
        };

        (total_trades, self.win_count, win_rate, self.total_pnl)
    }
}

/// Strategy manager
pub struct StrategyManager {
    strategies: HashMap<String, Strategy>,
}

impl StrategyManager {
    pub fn new() -> Self {
        StrategyManager {
            strategies: HashMap::new(),
        }
    }

    pub fn add_strategy(&mut self, config: StrategyConfig) {
        let id = config.id.clone();
        self.strategies.insert(id, Strategy::new(config));
    }

    pub fn remove_strategy(&mut self, id: &str) {
        self.strategies.remove(id);
    }

    pub fn get_strategy(&self, id: &str) -> Option<&Strategy> {
        self.strategies.get(id)
    }

    pub fn get_strategy_mut(&mut self, id: &str) -> Option<&mut Strategy> {
        self.strategies.get_mut(id)
    }

    pub fn list_strategies(&self) -> Vec<StrategyConfig> {
        self.strategies.values().map(|s| s.config.clone()).collect()
    }

    pub fn enable_strategy(&mut self, id: &str) {
        if let Some(strategy) = self.strategies.get_mut(id) {
            strategy.config.enabled = true;
        }
    }

    pub fn disable_strategy(&mut self, id: &str) {
        if let Some(strategy) = self.strategies.get_mut(id) {
            strategy.config.enabled = false;
        }
    }

    pub fn update_config(&mut self, id: &str, config: StrategyConfig) {
        if let Some(strategy) = self.strategies.get_mut(id) {
            strategy.config = config;
        }
    }

    pub fn get_all_stats(&self) -> Vec<(String, usize, usize, f64, f64)> {
        self.strategies
            .values()
            .map(|s| {
                let (total, wins, win_rate, pnl) = s.stats();
                (s.config.name.clone(), total, wins, win_rate, pnl)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strategy_creation() {
        let config = StrategyConfig::new(
            "Test Strategy".to_string(),
            StrategyType::RSIMomentum,
            "BTC".to_string(),
        );
        let strategy = Strategy::new(config);

        assert_eq!(strategy.positions.len(), 0);
        assert_eq!(strategy.total_pnl, 0.0);
    }

    #[test]
    fn test_position_size_calculation() {
        let config = StrategyConfig::new(
            "Test Strategy".to_string(),
            StrategyType::RSIMomentum,
            "BTC".to_string(),
        );
        let strategy = Strategy::new(config);

        let size = strategy.calculate_position_size(10000.0, 50000.0);
        assert!(size > 0.0);
    }
}
