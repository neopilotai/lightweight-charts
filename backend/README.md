# Lightweight Charts Backend

High-performance Rust trading backend with real-time market data, signal generation, strategy management, and backtesting.

## Features

- ⚡ **Real-time Data** - Live BTC/USDT candlestreams from Binance WebSocket
- 📊 **Technical Indicators** - RSI, MACD, EMA computed server-side
- 🎯 **Signal Generation** - Multi-indicator scoring with confidence levels
- 📈 **Strategy Management** - Create, enable, disable trading strategies
- 💰 **Trading Engine** - Position tracking, P&L calculation, portfolio stats
- 🔬 **Backtesting** - Historical strategy validation with detailed metrics
- 🌐 **REST API** - Full HTTP API for all trading operations
- 🔌 **WebSocket** - Real-time data streaming to frontend

## Quick Start

```bash
# Clone and build
cd backend
cargo build --release

# Run
cargo run --release

# Server runs on http://localhost:3000
```

## API Endpoints

| Endpoint | Description |
|----------|-------------|
| `GET /api/candles?symbol=btcusdt` | Get candlestick data with indicators |
| `POST /api/trading/strategies` | Create trading strategy |
| `GET /api/trading/strategies/list` | List all strategies |
| `GET /api/trading/signals` | Get current trading signals |
| `GET /ws?symbol=btcusdt` | WebSocket for real-time data |

## Example: Create Strategy

```bash
curl -X POST http://localhost:3000/api/trading/strategies \
  -H "Content-Type: application/json" \
  -d '{
    "name": "RSI Momentum",
    "strategy_type": "rsi_momentum",
    "symbol": "BTCUSDT",
    "risk_percent": 2.0,
    "stop_loss_pct": 2.0,
    "take_profit_pct": 5.0
  }'
```

## Performance

- Signal Generation: <1ms latency
- Order Execution: <100μs
- Backtesting: ~1000 candles/ms

## License

MIT