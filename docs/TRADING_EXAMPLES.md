# Trading System - Quick Start Examples

## Overview

This guide shows how to use the trading engine, strategy, signals, and backtesting components.

## Rust Backend Examples

### 1. Create and Run a Strategy

```rust
// src/main.rs or your trading module

use trading::{
    StrategyConfig, StrategyType, StrategyManager, 
    SignalGenerator, BacktestEngine
};
use std::collections::VecDeque;

#[tokio::main]
async fn main() {
    // Initialize strategy manager
    let mut manager = StrategyManager::new();

    // Create RSI Momentum strategy
    let mut config = StrategyConfig::new(
        "RSI Momentum".to_string(),
        StrategyType::RSIMomentum,
        "BTCUSDT".to_string(),
    );
    config.risk_percent = 2.0;
    config.stop_loss_pct = 2.0;
    config.take_profit_pct = 5.0;
    config.max_positions = 1;

    manager.add_strategy(config);

    // Get strategy stats
    let stats = manager.get_all_stats();
    for (name, total, wins, win_rate, pnl) in stats {
        println!("{}: {}/{} ({:.1}%) P&L: ${:.2}", 
            name, wins, total, win_rate, pnl);
    }
}
```

### 2. Generate Trading Signals

```rust
use trading::SignalGenerator;
use std::collections::VecDeque;

async fn process_candles(
    symbol: &str,
    candles: &VecDeque<Candle>,
    current_price: f64,
) {
    // Generate signal
    if let Some(signal) = SignalGenerator::generate_signal(symbol, candles, current_price) {
        println!("Signal: {:?}", signal.signal_type);
        println!("Confidence: {:.2}%", signal.confidence * 100.0);
        println!("RSI: {}", signal.indicators.rsi.unwrap_or(0.0));
        println!("MACD > Signal: {}", 
            signal.indicators.macd_signal.unwrap_or(false));
    }

    // Multi-timeframe analysis
    let short_signal = SignalGenerator::generate_signal(symbol, &short_candles, current_price);
    let long_signal = SignalGenerator::generate_signal(symbol, &long_candles, current_price);
    
    // Stronger signal if both timeframes agree
    if let (Some(short), Some(long)) = (short_signal, long_signal) {
        if short.signal_type == long.signal_type {
            println!("MULTI-TIMEFRAME CONFIRMATION!");
        }
    }
}
```

### 3. Run Backtest

```rust
use trading::{BacktestEngine, StrategyConfig, StrategyType};

async fn run_strategy_backtest(
    strategy_name: &str,
    historical_candles: &VecDeque<Candle>,
) {
    let mut backtest = BacktestEngine::new(10000.0);

    let config = StrategyConfig::new(
        strategy_name.to_string(),
        StrategyType::RSIMomentum,
        "BTCUSDT".to_string(),
    );

    let result = backtest.backtest(config, historical_candles);

    println!("\n=== BACKTEST RESULTS ===");
    println!("Total Trades: {}", result.total_trades);
    println!("Winning Trades: {} ({:.1}%)", result.winning_trades, result.win_rate);
    println!("Total Return: {:.2}%", result.total_return_pct);
    println!("Max Drawdown: ${:.2}", result.max_drawdown);
    println!("Final Balance: ${:.2}", result.final_balance);
    println!("========================\n");

    // Show some trades
    for (i, trade) in backtest.trades.iter().take(5).enumerate() {
        println!("Trade {}: Entry ${:.2} → Exit ${:.2} - ${:.2} ({:.2}%)",
            i + 1, trade.entry_price, trade.exit_price, trade.pnl, trade.pnl_pct);
    }
}
```

### 4. Trading Engine with Live Execution

```rust
use trading::TradingEngine;

async fn execute_trades() {
    let engine = TradingEngine::new(10000.0);

    // Execute buy order
    let buy_result = engine.execute_buy_order(
        "BTCUSDT".to_string(),
        0.1,      // quantity
        50000.0,  // price
        0.1,      // fee %
    );

    match buy_result {
        Ok(trade) => {
            println!("Buy executed: {} @ ${}", trade.quantity, trade.entry_price);
            
            // Update mark-to-market
            engine.update_prices("BTCUSDT", 51000.0);
            
            // Check unrealized P&L
            let unrealized_pnl = engine.get_unrealized_pnl();
            println!("Unrealized P&L: ${}", unrealized_pnl);
        }
        Err(e) => println!("Buy failed: {}", e),
    }

    // Get portfolio stats
    let stats = engine.get_stats();
    println!("Win Rate: {:.1}%", stats.win_rate);
    println!("Profit Factor: {:.2}", stats.profit_factor);
    println!("Sharpe Ratio: {:.2}", stats.sharpe_ratio);
}
```

## HTTP API Examples

### Create Multiple Strategies

```bash
# RSI Momentum Strategy
curl -X POST http://localhost:3000/api/trading/strategies \
  -H "Content-Type: application/json" \
  -d '{
    "name": "RSI Momentum",
    "strategy_type": "rsi_momentum",
    "symbol": "BTCUSDT",
    "risk_percent": 2.0,
    "stop_loss_pct": 2.0,
    "take_profit_pct": 5.0,
    "max_positions": 1
  }'

# MACD Crossover Strategy
curl -X POST http://localhost:3000/api/trading/strategies \
  -H "Content-Type: application/json" \
  -d '{
    "name": "MACD Crossover",
    "strategy_type": "macd_crossover",
    "symbol": "ETHUSDT",
    "risk_percent": 1.5,
    "stop_loss_pct": 1.5,
    "take_profit_pct": 3.0,
    "max_positions": 1
  }'

# EMA Crossover Strategy
curl -X POST http://localhost:3000/api/trading/strategies \
  -H "Content-Type: application/json" \
  -d '{
    "name": "EMA Crossover",
    "strategy_type": "moving_average_crossover",
    "symbol": "SOLUSDT",
    "risk_percent": 3.0,
    "stop_loss_pct": 2.5,
    "take_profit_pct": 7.5,
    "max_positions": 2
  }'
```

### Monitor Strategy Performance

```bash
# Get all strategy stats
curl http://localhost:3000/api/trading/strategies/stats

# Response:
# [
#   ["RSI Momentum", 10, 8, 80.0, 1250.50],
#   ["MACD Crossover", 5, 4, 80.0, 750.25],
#   ["EMA Crossover", 15, 12, 80.0, 2100.75]
# ]
```

### Get Trading Signals

```bash
# Get current signals for all symbols
curl http://localhost:3000/api/trading/signals

# Response:
# [
#   {
#     "symbol": "BTCUSDT",
#     "signal_type": "BuySignal",
#     "confidence": 0.68,
#     "timestamp": 1681234567,
#     "indicators": {
#       "rsi": 28.5,
#       "macd_signal": true,
#       "ema_signal": true,
#       "price": 50000.00
#     }
#   },
#   {
#     "symbol": "ETHUSDT",
#     "signal_type": "SellSignal",
#     "confidence": 0.72,
#     "timestamp": 1681234568,
#     "indicators": {
#       "rsi": 72.1,
#       "macd_signal": false,
#       "ema_signal": false,
#       "price": 2800.00
#     }
#   }
# ]
```

### Run Backtest via API

```bash
curl -X POST http://localhost:3000/api/trading/backtest \
  -H "Content-Type: application/json" \
  -d '{
    "strategy_id": "strategy_1681234567",
    "symbol": "BTCUSDT",
    "initial_balance": 10000.0
  }'

# Response:
# {
#   "total_trades": 42,
#   "winning_trades": 35,
#   "losing_trades": 7,
#   "win_rate": 83.33,
#   "total_pnl": 2150.00,
#   "total_return_pct": 21.5,
#   "avg_pnl": 51.19,
#   "max_pnl": 250.00,
#   "min_pnl": -150.00,
#   "max_drawdown": 300.00,
#   "final_balance": 12150.00
# }
```

## JavaScript/Node.js Examples

### Fetch from Trading API

```javascript
// Get all strategies
async function getStrategies() {
    const response = await fetch('http://localhost:3000/api/trading/strategies/list');
    const strategies = await response.json();
    console.log('Active Strategies:', strategies);
    return strategies;
}

// Create new strategy
async function createStrategy(name, type, symbol) {
    const response = await fetch('http://localhost:3000/api/trading/strategies', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
            name,
            strategy_type: type,
            symbol,
            risk_percent: 2.0,
            stop_loss_pct: 2.0,
            take_profit_pct: 5.0
        })
    });
    const result = await response.json();
    console.log('Strategy Created:', result);
    return result;
}

// Get signals
async function getSignals() {
    const response = await fetch('http://localhost:3000/api/trading/signals');
    const signals = await response.json();
    signals.forEach(sig => {
        console.log(`${sig.symbol}: ${sig.signal_type} (${(sig.confidence*100).toFixed(1)}%)`);
    });
    return signals;
}

// Run backtest
async function runBacktest(strategyId, symbol) {
    const response = await fetch('http://localhost:3000/api/trading/backtest', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
            strategy_id: strategyId,
            symbol,
            initial_balance: 10000.0
        })
    });
    const result = await response.json();
    console.log('Backtest Results:', result);
    return result;
}

// Usage
(async () => {
    const strategies = await getStrategies();
    const signals = await getSignals();
    const newStrat = await createStrategy('My Strategy', 'rsi_momentum', 'BTCUSDT');
    const backtest = await runBacktest(newStrat.id, 'BTCUSDT');
})();
```

### Real-time Signal Monitoring

```javascript
// Setup signal monitoring
async function monitorSignals(updateInterval = 5000) {
    setInterval(async () => {
        try {
            const response = await fetch('http://localhost:3000/api/trading/signals');
            const signals = await response.json();
            
            // Check for strong signals
            signals.forEach(signal => {
                if (signal.confidence > 0.7) {
                    console.warn(`⚠️ STRONG SIGNAL: ${signal.signal_type} on ${signal.symbol}`);
                    console.log(`   Confidence: ${(signal.confidence*100).toFixed(1)}%`);
                    console.log(`   RSI: ${signal.indicators.rsi?.toFixed(2)}`);
                    
                    // Send notification (email, SMS, Discord, etc.)
                    sendNotification(signal);
                }
            });
        } catch (error) {
            console.error('Error fetching signals:', error);
        }
    }, updateInterval);
}

function sendNotification(signal) {
    // Implement your notification method
    console.log('NOTIFICATION SENT');
}

// Start monitoring
monitorSignals(5000);  // Check every 5 seconds
```

## Python Examples

### Using Python Requests

```python
import requests
import json
from datetime import datetime

BASE_URL = 'http://localhost:3000/api/trading'

class TradingClient:
    def __init__(self, base_url=BASE_URL):
        self.base_url = base_url
        self.session = requests.Session()
    
    def create_strategy(self, name, strategy_type, symbol, **kwargs):
        """Create a new trading strategy"""
        data = {
            'name': name,
            'strategy_type': strategy_type,
            'symbol': symbol,
            **kwargs
        }
        response = self.session.post(
            f'{self.base_url}/strategies',
            json=data
        )
        return response.json()
    
    def list_strategies(self):
        """Get all strategies"""
        response = self.session.get(f'{self.base_url}/strategies/list')
        return response.json()
    
    def get_signals(self):
        """Get current trading signals"""
        response = self.session.get(f'{self.base_url}/signals')
        return response.json()
    
    def run_backtest(self, strategy_id, symbol, initial_balance=10000.0):
        """Run backtest for a strategy"""
        data = {
            'strategy_id': strategy_id,
            'symbol': symbol,
            'initial_balance': initial_balance
        }
        response = self.session.post(
            f'{self.base_url}/backtest',
            json=data
        )
        return response.json()

# Usage
client = TradingClient()

# Create strategies
rsi_strat = client.create_strategy(
    name='RSI Strategy',
    strategy_type='rsi_momentum',
    symbol='BTCUSDT',
    risk_percent=2.0,
    stop_loss_pct=2.0,
    take_profit_pct=5.0
)
print(f"Created: {rsi_strat['name']}")

# Get signals
signals = client.get_signals()
for sig in signals:
    print(f"{sig['symbol']}: {sig['signal_type']} ({sig['confidence']*100:.1f}%)")

# Run backtest
backtest_result = client.run_backtest(rsi_strat['id'], 'BTCUSDT')
print(f"Win Rate: {backtest_result['win_rate']:.2f}%")
print(f"Total P&L: ${backtest_result['total_pnl']:.2f}")
```

## Docker Deployment

### Dockerfile

```docker
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/backend /usr/local/bin/
EXPOSE 3000
CMD ["backend"]
```

### docker-compose.yml

```yaml
version: '3.8'
services:
  trading-api:
    build: ./backend
    ports:
      - "3000:3000"
    environment:
      - RUST_LOG=info
    volumes:
      - ./data:/data
    restart: always
```

Run with:
```bash
docker-compose up -d
```

## Performance Tips

1. **Backtest Optimization**:
   - Use smaller date ranges for quick tests
   - Start with one strategy before running multiple
   - Cache historical data to avoid re-fetching

2. **Signal Generation**:
   - Run signal generation on fixed intervals (not every tick)
   - Multi-timeframe analysis increases accuracy but adds latency
   - Cache last signals to reduce API calls

3. **Live Trading**:
   - Start with small position sizes (1-2% risk)
   - Monitor slippage vs backtested results
   - Use stop losses religiously

---

**Ready to build your trading bot! Start with backtesting, then move to live trading.**
