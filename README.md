# Rust + Lightweight Charts App

A clean, production-ready full project structure for building a **Rust + Lightweight Charts app** with both REST + real-time support.

## Features

- 🦀 Rust backend (Axum)
- 🌐 Frontend (Lightweight Charts)
- ⚡ WebSocket setup (real-time)
- 📈 **Real Binance BTC/USDT live data**
- 📊 Technical indicators (RSI, EMA, MACD)
- 🎯 Trading signal generation
- 🔬 Backtesting engine
- 💰 Strategy management
- 🔧 Dev workflow

## Project Structure

```
lightweight-charts/
│
├── backend/                 # Rust API server
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs
│       ├── routes/
│       │   ├── mod.rs
│       │   ├── market.rs
│       │   └── trading.rs
│       ├── models/
│       │   ├── candle.rs
│       │   ├── indicators.rs
│       │   └── orders.rs
│       ├── ws/
│       │   ├── handler.rs
│       │   └── binance_listener.rs
│       ├── services/
│       │   └── data_service.rs
│       ├── channels/
│       │   └── mod.rs
│       └── trading/
│           ├── engine.rs      # Position & P&L management
│           ├── strategy.rs   # Strategy configuration & execution
│           ├── signals.rs    # Signal generation from indicators
│           └── backtest.rs  # Historical backtesting
│
├── frontend/               # Static frontend
│   ├── index.html
│   ├── app.js
│   └── styles.css
│
├── .gitignore
└── README.md
```

## How to Run

### 1. Start Backend

```bash
cd backend
cargo run --release
```

### 2. Open Frontend

Just open:

```bash
frontend/index.html
```

(or use Live Server)

## API Endpoints

| Endpoint | Description |
|----------|-------------|
| `GET /api/candles?symbol=btcusdt` | Get candlestick data with indicators |
| `POST /api/trading/strategies` | Create trading strategy |
| `GET /api/trading/strategies/list` | List all strategies |
| `GET /api/trading/signals` | Get current trading signals |
| `GET /ws?symbol=btcusdt` | WebSocket for real-time data |

## Trading Engine

### Signal Generation
- RSI (Relative Strength Index) - oversold/overbought detection
- MACD - trend momentum via crossover signals
- EMA 12/26 - moving average crossover strategy
- Multi-timeframe analysis for signal confirmation

### Strategy Management
- Configurable stop-loss and take-profit percentages
- Risk-based position sizing
- Multiple strategy types: MA Crossover, RSI Momentum, MACD, Multi-Indicator

### Backtesting
- Historical data simulation
- Portfolio statistics: win rate, profit factor, Sharpe ratio
- Max drawdown calculation

### Real-Time Data

The app streams **live BTC/USDT candlestick data** from Binance:

- **Historical Data**: Fetches last 200 1-minute candles via Binance REST API
- **Real-Time Updates**: WebSocket connection to Binance streams live price updates
- **Indicators**: RSI, EMA12, EMA26, MACD computed server-side
- **Chart**: Lightweight Charts displays the data with real-time updates

## Upgrade Ideas

### 🔥 More data sources
* Coinbase feed
* Multiple symbol support

### 📊 More indicators
* Bollinger Bands
* Stochastic Oscillator
* Volume-weighted indicators

### ⚡ Performance boost
* Move calculations to WASM
* GPU acceleration

### 🧠 Advanced UI
* React + Lightweight Charts
* Or Rust frameworks (Leptos/Yew)

## Key Takeaway

This structure gives you:

* 🦀 Rust handles **data + speed + trading logic**
* 🌐 Lightweight Charts handles **rendering**
* ⚡ WebSocket enables **real-time trading UI**
* 📈 **Live market data** from Binance
* 🎯 **Trading strategies** with backtesting