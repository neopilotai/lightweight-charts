import React, { useState, useEffect, useRef } from 'react'
import { createChart } from 'lightweight-charts'

export default function ReplayChart({ symbol, interval = '1h' }) {
  const containerRef = useRef(null)
  const chartRef = useRef(null)
  const candleSeriesRef = useRef(null)
  
  const [candleData, setCandleData] = useState([])
  const [currentIndex, setCurrentIndex] = useState(0)
  const [isPlaying, setIsPlaying] = useState(false)
  const [playbackSpeed, setPlaybackSpeed] = useState(1000)
  const [visibleRange, setVisibleRange] = useState({ start: 0, end: 50 })
  
  useEffect(() => {
    const fetchData = async () => {
      try {
        const response = await fetch(`/api/candles?symbol=${symbol}&interval=${interval}`)
        const data = await response.json()
        setCandleData(data)
      } catch (error) {
        console.error('Error fetching data:', error)
      }
    }
    fetchData()
  }, [symbol, interval])

  useEffect(() => {
    if (!containerRef.current || candleData.length === 0) return

    const chart = createChart(containerRef.current, {
      layout: { textColor: '#DDD', background: { color: '#1e1e1e' } },
      width: containerRef.current.clientWidth,
      height: 400,
      timeScale: { timeVisible: true, secondsVisible: false },
    })

    const candleSeries = chart.addCandlestickSeries({
      upColor: '#26a69a', downColor: '#ef5350',
      borderDownColor: '#ef5350', borderUpColor: '#26a69a',
      wickDownColor: '#ef5350', wickUpColor: '#26a69a',
    })

    chartRef.current = chart
    candleSeriesRef.current = candleSeries

    const handleResize = () => {
      if (containerRef.current) {
        chart.applyOptions({ width: containerRef.current.clientWidth })
      }
    }
    window.addEventListener('resize', handleResize)

    return () => {
      window.removeEventListener('resize', handleResize)
      chart.remove()
    }
  }, [candleData.length])

  useEffect(() => {
    if (!candleSeriesRef.current || candleData.length === 0) return
    
    const visibleData = candleData.slice(visibleRange.start, visibleRange.end)
    const dataToShow = visibleData.slice(0, currentIndex + 1)
    candleSeriesRef.current.setData(dataToShow.map(d => ({
      time: d.time,
      open: d.open, high: d.high, low: d.low, close: d.close,
    })))
    chartRef.current?.timeScale().fitContent()
  }, [currentIndex, visibleRange, candleData])

  useEffect(() => {
    if (!isPlaying || currentIndex >= candleData.length - 1) {
      setIsPlaying(false)
      return
    }
    
    const timer = setInterval(() => {
      setCurrentIndex(prev => {
        if (prev >= candleData.length - 1) {
          setIsPlaying(false)
          return prev
        }
        return prev + 1
      })
    }, playbackSpeed)
    
    return () => clearInterval(timer)
  }, [isPlaying, playbackSpeed, candleData.length])

  const handlePlay = () => {
    if (currentIndex >= candleData.length - 1) {
      setCurrentIndex(0)
    }
    setIsPlaying(true)
  }

  const handlePause = () => setIsPlaying(false)

  const handleReset = () => {
    setIsPlaying(false)
    setCurrentIndex(0)
  }

  const handleStep = (direction) => {
    setIsPlaying(false)
    const newIndex = direction === 'forward' 
      ? Math.min(currentIndex + 1, candleData.length - 1)
      : Math.max(currentIndex - 1, 0)
    setCurrentIndex(newIndex)
  }

  const handleSpeedChange = (speed) => {
    setPlaybackSpeed(speed)
    setIsPlaying(true)
  }

  const totalCandles = candleData.length
  const currentDate = candleData[currentIndex] 
    ? new Date(candleData[currentIndex].time * 1000).toLocaleString() 
    : ''

  return (
    <div className="replay-chart">
      <div className="replay-controls">
        <button onClick={() => handleStep('backward')}>⏮</button>
        {isPlaying ? (
          <button onClick={handlePause}>⏸</button>
        ) : (
          <button onClick={handlePlay}>▶</button>
        )}
        <button onClick={() => handleStep('forward')}>⏭</button>
        <button onClick={handleReset}>⏹</button>
        
        <select value={playbackSpeed} onChange={(e) => handleSpeedChange(Number(e.target.value))}>
          <option value={2000}>0.5x</option>
          <option value={1000}>1x</option>
          <option value={500}>2x</option>
          <option value={200}>5x</option>
        </select>

        <span className="replay-info">
          {currentIndex + 1} / {totalCandles} | {currentDate}
        </span>
      </div>
      <div ref={containerRef} className="replay-container" />
    </div>
  )
}