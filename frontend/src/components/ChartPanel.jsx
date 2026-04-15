import React, { useEffect, useRef, useState, useCallback } from 'react'
import { createChart } from 'lightweight-charts'
import '../styles/ChartPanel.css'

const TIMEFRAMES = ['1m', '5m', '15m', '1h', '4h', '1d']
const STORAGE_KEY = 'trading-dashboard-state'

function loadState() {
  try {
    const saved = localStorage.getItem(STORAGE_KEY)
    return saved ? JSON.parse(saved) : null
  } catch {
    return null
  }
}

function saveState(state) {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(state))
  } catch (e) {
    console.warn('Failed to save state:', e)
  }
}

export default function ChartPanel({ symbol }) {
  const containerRef = useRef(null)
  const chartRef = useRef(null)
  const candleSeriesRef = useRef(null)
  const volumeSeriesRef = useRef(null)
  const rsiSeriesRef = useRef(null)
  const macdSeriesRef = useRef(null)
  const signalLineRef = useRef(null)
  
  const [timeframe, setTimeframe] = useState(() => {
    const saved = loadState()
    return saved?.timeframe || '1h'
  })
  const [indicatorVisibility, setIndicatorVisibility] = useState(() => {
    const saved = loadState()
    return saved?.indicators || { rsi: true, macd: true }
  })

  // Persist state on change
  useEffect(() => {
    saveState({ symbol, timeframe, indicators: indicatorVisibility })
  }, [symbol, timeframe, indicatorVisibility])

  const loadChartData = useCallback(async (sym, tf, candleSeries, volumeSeries, rsiSeries, macdSeries, signalLine) => {
    try {
      const response = await fetch(`/api/candles?symbol=${sym}&interval=${tf}`)
      const data = await response.json()

      // Set candlestick data
      const candleData = data.map(d => ({
        time: d.time,
        open: d.open,
        high: d.high,
        low: d.low,
        close: d.close,
      }))
      candleSeries.setData(candleData)

      // Set volume data
      const volumeData = data.map(d => ({
        time: d.time,
        value: d.volume || 0,
        color: d.close >= d.open ? '#26a69a40' : '#ef535040',
      }))
      volumeSeries.setData(volumeData)

      // Set RSI data
      if (indicatorVisibility.rsi) {
        const rsiData = data
          .filter(d => d.rsi !== null)
          .map(d => ({ time: d.time, value: d.rsi }))
        rsiSeries.setData(rsiData)
      }

      // Set MACD data
      if (indicatorVisibility.macd) {
        const macdData = data
          .filter(d => d.histogram !== null)
          .map(d => ({ time: d.time, value: d.histogram }))
        macdSeries.setData(macdData)

        const signalData = data
          .filter(d => d.macd_line !== null)
          .map(d => ({ time: d.time, value: d.macd_line }))
        signalLine.setData(signalData)
      }

      chartRef.current?.timeScale().fitContent()
    } catch (error) {
      console.error('Error loading chart data:', error)
    }
  }, [indicatorVisibility])

  useEffect(() => {
    if (!containerRef.current) return

    const chart = createChart(containerRef.current, {
      layout: { textColor: '#DDD', background: { color: '#1e1e1e' } },
      width: containerRef.current.clientWidth,
      height: containerRef.current.clientHeight,
      timeScale: { timeVisible: true, secondsVisible: false },
    })

    const candleSeries = chart.addCandlestickSeries({
      upColor: '#26a69a', downColor: '#ef5350',
      borderDownColor: '#ef5350', borderUpColor: '#26a69a',
      wickDownColor: '#ef5350', wickUpColor: '#26a69a',
    })

    const volumeSeries = chart.addHistogramSeries({
      priceFormat: { type: 'volume' },
      priceScaleId: 'volume',
    })
    chart.priceScale('volume').applyOptions({ scaleMargins: { top: 0.85, bottom: 0 } })

    const rsiSeries = chart.addLineSeries({
      color: '#9c27b0', lineWidth: 2,
      priceScaleId: 'rsi', title: 'RSI (14)',
    })

    const macdSeries = chart.addHistogramSeries({
      color: '#2196F3',
      priceScaleId: 'macd', title: 'MACD Histogram',
    })

    const signalLine = chart.addLineSeries({
      color: '#FF9800', lineWidth: 1,
      priceScaleId: 'macd', title: 'MACD Signal',
    })

    // Configure price scales
    chart.priceScale('left').applyOptions({ scaleMargins: { top: 0.1, bottom: 0.1 } })
    chart.priceScale('rsi').applyOptions({ scaleMargins: { top: 0.7, bottom: 0 }, autoScale: false, fixedMin: 0, fixedMax: 100 })
    chart.priceScale('macd').applyOptions({ scaleMargins: { top: 0.85, bottom: 0 } })

    chartRef.current = chart
    candleSeriesRef.current = candleSeries
    volumeSeriesRef.current = volumeSeries
    rsiSeriesRef.current = rsiSeries
    macdSeriesRef.current = macdSeries
    signalLineRef.current = signalLine

    loadChartData(symbol, timeframe, candleSeries, volumeSeries, rsiSeries, macdSeries, signalLine)

    const handleResize = () => {
      if (containerRef.current) {
        chart.applyOptions({
          width: containerRef.current.clientWidth,
          height: containerRef.current.clientHeight,
        })
      }
    }
    window.addEventListener('resize', handleResize)

    // Crosshair/zoom sync setup (placeholder for future)
    chart.timeScale().subscribeVisibleTimeRangeChange(() => {})
    chart.subscribeCrosshairMove(() => {})

    return () => {
      window.removeEventListener('resize', handleResize)
      chart.remove()
    }
  }, [symbol, timeframe, loadChartData])

  // WebSocket for real-time updates
  useEffect(() => {
    const ws = new WebSocket(`ws://localhost:3000/ws?symbol=${symbol}`)
    ws.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data)
        candleSeriesRef.current?.update({ time: data.time, open: data.open, high: data.high, low: data.low, close: data.close })
        volumeSeriesRef.current?.update({ time: data.time, value: data.volume || 0, color: data.close >= data.open ? '#26a69a40' : '#ef535040' })
        rsiSeriesRef.current?.data.rsi !== null && rsiSeriesRef.current?.update({ time: data.time, value: data.rsi })
        if (macdSeriesRef.current && data.histogram !== null) {
          macdSeriesRef.current.update({ time: data.time, value: data.histogram })
          signalLineRef.current?.update({ time: data.time, value: data.macd_line })
        }
      } catch (error) {
        console.error('WebSocket error:', error)
      }
    }
    return () => ws.close()
  }, [symbol])

  const toggleIndicator = (indicator) => {
    setIndicatorVisibility(prev => ({ ...prev, [indicator]: !prev[indicator] }))
  }

  return (
    <div className="chart-panel">
      <div className="chart-header">
        <div className="chart-title">Price Chart - {symbol.toUpperCase()}</div>
        <div className="chart-controls">
          <div className="timeframe-selector">
            {TIMEFRAMES.map(tf => (
              <button key={tf} className={`tf-btn ${timeframe === tf ? 'active' : ''}`} onClick={() => setTimeframe(tf)}>
                {tf}
              </button>
            ))}
          </div>
          <div className="indicator-toggles">
            <label className="toggle-label">
              <input type="checkbox" checked={indicatorVisibility.rsi} onChange={() => toggleIndicator('rsi')} />
              RSI
            </label>
            <label className="toggle-label">
              <input type="checkbox" checked={indicatorVisibility.macd} onChange={() => toggleIndicator('macd')} />
              MACD
            </label>
          </div>
        </div>
      </div>
      <div className="chart-container-inner" ref={containerRef} />
    </div>
  )
}