pub fn ema(prev: f64, price: f64, period: usize) -> f64 {
    let k = 2.0 / (period as f64 + 1.0);
    price * k + prev * (1.0 - k)
}

pub fn rsi(closes: &[f64], period: usize) -> f64 {
    if closes.len() < period + 1 {
        return 50.0;
    }

    let mut gain = 0.0;
    let mut loss = 0.0;

    for i in closes.len() - period..closes.len() - 1 {
        let diff = closes[i + 1] - closes[i];
        if diff > 0.0 {
            gain += diff;
        } else {
            loss -= diff;
        }
    }

    if loss == 0.0 {
        return 100.0;
    }

    let rs = gain / loss;
    100.0 - (100.0 / (1.0 + rs))
}

pub fn highest(values: &[f64]) -> f64 {
    values.iter().copied().fold(f64::NEG_INFINITY, f64::max)
}

pub fn lowest(values: &[f64]) -> f64 {
    values.iter().copied().fold(f64::INFINITY, f64::min)
}
