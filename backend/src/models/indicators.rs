// src/models/indicators.rs
use crate::models::candle::Candle;
use std::collections::VecDeque;

pub fn calculate_indicators(candles: &mut Vec<Candle>) {
    if candles.is_empty() {
        return;
    }

    let closes: Vec<f64> = candles.iter().map(|c| c.close).collect();

    // Calculate RSI (14-period)
    let rsi_values = calculate_rsi(&closes, 14);
    for (i, rsi) in rsi_values.into_iter().enumerate() {
        if let Some(candle) = candles.get_mut(i) {
            candle.rsi = rsi;
        }
    }

    // Calculate EMAs
    let ema12_values = calculate_ema(&closes, 12);
    let ema26_values = calculate_ema(&closes, 26);

    for (i, ema12) in ema12_values.into_iter().enumerate() {
        if let Some(candle) = candles.get_mut(i) {
            candle.ema12 = ema12;
        }
    }
    for (i, ema26) in ema26_values.into_iter().enumerate() {
        if let Some(candle) = candles.get_mut(i) {
            candle.ema26 = ema26;
        }
    }

    // Calculate MACD
    let macd_values = calculate_macd(&closes);
    for (i, macd) in macd_values.into_iter().enumerate() {
        if let Some(candle) = candles.get_mut(i) {
            candle.macd = macd.macd;
            candle.signal = macd.signal;
            candle.histogram = macd.histogram;
        }
    }
}

pub fn update_indicators_last(candles: &mut VecDeque<Candle>) {
    if candles.is_empty() {
        return;
    }

    // Clone the candles to avoid borrowing issues during calculations
    let candles_clone: VecDeque<Candle> = candles.clone();

    // Get the last candle for updating
    if let Some(last_candle) = candles.back_mut() {
        // Get a clone of the closes for calculations to avoid borrowing issues
        let closes: Vec<f64> = candles_clone.iter().map(|c| c.close).collect();

        // Update RSI for last candle (needs access to recent closes)
        if closes.len() >= 15 {
            // Need at least period+1 for RSI
            if let Some(rsi) = update_rsi_last(&closes, 14) {
                last_candle.rsi = Some(rsi);
            }
        }

        // Update EMAs for last candle (truly incremental)
        if let Some(prev_ema12) = last_candle.ema12 {
            // EMA formula: EMA_today = (Price_today * k) + (EMA_yesterday * (1 - k))
            // where k = 2 / (period + 1)
            let k12 = 2.0 / (12.0 + 1.0);
            let ema12 = (last_candle.close * k12) + (prev_ema12 * (1.0 - k12));
            last_candle.ema12 = Some(ema12);
        } else if closes.len() >= 12 {
            // Initial EMA is simple average of first 12 periods
            let sum: f64 = closes.iter().rev().take(12).sum();
            last_candle.ema12 = Some(sum / 12.0);
        }

        if let Some(prev_ema26) = last_candle.ema26 {
            let k26 = 2.0 / (26.0 + 1.0);
            let ema26 = (last_candle.close * k26) + (prev_ema26 * (1.0 - k26));
            last_candle.ema26 = Some(ema26);
        } else if closes.len() >= 26 {
            // Initial EMA is simple average of first 26 periods
            let sum: f64 = closes.iter().rev().take(26).sum();
            last_candle.ema26 = Some(sum / 26.0);
        }

        // Update MACD components
        if let (Some(ema12), Some(ema26)) = (last_candle.ema12, last_candle.ema26) {
            let macd = ema12 - ema26;
            last_candle.macd = Some(macd);

            // For signal line and histogram, we'd need to maintain EMA of MACD
            // Simplified approach: use previous values if available
            if let (Some(prev_signal), Some(_prev_histogram)) =
                (last_candle.signal, last_candle.histogram)
            {
                // Simplified signal update (would be better with proper EMA of MACD)
                let signal = macd * 0.2 + prev_signal * 0.8; // Approximate 9-period EMA
                last_candle.signal = Some(signal);
                last_candle.histogram = Some(macd - signal);
            }
        }
    }
}

fn calculate_rsi(closes: &[f64], period: usize) -> Vec<Option<f64>> {
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

fn update_rsi_last(closes: &[f64], period: usize) -> Option<f64> {
    if closes.len() < period + 1 {
        return None;
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
        if i == closes.len() - 1 {
            if avg_loss == 0.0 {
                return Some(100.0);
            } else {
                let rs = avg_gain / avg_loss;
                return Some(100.0 - (100.0 / (1.0 + rs)));
            }
        }
        avg_gain = (avg_gain * (period as f64 - 1.0) + gains[i + 1]) / period as f64;
        avg_loss = (avg_loss * (period as f64 - 1.0) + losses[i + 1]) / period as f64;
    }

    None
}

fn calculate_ema(closes: &[f64], period: usize) -> Vec<Option<f64>> {
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

fn update_ema_last(closes: &[f64], period: usize) -> Option<f64> {
    if closes.len() < period {
        return None;
    }

    let multiplier = 2.0 / (period as f64 + 1.0);
    let mut ema = closes[..period].iter().sum::<f64>() / period as f64;

    for i in period..closes.len() {
        ema = (closes[i] * multiplier) + (ema * (1.0 - multiplier));
    }

    Some(ema)
}

struct MacdValue {
    macd: Option<f64>,
    signal: Option<f64>,
    histogram: Option<f64>,
}

fn calculate_macd(closes: &[f64]) -> Vec<MacdValue> {
    let ema12 = calculate_ema(closes, 12);
    let ema26 = calculate_ema(closes, 26);
    let mut macd_line = vec![None; closes.len()];

    for i in 0..closes.len() {
        if let (Some(e12), Some(e26)) = (ema12[i], ema26[i]) {
            macd_line[i] = Some(e12 - e26);
        }
    }

    let signal_line = calculate_ema(
        &macd_line.iter().filter_map(|&x| x).collect::<Vec<f64>>(),
        9,
    );
    let mut signal_full = vec![None; closes.len()];
    let start = 26 + 9 - 1; // EMA26 starts at 25, signal at 25+8=33
    for (i, sig) in signal_line.into_iter().enumerate() {
        signal_full[i + start] = sig;
    }

    let mut macd_values = vec![];

    for i in 0..closes.len() {
        let histogram = match (macd_line[i], signal_full[i]) {
            (Some(m), Some(s)) => Some(m - s),
            _ => None,
        };
        macd_values.push(MacdValue {
            macd: macd_line[i],
            signal: signal_full[i],
            histogram,
        });
    }

    macd_values
}

fn update_macd_last(closes: &[f64]) -> Option<MacdValue> {
    let ema12 = update_ema_last(closes, 12)?;
    let ema26 = update_ema_last(closes, 26)?;
    let macd = ema12 - ema26;

    // For signal, need full series, simplified for last
    Some(MacdValue {
        macd: Some(macd),
        signal: None, // Would need full recalc
        histogram: None,
    })
}
