use crate::backtest::types::*;

pub struct Executor {
    pub balance: f64,
    pub position: Option<(f64, u64)>, // (entry_price, time)
    pub trades: Vec<Trade>,
}

impl Executor {
    pub fn new(balance: f64) -> Self {
        Self {
            balance,
            position: None,
            trades: vec![],
        }
    }

    pub fn on_signal(&mut self, signal: &str, price: f64, time: u64) {
        match signal {
            "BUY" => {
                if self.position.is_none() {
                    self.position = Some((price, time));
                }
            }

            "SELL" => {
                if let Some((entry, entry_time)) = self.position.take() {
                    let profit = price - entry;

                    self.trades.push(Trade {
                        entry_price: entry,
                        exit_price: price,
                        profit,
                        side: Side::Buy,
                        entry_time,
                        exit_time: time,
                    });

                    self.balance += profit;
                }
            }

            _ => {}
        }
    }
}