# Lightweight Charts Trading Dashboard Roadmap

## Vision
Build a full-featured trading dashboard around Lightweight Charts by adding the trading platform features the library intentionally omits.

Lightweight Charts is a best-in-class rendering engine, not a complete trading product. This project should become the bridge between high-performance chart rendering and a real trading workflow.

---

## Strategic Goals

- Deliver a **multi-pane trading chart experience** with separate indicator panels.
- Support **real-time market data, timeframe switching, and persistent state**.
- Add **interactive indicator controls** and **drawing / annotation tools**.
- Build a **trading-ready dashboard** with order book, strategy signals, and replay mode.
- Create a foundation for **plugins and extensibility**.

---

## Phase A — Core Trading Chart Experience

### 1. Multi-pane chart layout
- Add dedicated panes for:
  - price candles
  - RSI
  - MACD / histogram
  - volume
- Keep time range and cursor sync across panes.
- Use separate Lightweight Charts instances per panel.

### 2. Timeframe switching
- Add UI for interval selection (`1m`, `5m`, `15m`, `1h`, etc.).
- Fetch candle data from backend for selected timeframe.
- Support switching without full page reload.

### 3. Volume and price overlays
- Add a volume histogram series to the price chart or a dedicated volume pane.
- Keep the chart data and indicator engine tightly synced with backend candle feeds.

### 4. State persistence
- Persist user state in `localStorage`:
  - selected symbol
  - timeframe
  - active indicators
  - panel layout
- Restore state on reload.

### 5. Crosshair / zoom sync
- Synchronize crosshair position across charts.
- Synchronize visible time range when the user zooms or scrolls.

---

## Phase B — Indicator Interactivity and Drawing Tools

### 1. Dynamic indicator controls
- Add UI for toggling indicators:
  - RSI
  - MACD
  - EMA / moving averages
  - custom indicator toggles
- Support runtime parameter updates, e.g. RSI period and MACD settings.
- Recompute and redraw indicator panels on demand.

### 2. Indicator engine improvements
- Extend backend indicator support with additional technical indicators.
- Expose indicator endpoints for frontend requests.
- Ensure indicator values are computed incrementally for performance.

### 3. Drawing and annotation tools
- Add a custom canvas overlay for drawing:
  - trendlines
  - support / resistance lines
  - Fibonacci retracements
- Track drawings as project state and persist them.
- Allow editing / deleting drawings.

### 4. Better UI controls
- Add a panel for indicator settings and chart controls.
- Provide quick buttons for common overlays and chart actions.

---

## Phase C — Trading Platform Capabilities

### 1. Order book and market depth
- Add a depth chart or order book side panel.
- Stream Binance depth data from backend.
- Display top bids and asks with quantity.

### 2. Trades feed and trade history
- Show a live trade feed for the selected symbol.
- Display recent executed trades and trade direction.

### 3. Strategy signal visualization
- Render buy/sell markers on the chart.
- Use backend strategy engine signals from existing Rust trading logic.
- Add a strategy panel showing current strategy status.

### 4. Trading controls
- Add a simple buy/sell panel for simulated or real orders.
- Connect order actions to backend trading APIs.

### 5. Session persistence and user workspace
- Persist the full workspace: chart layout, active symbols, indicators, and drawing annotations.
- Allow saving and restoring named workspace states.

---

## Phase D — Advanced Product Enhancements

### 1. Replay mode
- Implement historical replay mode like TradingView.
- Allow stepping through candles in time.
- Support play / pause and speed controls.

### 2. Multi-chart dashboard
- Add a multi-symbol dashboard with multiple charts.
- Support small multiples for BTC, ETH, SOL, or custom watchlists.

### 3. Alerts and notifications
- Add alert rules for price and indicator conditions.
- Notify users in-app when conditions are met.

### 4. Plugin architecture
- Define a plugin system for custom chart extensions.
- Allow registering new indicator modules and UI widgets.
- Keep the core chart engine stable while enabling extensibility.

### 5. Advanced analytics and reports
- Add portfolio and performance analytics.
- Add PnL tracking, position history, and trade performance dashboards.

---

## Recommended Priority

1. Multi-pane charts + synced time range
2. Timeframe switching + state persistence
3. Volume + RSI / MACD panel
4. Dynamic indicator controls
5. Drawing tools and annotations
6. Order book / trades feed
7. Strategy signal markers
8. Replay mode and multi-chart layout
9. Alerts and plugin support

---

## How to Use This Roadmap

- Use Phase A to establish the product foundation.
- Use Phase B to deliver polished chart interaction and indicator control.
- Use Phase C to turn the app into a trading dashboard.
- Use Phase D to differentiate the product with advanced UX and extensibility.

This roadmap is intentionally focused on shipping real trading value on top of Lightweight Charts, not on building another bare chart renderer.
