use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Candle {
    pub time: u64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

impl Candle {
    pub fn new(time: u64, open: f64, high: f64, low: f64, close: f64) -> Self {
        Self {
            time,
            open,
            high,
            low,
            close,
        }
    }
}

pub fn calculate_rsi(closes: &[f64], period: usize) -> Vec<Option<f64>> {
    let mut rsi_values = vec![None; closes.len()];
    if closes.len() < period + 1 {
        return rsi_values;
    }

    let mut gains = vec![0.0; closes.len()];
    let mut losses = vec![0.0; closes.len()];

    for i in 1..closes.len() {
        let change = closes[i] - closes[i - 1];
        if change > 0.0 {
            gains[i] = change;
        } else {
            losses[i] = -change;
        }
    }

    let mut avg_gain = gains[1..=period].iter().sum::<f64>() / period as f64;
    let mut avg_loss = losses[1..=period].iter().sum::<f64>() / period as f64;

    for i in period..closes.len() {
        if avg_loss == 0.0 {
            rsi_values[i] = Some(100.0);
        } else {
            let rs = avg_gain / avg_loss;
            rsi_values[i] = Some(100.0 - (100.0 / (1.0 + rs)));
        }
        if i < closes.len() - 1 {
            avg_gain = (avg_gain * (period as f64 - 1.0) + gains[i + 1]) / period as f64;
            avg_loss = (avg_loss * (period as f64 - 1.0) + losses[i + 1]) / period as f64;
        }
    }

    rsi_values
}

pub fn calculate_ema(closes: &[f64], period: usize) -> Vec<Option<f64>> {
    let mut ema_values = vec![None; closes.len()];
    if closes.len() < period {
        return ema_values;
    }

    let multiplier = 2.0 / (period as f64 + 1.0);
    let mut ema = closes[..period].iter().sum::<f64>() / period as f64;
    ema_values[period - 1] = Some(ema);

    for i in period..closes.len() {
        ema = (closes[i] * multiplier) + (ema * (1.0 - multiplier));
        ema_values[i] = Some(ema);
    }

    ema_values
}

pub struct MACDResult {
    pub macd: Option<f64>,
    pub signal: Option<f64>,
    pub histogram: Option<f64>,
}

pub fn calculate_macd(closes: &[f64]) -> Vec<MACDResult> {
    let ema12 = calculate_ema(closes, 12);
    let ema26 = calculate_ema(closes, 26);
    let mut macd_line = vec![None; closes.len()];

    for i in 0..closes.len() {
        if let (Some(e12), Some(e26)) = (ema12[i], ema26[i]) {
            macd_line[i] = Some(e12 - e26);
        }
    }

    let macd_values: Vec<f64> = macd_line.iter().filter_map(|&x| x).collect();

    let signal_line = if macd_values.len() >= 9 {
        calculate_ema(&macd_values, 9)
    } else {
        vec![None; macd_values.len()]
    };

    let mut results = Vec::new();
    for i in 0..closes.len() {
        let signal_idx = i.saturating_sub(26 + 9 - 1);
        let signal_val = if signal_idx < signal_line.len() {
            signal_line[signal_idx]
        } else {
            None
        };

        let histogram = match (macd_line[i], signal_val) {
            (Some(m), Some(s)) => Some(m - s),
            _ => None,
        };
        results.push(MACDResult {
            macd: macd_line[i],
            signal: signal_val,
            histogram,
        });
    }

    results
}

pub fn calculate_sma(closes: &[f64], period: usize) -> Vec<Option<f64>> {
    let mut sma_values = vec![None; closes.len()];
    if closes.len() < period {
        return sma_values;
    }

    for i in (period - 1)..closes.len() {
        let sum: f64 = closes[i + 1 - period..=i].iter().sum();
        sma_values[i] = Some(sum / period as f64);
    }

    sma_values
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rsi() {
        let prices = vec![
            44.0, 44.34, 44.09, 43.61, 44.33, 44.83, 45.10, 45.42, 45.84, 46.08, 45.89, 46.03,
            45.61, 46.28, 45.64, 46.21,
        ];
        let rsi = calculate_rsi(&prices, 14);
        assert!(rsi[15].is_some());
    }

    #[test]
    fn test_ema() {
        let prices = vec![100.0, 101.0, 102.0, 103.0, 104.0];
        let ema = calculate_ema(&prices, 5);
        assert!(ema[4].is_some());
    }
}
