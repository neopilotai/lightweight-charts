// src/models/indicators.rs
use crate::models::candle::Candle;
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct IndicatorParams {
    pub rsi_period: usize,
    pub ema_short: usize,
    pub ema_long: usize,
    pub macd_fast: usize,
    pub macd_slow: usize,
    pub macd_signal: usize,
    pub bb_period: usize,
    pub bb_std: f64,
    pub stoch_period: usize,
    pub stoch_smooth: usize,
}

impl Default for IndicatorParams {
    fn default() -> Self {
        Self {
            rsi_period: 14,
            ema_short: 12,
            ema_long: 26,
            macd_fast: 12,
            macd_slow: 26,
            macd_signal: 9,
            bb_period: 20,
            bb_std: 2.0,
            stoch_period: 14,
            stoch_smooth: 3,
        }
    }
}

pub fn calculate_indicators_with_params(candles: &mut Vec<Candle>, params: &IndicatorParams) {
    if candles.is_empty() {
        return;
    }

    let closes: Vec<f64> = candles.iter().map(|c| c.close).collect();
    let highs: Vec<f64> = candles.iter().map(|c| c.high).collect();
    let lows: Vec<f64> = candles.iter().map(|c| c.low).collect();

    // Calculate RSI
    let rsi_values = calculate_rsi(&closes, params.rsi_period);
    for (i, rsi) in rsi_values.into_iter().enumerate() {
        if let Some(candle) = candles.get_mut(i) {
            candle.rsi = rsi;
        }
    }

    // Calculate EMAs
    let ema_short_values = calculate_ema(&closes, params.ema_short);
    let ema_long_values = calculate_ema(&closes, params.ema_long);

    for (i, ema) in ema_short_values.into_iter().enumerate() {
        if let Some(candle) = candles.get_mut(i) {
            candle.ema12 = ema;
        }
    }
    for (i, ema) in ema_long_values.into_iter().enumerate() {
        if let Some(candle) = candles.get_mut(i) {
            candle.ema26 = ema;
        }
    }

    // Calculate MACD with custom parameters
    let macd_values = calculate_macd_custom(
        &closes,
        params.macd_fast,
        params.macd_slow,
        params.macd_signal,
    );
    for (i, macd) in macd_values.into_iter().enumerate() {
        if let Some(candle) = candles.get_mut(i) {
            candle.macd = macd.macd;
            candle.signal = macd.signal;
            candle.histogram = macd.histogram;
        }
    }

    // Calculate Bollinger Bands
    let bb_values = calculate_bollinger_bands(&closes, params.bb_period, params.bb_std);
    for (i, bb) in bb_values.into_iter().enumerate() {
        if let Some(candle) = candles.get_mut(i) {
            candle.bollinger_upper = bb.upper;
            candle.bollinger_middle = bb.middle;
            candle.bollinger_lower = bb.lower;
        }
    }

    // Calculate Stochastic
    let stoch_values = calculate_stochastic(
        &highs,
        &lows,
        &closes,
        params.stoch_period,
        params.stoch_smooth,
    );
    for (i, stoch) in stoch_values.into_iter().enumerate() {
        if let Some(candle) = candles.get_mut(i) {
            candle.stoch_k = stoch.k;
            candle.stoch_d = stoch.d;
        }
    }
}

pub fn calculate_indicators(candles: &mut Vec<Candle>) {
    calculate_indicators_with_params(candles, &IndicatorParams::default());
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

    Some(MacdValue {
        macd: Some(macd),
        signal: None,
        histogram: None,
    })
}

fn calculate_macd_custom(
    closes: &[f64],
    fast: usize,
    slow: usize,
    signal: usize,
) -> Vec<MacdValue> {
    let ema_fast = calculate_ema(closes, fast);
    let ema_slow = calculate_ema(closes, slow);
    let mut macd_line = vec![None; closes.len()];

    for i in 0..closes.len() {
        if let (Some(f), Some(s)) = (ema_fast[i], ema_slow[i]) {
            macd_line[i] = Some(f - s);
        }
    }

    let signal_values: Vec<f64> = macd_line.iter().filter_map(|&x| x).collect();
    let signal_line = calculate_ema(&signal_values, signal);

    let mut signal_full = vec![None; closes.len()];
    let start = slow + signal - 1;
    for (i, sig) in signal_line.into_iter().enumerate() {
        if let Some(s) = sig {
            if i + start < closes.len() {
                signal_full[i + start] = Some(s);
            }
        }
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

struct BollingerValue {
    upper: Option<f64>,
    middle: Option<f64>,
    lower: Option<f64>,
}

fn calculate_bollinger_bands(
    closes: &[f64],
    period: usize,
    std_multiplier: f64,
) -> Vec<BollingerValue> {
    let mut bb_values = vec![
        BollingerValue {
            upper: None,
            middle: None,
            lower: None
        };
        closes.len()
    ];

    if closes.len() < period {
        return bb_values;
    }

    for i in (period - 1)..closes.len() {
        let slice = &closes[i - period + 1..=i];
        let mean = slice.iter().sum::<f64>() / period as f64;
        let variance = slice.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / period as f64;
        let std = variance.sqrt();

        bb_values[i] = BollingerValue {
            middle: Some(mean),
            upper: Some(mean + std_multiplier * std),
            lower: Some(mean - std_multiplier * std),
        };
    }
    bb_values
}

struct StochasticValue {
    k: Option<f64>,
    d: Option<f64>,
}

fn calculate_stochastic(
    highs: &[f64],
    lows: &[f64],
    closes: &[f64],
    period: usize,
    smooth: usize,
) -> Vec<StochasticValue> {
    let mut stoch_values = vec![StochasticValue { k: None, d: None }; closes.len()];

    if closes.len() < period {
        return stoch_values;
    }

    let mut k_values = vec![None; closes.len()];

    for i in (period - 1)..closes.len() {
        let high_slice = &highs[i - period + 1..=i];
        let low_slice = &lows[i - period + 1..=i];

        let highest = high_slice.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let lowest = low_slice.iter().cloned().fold(f64::INFINITY, f64::min);

        let range = highest - lowest;
        if range > 0.0 {
            k_values[i] = Some(100.0 * (closes[i] - lowest) / range);
        }
    }

    // Smooth %K to get %D
    for i in (smooth - 1)..closes.len() {
        let slice: Vec<f64> = k_values[i - smooth + 1..=i]
            .iter()
            .filter_map(|&x| x)
            .collect();
        if slice.len() == smooth {
            let k = slice.iter().sum::<f64>() / smooth as f64;
            stoch_values[i] = StochasticValue {
                k: k_values[i],
                d: Some(k),
            };
        }
    }

    stoch_values
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_closes() -> Vec<f64> {
        vec![
            44.34, 44.09, 44.15, 43.61, 44.33, 44.83, 45.10, 45.42, 45.84, 46.08, 45.89, 46.03,
            45.61, 46.28, 46.28, 46.00, 46.03, 46.41, 46.22, 45.64, 46.21, 46.25, 45.71, 46.45,
            45.78, 45.35, 44.03, 44.18, 44.22, 44.57, 43.42, 42.66, 43.13, 43.72, 44.03, 43.61,
        ]
    }

    #[test]
    fn test_rsi_calculation_matches_reference() {
        let closes = sample_closes();
        let rsi_values = calculate_rsi(&closes, 14);

        assert_eq!(rsi_values.len(), closes.len());
        assert!(rsi_values[13].is_none());
        let last_rsi = rsi_values.last().unwrap().unwrap();
        let incremental_rsi = update_rsi_last(&closes, 14).unwrap();

        assert!(
            (last_rsi - 71.27).abs() < 0.5,
            "expected RSI around 71.27, got {}",
            last_rsi
        );
        assert!(
            (incremental_rsi - last_rsi).abs() < 1.0e-6,
            "incremental RSI must match full RSI"
        );
    }

    #[test]
    fn test_ema_incremental_equals_full() {
        let closes = sample_closes();
        let ema12_values = calculate_ema(&closes, 12);
        let ema26_values = calculate_ema(&closes, 26);

        let last_ema12 = ema12_values.last().unwrap().unwrap();
        let last_ema26 = ema26_values.last().unwrap().unwrap();

        let incremental_ema12 = update_ema_last(&closes, 12).unwrap();
        let incremental_ema26 = update_ema_last(&closes, 26).unwrap();

        assert!((incremental_ema12 - last_ema12).abs() < 1.0e-6);
        assert!((incremental_ema26 - last_ema26).abs() < 1.0e-6);
    }
}
