# Trading Dashboard Quick Start Guide

## 🚀 Quick Start (5 minutes)

### 1. Install Dependencies
```bash
cd frontend
npm install
```

### 2. Start the Development Server
```bash
npm run dev
```
The UI will be available at `http://localhost:5173`

### 3. Ensure Backend is Running
Make sure your Rust backend is running on `http://localhost:3000`:
```bash
cd ../backend
cargo run --release
```

### 4. Open the Trading Dashboard
Navigate to `http://localhost:5173` in your browser.

## 📋 Features Overview

### Chart Panel (Center)
- **Real-time Candlesticks**: Live price data with lightweight-charts
- **Technical Indicators**: RSI and MACD with separate panes
- **Timeframe Selection**: 1m, 5m, 15m, 1h, 4h, 1d
- **Indicator Toggle**: Show/hide RSI and MACD
- **Dynamic Resizing**: Responsive to window changes
- **WebSocket Updates**: Real-time price updates

### Symbol Selector (Top Right)
- **Multi-Symbol Support**: BTC/USDT, ETH/USDT, SOL/USDT
- **Quick Switch**: Change symbols with one click
- **Icon Display**: Visual indicators for each pair

### Trading Signals (Right Sidebar, Top)
- **Live Signals**: BUY/SELL indicators
- **Confidence Levels**: 0-100% confidence scores
- **Entry Prices**: Recommended entry points
- **Indicator Source**: Shows which indicator generated signal

### Portfolio Stats (Right Sidebar, Middle)
- **Total Balance**: Current portfolio value
- **Daily P&L**: Profit/Loss and percentage
- **Open Positions**: Number of active trades
- **Win Rate**: Percentage of winning trades
- **Monthly Return**: Month-to-date performance
- **Max Drawdown**: Largest decline from peak

### Strategy Management (Right Sidebar, Bottom)
- **Create Strategies**: Add new trading strategies
- **Strategy Types**:
  - MA Crossover (Moving Average)
  - RSI Momentum
  - MACD
  - Multi-Indicator
- **Configuration**:
  - Risk percentage
  - Stop-loss percentage
  - Take-profit percentage
- **Enable/Disable**: Toggle strategies on/off
- **Delete**: Remove strategies

## 🔌 API Endpoints Used

The UI communicates with these backend endpoints:

```
GET  /api/candles?symbol=btcusdt         → Fetch candle data with indicators
GET  /api/trading/signals?symbol=btcusdt → Get trading signals
GET  /api/trading/strategies/list        → List all strategies
POST /api/trading/strategies             → Create new strategy
PUT  /api/trading/strategies/:id/toggle  → Enable/disable strategy
DEL  /api/trading/strategies/:id         → Delete strategy
WS   /ws?symbol=btcusdt                  → WebSocket for real-time updates
```

## 🎯 Usage Examples

### Creating a Strategy
1. Click **"+ Add"** in the Strategy panel
2. Fill in strategy details:
   - **Name**: e.g., "BTC Momentum"
   - **Type**: Select strategy type
   - **Symbol**: Choose trading pair
   - **Risk %**: 2% (typical)
   - **Stop Loss %**: 2% (max loss per trade)
   - **Take Profit %**: 5% (target profit)
3. Click **"Create Strategy"**

### Switching Between Symbols
1. Click the symbol button in the top-right (e.g., "₿ BTC/USDT")
2. Select a different pair from the dropdown
3. Chart updates automatically with new data

### Reading Signals
- 🟢 **BUY Signal**: Green indicator, ready to buy
- 🔴 **SELL Signal**: Red indicator, ready to sell
- **Confidence %**: How confident is the signal (0-100%)
- **Entry Price**: Suggested price to enter

### Analyzing Charts
1. **Candlesticks**: White/green = up, black/red = down
2. **RSI**: Oscillates 0-100, above 70 = overbought, below 30 = oversold
3. **MACD**: Histogram shows momentum, line shows trend
4. Toggle indicators on/off using checkboxes

## 🔧 Customization

### Change Theme Colors
Edit `src/styles/index.css`:
```css
--color-up: #26a69a;          /* Green for up candles */
--color-down: #ef5350;        /* Red for down candles */
--color-accent: #2196f3;      /* Blue for UI elements */
--primary-dark: #1e1e1e;      /* Main background */
```

### Adjust Chart Height
Edit `src/components/ChartPanel.jsx`:
```javascript
height: containerRef.current.clientHeight,  // Auto-fit
// OR
height: 600,  // Fixed height
```

### Add New Symbols
Edit `src/components/SymbolSelector.jsx`:
```javascript
const SYMBOLS = [
  { id: 'btcusdt', name: 'BTC/USDT', icon: '₿' },
  { id: 'bnbusdt', name: 'BNB/USDT', icon: '◆' },  // Add here
]
```

## 📊 Component Structure

```
App (Main)
├── Header
│   ├── Title
│   └── SymbolSelector
├── ChartPanel
│   ├── Timeframe Controls
│   ├── Indicator Toggles
│   └── Chart (with RSI & MACD)
└── Sidebar
    ├── SignalsPanel
    ├── PortfolioStatsPanel
    └── StrategyPanel
```

## 🌐 Responsive Design
- **Desktop (1400px+)**: Full layout with right sidebar
- **Tablet (1024-1400px)**: Compact sidebar
- **Mobile (<1024px)**: Stacked layout

## 🐛 Troubleshooting

### WebSocket Not Connecting
```
Error: WebSocket connection failed
Solution: 
- Verify backend is running on port 3000
- Check vite.config.js proxy settings
- Look for CORS errors in console
```

### Chart Not Showing Data
```
Error: Blank chart
Solution:
- Check browser DevTools → Network tab
- Verify API endpoint returns data
- Check symbol is lowercase (btcusdt)
```

### Strategy Creation Failed
```
Error: 422 Unprocessable Entity
Solution:
- Verify all required fields are filled
- Check risk/stop-loss/take-profit values
- Ensure backend is running
```

### Signals Not Updating
```
Error: No signals shown
Solution:
- Strategy must be enabled first
- Wait a few seconds for indicator calculation
- Verify dataflow with browser devtools
```

## 📱 Mobile Tips
- Use landscape orientation for better chart view
- Expand panels by tapping them
- Use shorter timeframes (1m, 5m) for better responsiveness
- Close unused panels to focus on chart

## ⚙️ Advanced Configuration

### API Base URL
If your backend is on a different host:
```javascript
// src/services/api.js
const API_BASE = 'https://your-api.com/api'
```

### WebSocket URL
```javascript
// src/components/ChartPanel.jsx
const ws = new WebSocket('wss://your-ws-server.com/ws?symbol=...')
```

### Chart Styling
See `src/components/ChartPanel.jsx` for chart configuration options:
```javascript
{
  layout: {
    textColor: '#DDD',
    background: { color: '#1e1e1e' },
  },
  // ... more options
}
```

## 📚 File Structure

```
frontend/
├── src/
│   ├── components/
│   │   ├── App.jsx                    # Root component
│   │   ├── ChartPanel.jsx             # Main chart
│   │   ├── SymbolSelector.jsx         # Symbol switcher
│   │   ├── SignalsPanel.jsx           # Signal display
│   │   ├── PortfolioStatsPanel.jsx    # Portfolio stats
│   │   ├── StrategyPanel.jsx          # Strategy manager
│   │   ├── OrderPanel.jsx             # Order placement (future)
│   │   ├── PositionsPanel.jsx         # Open positions (future)
│   │   └── MarketDepth.jsx            # Market depth (future)
│   ├── services/
│   │   └── api.js                     # API client
│   ├── styles/
│   │   ├── index.css                  # Global styles
│   │   ├── App.css                    # App layout
│   │   ├── ChartPanel.css
│   │   ├── SignalsPanel.css
│   │   ├── PortfolioStatsPanel.css
│   │   ├── StrategyPanel.css
│   │   └── ...
│   ├── App.jsx
│   └── main.jsx
├── package.json
├── vite.config.js
├── index.html
├── README.md
└── .gitignore
```

## 🚢 Production Build

```bash
# Build optimized bundle
npm run build

# Preview production build locally
npm run preview

# Deploy dist/ folder to your server
```

## 📞 Support

For issues or questions:
1. Check the troubleshooting section above
2. Review console for error messages (DevTools → Console)
3. Check backend logs for API errors
4. Verify backend endpoints match expected routes

## 🎓 Learning Resources

- [Lightweight Charts Docs](https://tradingview.github.io/lightweight-charts/)
- [React Docs](https://react.dev)
- [Vite Guide](https://vitejs.dev)
- [WebSocket API](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket)

---

**Happy Trading! 📈**
