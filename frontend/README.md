# Trading Dashboard UI

Professional React trading UI with lightweight-charts integration, real-time WebSocket updates, and comprehensive strategy management.

## Features

✨ **Pro Chart Interface**
- Real-time candlestick charts with lightweight-charts
- Multiple technical indicators (RSI, MACD, EMA)
- Configurable timeframes (1m, 5m, 15m, 1h, 4h, 1d)
- Responsive, professional dark theme

📊 **Trading Signals**
- Live trading signals with confidence levels
- Multiple indicator sources
- Entry price and strategy type display

💼 **Portfolio Analytics**
- Real-time P&L tracking
- Win rate and drawdown metrics
- Daily/monthly return statistics

🎯 **Strategy Management**
- Create and manage trading strategies
- Configure risk parameters
- Enable/disable strategies on the fly
- Multiple strategy types support

🌐 **Real-time Updates**
- WebSocket integration for live data
- Automatic chart updates
- Live signal generation

## Setup

### Prerequisites
- Node.js 16+
- Rust backend running on `http://localhost:3000`

### Installation

```bash
# Install dependencies
npm install

# Start development server
npm run dev

# Build for production
npm run build

# Preview production build
npm run preview
```

The dev server runs on `http://localhost:5173`.

## Project Structure

```
frontend/
├── src/
│   ├── components/
│   │   ├── App.jsx              # Main app component
│   │   ├── ChartPanel.jsx       # Chart with indicators
│   │   ├── SymbolSelector.jsx   # Trading pair selector
│   │   ├── SignalsPanel.jsx     # Trading signals display
│   │   ├── PortfolioStatsPanel.jsx  # P&L and stats
│   │   └── StrategyPanel.jsx    # Strategy management
│   ├── services/
│   │   └── api.js               # Backend API client
│   ├── styles/
│   │   └── *.css                # Component & global styles
│   ├── App.jsx                  # Root component
│   └── main.jsx                 # Entry point
├── package.json
├── vite.config.js               # Vite configuration
└── index.html                   # HTML template
```

## API Integration

The UI connects to the REST API on `http://localhost:3000`:

| Endpoint | Description |
|----------|-------------|
| `GET /api/candles?symbol=btcusdt` | Fetch candlestick data with indicators |
| `GET /api/trading/signals?symbol=btcusdt` | Get current trading signals |
| `GET /api/trading/strategies/list` | List all strategies |
| `POST /api/trading/strategies` | Create new strategy |
| `PUT /api/trading/strategies/:id/toggle` | Enable/disable strategy |
| `DELETE /api/trading/strategies/:id` | Delete strategy |
| `WS /ws?symbol=btcusdt` | WebSocket for real-time data |

## WebSocket Connection

The chart updates automatically via WebSocket connection to the backend:

```javascript
// Automatically connects to ws://localhost:3000/ws
// Receives real-time candle updates
// Updates chart and indicators automatically
```

## Styling

The UI uses a professional dark theme with:
- Consistent color scheme
- Responsive grid layout
- Mobile-friendly design
- CSS custom variables for easy theming
- Smooth animations and transitions

### Theme Variables

Edit CSS variables in `src/styles/index.css`:

```css
--primary-dark: #1e1e1e;
--color-up: #26a69a;
--color-down: #ef5350;
--color-accent: #2196f3;
/* ... more variables ... */
```

## Responsive Design

The UI is fully responsive:
- **Desktop (> 1400px)**: Full layout with sidebar
- **Tablet (1024px - 1400px)**: Compact sidebar layout
- **Mobile (< 1024px)**: Stacked layout

## Performance Optimizations

- Lazy component loading
- Optimized re-renders
- Efficient WebSocket handling
- Debounced resize events
- Memoized API calls

## Browser Support

- Chrome/Chromium 90+
- Firefox 88+
- Safari 14+
- Edge 90+

## Development

```bash
# Install dependencies
npm install

# Start dev server with HMR
npm run dev

# Build production bundle
npm run build

# Check bundle size
npm run build -- --analyze
```

## Troubleshooting

### WebSocket Connection Failed
- Ensure backend is running on `http://localhost:3000`
- Check CORS is enabled on backend
- Verify WebSocket proxy in `vite.config.js`

### API Calls Returning 404
- Verify backend endpoints match expected routes
- Check backend is running correctly
- Verify symbols are lowercase (btcusdt, ethusdt, etc.)

### Chart Not Displaying
- Check DevTools for JavaScript errors
- Verify lightweight-charts library is loaded
- Check container height is not zero

## License

MIT
