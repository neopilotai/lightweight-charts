# lightweight-charts-backtest

Historical backtesting engine for trading strategies.

## Usage

```rust
use lightweight_charts_backtest::backtest;

let prices: Vec<f64> = (0..100).map(|i| 100.0 + i as f64 * 0.5).collect();
let times: Vec<u64> = (0..100).map(|i| i as u64 * 60).collect();

let result = backtest("Test Strategy", &prices, &times, 10000.0);

println!("Win Rate: {:.1}%", result.win_rate);
println!("Total P&L: ${:.2}", result.total_pnl);
println!("Max Drawdown: ${:.2}", result.max_drawdown);
```

## Features

- Historical strategy simulation
- Trade-by-trade tracking
- Performance metrics (win rate, P&L, drawdown)
- Fee simulation

## Dependencies

- lightweight-charts-strategy

## License

MIT