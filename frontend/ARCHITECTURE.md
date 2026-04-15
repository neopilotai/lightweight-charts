# Trading Dashboard Architecture

## System Overview

```
┌─────────────────────────────────────────────────────────────┐
│                     Trading Dashboard UI                     │
│                   (React + Vite + Lightweight Charts)         │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐       │
│  │   ChartPanel │  │   Signals    │  │  Portfolio   │       │
│  │   (Main)     │  │   Display    │  │   Stats      │       │
│  └──────────────┘  └──────────────┘  └──────────────┘       │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐       │
│  │   Symbol     │  │   Strategy   │  │   Optional   │       │
│  │   Selector   │  │   Manager    │  │   Panels     │       │
│  └──────────────┘  └──────────────┘  └──────────────┘       │
│                                                              │
│                    ↓ REST API / WebSocket                    │
└──────────────────────┬──────────────────────────────────────┘
                       │
┌──────────────────────┴──────────────────────────────────────┐
│                  Rust Backend (Axum)                        │
│                  http://localhost:3000                       │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐       │
│  │   Market     │  │   Trading    │  │   WebSocket  │       │
│  │   Routes     │  │   Routes     │  │   Handler    │       │
│  └──────────────┘  └──────────────┘  └──────────────┘       │
│                                                              │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐       │
│  │   Strategy   │  │   Trading    │  │   Technical  │       │
│  │   Manager    │  │   Engine     │  │   Indicators │       │
│  └──────────────┘  └──────────────┘  └──────────────┘       │
│                                                              │
│           ↓ Binance WebSocket API                            │
└──────────────────────┬──────────────────────────────────────┘
                       │
        ┌──────────────┴──────────────┐
        │                             │
   ┌────▼────┐              ┌────────▼────┐
   │  Binance │              │  Historical │
   │    API   │              │    Data     │
   │ Real-time│              │   Cache     │
   └──────────┘              └─────────────┘
```

## Component Architecture

### Frontend Components

#### `App.jsx` (Root)
- Main application container
- State management for symbols, signals, strategies
- Layout orchestration
- API integration main entry point

```
App
├── Header
│   ├── Title
│   └── SymbolSelector
├── ChartPanel
│   ├── Candlestick Series
│   ├── RSI Indicator
│   ├── MACD Indicator
│   └── Controls (Timeframes, Toggles)
└── Sidebar
    ├── SignalsPanel
    ├── PortfolioStatsPanel
    └── StrategyPanel
```

#### `ChartPanel.jsx`
- Main trading chart display
- Lightweight-charts initialization
- WebSocket real-time updates
- Indicator visualization
- Responsive canvas management

**Key Features:**
- Multi-indicator support (RSI, MACD)
- Timeframe switching (1m, 5m, 15m, 1h, 4h, 1d)
- Dynamic price scales
- Auto-fit to window
- Real-time candle updates

#### `SignalsPanel.jsx`
- Trading signal display
- Confidence score visualization
- Signal type indicators (BUY/SELL)
- Entry price suggestions
- Scrollable signal list

#### `PortfolioStatsPanel.jsx`
- Portfolio performance metrics
- Real-time P&L calculation
- Win rate display
- Monthly returns
- Maximum drawdown tracking
- 2x3 stats grid layout

#### `StrategyPanel.jsx`
- Strategy CRUD operations
- Multi-strategy management
- Strategy configuration form
  - Strategy type selection
  - Risk parameter configuration
  - Stop-loss and take-profit settings
- Enable/disable toggle
- Strategy deletion
- Real-time strategy list updates

#### `SymbolSelector.jsx`
- Trading pair selection
- Dropdown menu UI
- Icon display for pairs
- Quick symbol switching

#### `OrderPanel.jsx` (Optional)
- Order placement interface
- Order type selection (Limit, Market, Stop)
- BUY/SELL buttons
- Quantity and price inputs

#### `PositionsPanel.jsx` (Optional)
- Open positions display
- Position P&L tracking
- Close position functionality
- Position details (entry, current, return %)

#### `MarketDepth.jsx` (Optional)
- Order book visualization
- Bid/ask depth display
- Visual bar charts
- Real-time depth updates

### Backend Routes

#### Market Routes (`/api/market`)

```
GET /api/candles?symbol=btcusdt
Response: [
  {
    time: 1234567890,
    open: 43500.50,
    high: 43600.00,
    low: 43400.00,
    close: 43550.75,
    volume: 1234.56,
    rsi: 65.5,
    macd_line: 125.34,
    signal_line: 120.45,
    histogram: 4.89
  },
  ...
]
```

#### Trading Routes (`/api/trading`)

```
POST /api/trading/strategies
Body: {
  name: "RSI Momentum",
  strategy_type: "rsi_momentum",
  symbol: "BTCUSDT",
  risk_percent: 2.0,
  stop_loss_pct: 2.0,
  take_profit_pct: 5.0
}

GET /api/trading/strategies/list
Response: [
  {
    id: "uuid",
    name: "RSI Momentum",
    enabled: true,
    config: { ... }
  },
  ...
]

GET /api/trading/signals?symbol=btcusdt
Response: [
  {
    signal_type: "BUY",
    confidence: 0.85,
    indicator_source: "RSI",
    entry_price: 43500,
    timestamp: 1234567890
  },
  ...
]

PUT /api/trading/strategies/:id/toggle
DELETE /api/trading/strategies/:id
```

### WebSocket Connection

```
ws://localhost:3000/ws?symbol=btcusdt

Incoming Messages:
{
  time: 1234567890,
  open: 43500,
  high: 43600,
  low: 43400,
  close: 43550,
  volume: 1234,
  rsi: 65.5,
  macd_line: 125.34,
  histogram: 4.89
}
```

## Data Flow

### 1. Initial Load Flow
```
User Opens Dashboard
    ↓
App Component Mounts
    ↓
Fetch /api/candles
    ↓
ChartPanel Renders Data
    ↓
WebSocket Connection Established
    ↓
Real-time Updates Begin
```

### 2. Symbol Change Flow
```
User Selects Symbol
    ↓
setSelectedSymbol() Called
    ↓
useEffect Triggers
    ↓
Fetch /api/candles for New Symbol
    ↓
Fetch /api/trading/signals for New Symbol
    ↓
Chart Updates
    ↓
WebSocket URL Changes
    ↓
New Real-time Stream Starts
```

### 3. Strategy Creation Flow
```
User Clicks "+ Add"
    ↓
Form Displayed
    ↓
User Fills Form
    ↓
handleSubmit()
    ↓
POST /api/trading/strategies
    ↓
Backend Validates & Creates
    ↓
Response Returns Strategy ID
    ↓
UI Updates Strategy List
    ↓
Form Clears
```

### 4. Real-time Update Flow
```
WebSocket Message Received
    ↓
Parse JSON Data
    ↓
candleSeries.update()
    ↓
rsiSeries.update()
    ↓
macdSeries.update()
    ↓
Chart Renders New Data
    ↓
Automatic Viewport Adjustment
```

## State Management

### App-Level State
```javascript
const [selectedSymbol, setSelectedSymbol] = useState('btcusdt')
const [signals, setSignals] = useState([])
const [strategies, setStrategies] = useState([])
const [portfolioStats, setPortfolioStats] = useState(null)
```

### Component-Level State
- `ChartPanel`: timeframe, indicatorVisibility
- `StrategyPanel`: showForm, formData, loading
- `SymbolSelector`: isOpen
- `PortfolioStatsPanel`: portfolioData

## Styling Architecture

### CSS Variables (src/styles/index.css)
```css
--primary-dark: #1e1e1e
--secondary-dark: #2d2d2d
--tertiary-dark: #3a3a3a
--border-dark: #404040

--text-primary: #e0e0e0
--text-secondary: #a0a0a0

--color-up: #26a69a     (Green)
--color-down: #ef5350   (Red)
--color-accent: #2196f3 (Blue)

--spacing-xs through --spacing-xl
--font-size-xs through --font-size-xl
--border-radius-sm through --border-radius-lg
--shadow-sm through --shadow-lg
--transition-fast, --transition-base, --transition-slow
```

### Responsive Breakpoints
```css
Desktop (> 1400px)    → Full layout
Tablet (1024-1400px)  → Compact sidebar
Mobile (< 1024px)     → Stacked layout
```

## Performance Optimizations

1. **Lazy Chart Initialization**
   - Chart created only when container mounted
   - Cleanup on unmount

2. **Optimized Rerenders**
   - Component state isolated
   - Unnecessary rerenders prevented
   - Memoization used for static lists

3. **WebSocket Efficiency**
   - Single connection per symbol
   - Automatic connection switching
   - Error handling and reconnection logic

4. **API Caching**
   - Backend caches candle data
   - Reduced redundant requests
   - Historical data served from cache

5. **UI Rendering**
   - CSS transforms for animations
   - Debounced resize events
   - Virtualized lists for long signal/strategy lists

## Error Handling

### Network Errors
```javascript
try {
  const response = await fetch(url)
  const data = await response.json()
} catch (error) {
  console.error('Error:', error)
  // Show user-friendly message
}
```

### WebSocket Errors
```javascript
ws.onerror = (error) => {
  console.error('WebSocket error:', error)
  // Attempt reconnection
}

ws.onclose = () => {
  // Reconnect logic
}
```

### Form Validation
- Required field checks
- Type validation
- Range validation for percentages
- Backend validation for security

## SSL/HTTPS Deployment

For production HTTPS deployment:

1. **Update vite.config.js:**
```javascript
server: {
  proxy: {
    '/api': 'https://api.yourdomain.com',
    '/ws': {
      target: 'wss://api.yourdomain.com',
      ws: true
    }
  }
}
```

2. **Update WebSocket URL:**
```javascript
const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:'
const ws = new WebSocket(`${protocol}//api.yourdomain.com/ws?symbol=${symbol}`)
```

## Security Considerations

1. **API Validation**: Backend validates all input
2. **Rate Limiting**: Prevent spam requests
3. **CORS**: Properly configured on backend
4. **WebSocket Auth**: Consider adding token-based auth
5. **Secure Connections**: Use HTTPS/WSS in production
6. **Input Sanitization**: All user inputs validated

## Future Enhancements

1. **Advanced Features**
   - Multi-timeframe analysis
   - Custom indicators
   - Backtest runner UI
   - Order book visualization
   - Trade history

2. **Performance**
   - State management (Redux/Zustand)
   - Code splitting and lazy loading
   - Service worker for offline support
   - Data compression for WebSocket

3. **User Experience**
   - Theme customization
   - Dashboard layout customization
   - Keyboard shortcuts
   - Mobile app version

4. **Analytics**
   - Event tracking
   - Performance monitoring
   - User behavior analytics
   - Error reporting

---

**Last Updated**: April 2026
