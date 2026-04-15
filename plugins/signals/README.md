# lightweight-charts-signals

Trading signal generation library from multiple indicators.

## Usage

```rust
use lightweight_charts_signals::generate_signals;

let prices: Vec<f64> = (0..50).map(|i| 100.0 + i as f64).collect();
let signal = generate_signals(&prices, 125.0);

if let Some(sig) = signal {
    println!("Signal: {:?} ({:.1}%)", sig.signal_type, sig.confidence * 100.0);
}
```

## Features

- Multi-indicator signal scoring (RSI + MACD + EMA)
- Confidence levels (BuySignal, StrongBuy, SellSignal, StrongSell)
- Price position analysis

## Dependencies

- lightweight-charts-indicators

## License

MIT