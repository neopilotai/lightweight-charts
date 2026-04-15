// src/models/orders.rs
use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum OrderType {
    Market,
    Limit,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum OrderSide {
    Buy,
    Sell,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum OrderStatus {
    Pending,
    Filled,
    PartiallyFilled,
    Cancelled,
    Rejected,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum SignalType {
    BuySignal,
    SellSignal,
    StrongBuy,
    StrongSell,
    Neutral,
}

/// Market order
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Order {
    pub id: String,
    pub symbol: String,
    pub order_type: OrderType,
    pub side: OrderSide,
    pub quantity: f64,
    pub price: f64,
    pub status: OrderStatus,
    pub filled_quantity: f64,
    pub created_at: i64,
    pub updated_at: i64,
}

impl Order {
    pub fn new(
        symbol: String,
        order_type: OrderType,
        side: OrderSide,
        quantity: f64,
        price: f64,
    ) -> Self {
        let now = Utc::now().timestamp();
        Order {
            id: format!("{}_{}", symbol, now),
            symbol,
            order_type,
            side,
            quantity,
            price,
            status: OrderStatus::Pending,
            filled_quantity: 0.0,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn notional_value(&self) -> f64 {
        self.quantity * self.price
    }

    pub fn remaining_quantity(&self) -> f64 {
        self.quantity - self.filled_quantity
    }

    pub fn is_fully_filled(&self) -> bool {
        (self.filled_quantity - self.quantity).abs() < 1e-6
    }
}

/// Executed trade
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Trade {
    pub id: String,
    pub symbol: String,
    pub side: OrderSide,
    pub quantity: f64,
    pub entry_price: f64,
    pub executed_at: i64,
    pub commission: f64,
}

impl Trade {
    pub fn new(symbol: String, side: OrderSide, quantity: f64, entry_price: f64) -> Self {
        Trade {
            id: format!("TRADE_{}_{}", symbol, Utc::now().timestamp()),
            symbol,
            side,
            quantity,
            entry_price,
            executed_at: Utc::now().timestamp(),
            commission: 0.0, // 0% for now, can add commission logic
        }
    }

    pub fn notional_value(&self) -> f64 {
        self.quantity * self.entry_price
    }
}

/// Open position
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Position {
    pub symbol: String,
    pub quantity: f64,
    pub entry_price: f64,
    pub current_price: f64,
    pub opened_at: i64,
    pub side: OrderSide,
}

impl Position {
    pub fn new(symbol: String, quantity: f64, entry_price: f64, side: OrderSide) -> Self {
        Position {
            symbol,
            quantity,
            entry_price,
            current_price: entry_price,
            opened_at: Utc::now().timestamp(),
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

    pub fn update_price(&mut self, new_price: f64) {
        self.current_price = new_price;
    }
}

/// Trading signal with confidence
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Signal {
    pub symbol: String,
    pub signal_type: SignalType,
    pub confidence: f64, // 0.0 to 1.0
    pub timestamp: i64,
    pub indicators: SignalIndicators,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SignalIndicators {
    pub rsi: Option<f64>,
    pub macd_signal: Option<bool>, // true if MACD > signal
    pub ema_signal: Option<bool>,  // true if EMA12 > EMA26
    pub price: f64,
}

impl Signal {
    pub fn new_buy(symbol: String, confidence: f64, indicators: SignalIndicators) -> Self {
        Signal {
            symbol,
            signal_type: if confidence > 0.7 {
                SignalType::StrongBuy
            } else {
                SignalType::BuySignal
            },
            confidence,
            timestamp: Utc::now().timestamp(),
            indicators,
        }
    }

    pub fn new_sell(symbol: String, confidence: f64, indicators: SignalIndicators) -> Self {
        Signal {
            symbol,
            signal_type: if confidence > 0.7 {
                SignalType::StrongSell
            } else {
                SignalType::SellSignal
            },
            confidence,
            timestamp: Utc::now().timestamp(),
            indicators,
        }
    }
}

/// Trade P&L record
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TradeResult {
    pub entry_time: i64,
    pub exit_time: i64,
    pub entry_price: f64,
    pub exit_price: f64,
    pub quantity: f64,
    pub pnl: f64,
    pub pnl_pct: f64,
    pub side: OrderSide,
    pub symbol: String,
}

impl TradeResult {
    pub fn new(
        entry_time: i64,
        exit_time: i64,
        entry_price: f64,
        exit_price: f64,
        quantity: f64,
        side: OrderSide,
        symbol: String,
    ) -> Self {
        let pnl = match side {
            OrderSide::Buy => (exit_price - entry_price) * quantity,
            OrderSide::Sell => (entry_price - exit_price) * quantity,
        };
        let pnl_pct = (pnl / (entry_price * quantity)) * 100.0;

        TradeResult {
            entry_time,
            exit_time,
            entry_price,
            exit_price,
            quantity,
            pnl,
            pnl_pct,
            side,
            symbol,
        }
    }

    pub fn is_winning(&self) -> bool {
        self.pnl > 0.0
    }
}

/// Portfolio statistics
#[derive(Clone, Debug, Serialize, Deserialize)]
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
