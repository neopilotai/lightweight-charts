use lightweight_charts_signals::{generate_signals, Signal, SignalType};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StrategyType {
    MovingAverageCrossover,
    RSIMomentum,
    MACDCrossover,
    MultiIndicator,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyConfig {
    pub id: String,
    pub name: String,
    pub strategy_type: StrategyType,
    pub symbol: String,
    pub enabled: bool,
    pub risk_percent: f64,
    pub stop_loss_pct: f64,
    pub take_profit_pct: f64,
    pub max_positions: usize,
    pub position_size: f64,
}

impl StrategyConfig {
    pub fn new(name: String, strategy_type: StrategyType, symbol: String) -> Self {
        Self {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub symbol: String,
    pub quantity: f64,
    pub entry_price: f64,
    pub current_price: f64,
    pub side: OrderSide,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OrderSide {
    Buy,
    Sell,
}

impl Position {
    pub fn new(symbol: String, quantity: f64, entry_price: f64, side: OrderSide) -> Self {
        Self {
            symbol,
            quantity,
            entry_price,
            current_price: entry_price,
            side,
        }
    }

    pub fn unrealized_pnl(&self) -> f64 {
        match self.side {
            OrderSide::Buy => (self.current_price - self.entry_price) * self.quantity,
            OrderSide::Sell => (self.entry_price - self.current_price) * self.quantity,
        }
    }

    pub fn unrealized_pnl_pct(&self) -> f64 {
        if self.entry_price == 0.0 {
            return 0.0;
        }
        match self.side {
            OrderSide::Buy => ((self.current_price - self.entry_price) / self.entry_price) * 100.0,
            OrderSide::Sell => ((self.entry_price - self.current_price) / self.entry_price) * 100.0,
        }
    }

    pub fn update_price(&mut self, price: f64) {
        self.current_price = price;
    }
}

pub struct Strategy {
    pub config: StrategyConfig,
    pub positions: Vec<Position>,
}

impl Strategy {
    pub fn new(config: StrategyConfig) -> Self {
        Self {
            config,
            positions: Vec::new(),
        }
    }

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

    pub fn should_exit(&self, position: &Position) -> Option<String> {
        let pnl_pct = position.unrealized_pnl_pct();
        if pnl_pct <= -self.config.stop_loss_pct {
            return Some("stop_loss".to_string());
        }
        if pnl_pct >= self.config.take_profit_pct {
            return Some("take_profit".to_string());
        }
        None
    }

    pub fn calculate_position_size(&self, account_balance: f64, entry_price: f64) -> f64 {
        if self.config.position_size > 0.0 {
            return self.config.position_size;
        }
        let risk_amount = account_balance * (self.config.risk_percent / 100.0);
        let stop_distance = entry_price * (self.config.stop_loss_pct / 100.0);
        (risk_amount / stop_distance).max(0.001)
    }

    pub fn process_signal(
        &mut self,
        closes: &[f64],
        price: f64,
        account_balance: f64,
    ) -> (Vec<Position>, Vec<Position>) {
        let mut entered = Vec::new();
        let mut exited = Vec::new();

        // Check exits
        let exit_indices: Vec<usize> = self
            .positions
            .iter()
            .enumerate()
            .filter(|(_, p)| {
                let pnl_pct = ((price - p.entry_price) / p.entry_price) * 100.0;
                pnl_pct <= -self.config.stop_loss_pct || pnl_pct >= self.config.take_profit_pct
            })
            .map(|(i, _)| i)
            .collect();

        for &idx in exit_indices.iter().rev() {
            if let Some(pos) = self.positions.get(idx) {
                exited.push(pos.clone());
            }
        }

        self.positions = self
            .positions
            .iter()
            .enumerate()
            .filter(|(i, _)| !exit_indices.contains(i))
            .map(|(_, p)| p.clone())
            .collect();

        // Check entry
        if let Some(signal) = generate_signals(closes, price) {
            if self.should_enter(&signal.signal_type, self.positions.len()) {
                let qty = self.calculate_position_size(account_balance, price);
                let side = match signal.signal_type {
                    SignalType::BuySignal | SignalType::StrongBuy => OrderSide::Buy,
                    _ => OrderSide::Sell,
                };
                let pos = Position::new(self.config.symbol.clone(), qty, price, side);
                entered.push(pos.clone());
                self.positions.push(pos);
            }
        }

        (entered, exited)
    }
}

pub fn create_strategy(name: &str, strategy_type: StrategyType, symbol: &str) -> Strategy {
    let config = StrategyConfig::new(name.to_string(), strategy_type, symbol.to_string());
    Strategy::new(config)
}

pub fn run_strategy(
    strategy: &mut Strategy,
    closes: &[f64],
    price: f64,
    balance: f64,
) -> (Vec<Position>, Vec<Position>) {
    strategy.process_signal(closes, price, balance)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strategy_creation() {
        let strategy = create_strategy("Test", StrategyType::RSIMomentum, "BTCUSDT");
        assert_eq!(strategy.config.name, "Test");
    }

    #[test]
    fn test_position_sizing() {
        let strategy = create_strategy("Test", StrategyType::RSIMomentum, "BTCUSDT");
        let size = strategy.calculate_position_size(10000.0, 50000.0);
        assert!(size > 0.0);
    }
}
