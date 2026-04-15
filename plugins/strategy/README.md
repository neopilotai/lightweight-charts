# lightweight-charts-strategy

Trading strategy management and execution library.

## Usage

```rust
use lightweight_charts_strategy::{create_strategy, StrategyType};

let mut strategy = create_strategy("My Strategy", StrategyType::RSIMomentum, "BTCUSDT");

// Process market data
let closes: Vec<f64> = (0..50).map(|i| 100.0 + i as f64).collect();
let price = 125.0;
let balance = 10000.0;

let (entered, exited) = strategy.process_signal(&closes, price, balance);
```

## Features

- Strategy configuration (risk %, stop loss, take profit)
- Position management
- Entry/exit logic based on signals
- Risk-based position sizing

## Dependencies

- lightweight-charts-signals

## License

MIT