# lightweight-charts-indicators

Technical indicators library for trading - RSI, MACD, EMA, SMA.

## Usage

```rust
use lightweight_charts_indicators::{calculate_rsi, calculate_ema, calculate_macd};

let prices = vec![100.0, 101.0, 102.0, 103.0, 104.0, 105.0, 106.0, 107.0, 108.0, 109.0];

// RSI
let rsi = calculate_rsi(&prices, 14);

// EMA
let ema12 = calculate_ema(&prices, 12);
let ema26 = calculate_ema(&prices, 26);

// MACD
let macd = calculate_macd(&prices);
```

## Features

- RSI (Relative Strength Index)
- EMA (Exponential Moving Average)
- MACD (Moving Average Convergence Divergence)
- SMA (Simple Moving Average)

## License

MIT