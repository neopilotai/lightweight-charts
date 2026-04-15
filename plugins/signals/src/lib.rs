use lightweight_charts_indicators::{calculate_ema, calculate_macd, calculate_rsi, MACDResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SignalType {
    BuySignal,
    SellSignal,
    StrongBuy,
    StrongSell,
    Neutral,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signal {
    pub symbol: String,
    pub signal_type: SignalType,
    pub confidence: f64,
    pub timestamp: i64,
    pub rsi: Option<f64>,
    pub macd: Option<f64>,
    pub signal_line: Option<f64>,
}

pub struct SignalGenerator;

impl SignalGenerator {
    pub fn generate(closes: &[f64], price: f64) -> Option<Signal> {
        if closes.len() < 26 {
            return None;
        }

        let rsi_values = calculate_rsi(closes, 14);
        let rsi = if let Some(r) = rsi_values.last() {
            *r
        } else {
            None
        };

        let macd_results = calculate_macd(closes);
        let last_macd = macd_results.last()?;

        let macd_val = last_macd.macd;
        let signal_val = last_macd.signal;
        let macd_bullish = if let (Some(m), Some(s)) = (macd_val, signal_val) {
            Some(m > s)
        } else {
            None
        };

        let ema12 = calculate_ema(closes, 12);
        let ema26 = calculate_ema(closes, 26);

        let ema12_val = ema12.last().and_then(|v| *v);
        let ema26_val = ema26.last().and_then(|v| *v);
        let ema_bullish = if let (Some(e12), Some(e26)) = (ema12_val, ema26_val) {
            Some(e12 > e26)
        } else {
            None
        };

        let mut buy_score: f64 = 0.0;
        let mut sell_score: f64 = 0.0;

        if let Some(r) = rsi {
            if r < 30.0 {
                buy_score += 2.0;
            } else if r < 40.0 {
                buy_score += 1.0;
            } else if r > 70.0 {
                sell_score += 2.0;
            } else if r > 60.0 {
                sell_score += 1.0;
            }
        }

        if let Some(true) = macd_bullish {
            buy_score += 2.0;
        } else if let Some(false) = macd_bullish {
            sell_score += 2.0;
        }

        if let Some(true) = ema_bullish {
            buy_score += 1.0;
        } else if let Some(false) = ema_bullish {
            sell_score += 1.0;
        }

        let last_close = *closes.last()?;
        if price > last_close * 1.01 {
            buy_score += 0.5;
        } else if price < last_close * 0.99 {
            sell_score += 0.5;
        }

        let total = buy_score + sell_score;
        let confidence: f64 = if total > 0.0 {
            buy_score.max(sell_score) / total
        } else {
            0.0
        };

        if buy_score > sell_score && buy_score > 2.0 && confidence > 0.5 {
            Some(Signal {
                symbol: "BTCUSDT".to_string(),
                signal_type: if confidence > 0.7 {
                    SignalType::StrongBuy
                } else {
                    SignalType::BuySignal
                },
                confidence,
                timestamp: chrono::Utc::now().timestamp(),
                rsi,
                macd: macd_val,
                signal_line: signal_val,
            })
        } else if sell_score > buy_score && sell_score > 2.0 && confidence > 0.5 {
            Some(Signal {
                symbol: "BTCUSDT".to_string(),
                signal_type: if confidence > 0.7 {
                    SignalType::StrongSell
                } else {
                    SignalType::SellSignal
                },
                confidence,
                timestamp: chrono::Utc::now().timestamp(),
                rsi,
                macd: macd_val,
                signal_line: signal_val,
            })
        } else {
            None
        }
    }
}

pub fn generate_signals(closes: &[f64], price: f64) -> Option<Signal> {
    SignalGenerator::generate(closes, price)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_signal() {
        let prices: Vec<f64> = (0..50).map(|i| 100.0 + (i as f64) * 0.5).collect();
        let signal = generate_signals(&prices, 125.0);
        assert!(signal.is_some());
    }
}
