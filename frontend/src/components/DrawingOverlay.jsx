// src/components/DrawingOverlay.jsx
import React, { useRef, useEffect, useState, useCallback } from 'react'

const STORAGE_KEY = 'trading-drawings'

const DRAWING_TYPES = {
  TRENDLINE: 'trendline',
  HORIZONTAL: 'horizontal',
  FIBONACCI: 'fibonacci',
}

function loadDrawings() {
  try {
    const saved = localStorage.getItem(STORAGE_KEY)
    return saved ? JSON.parse(saved) : {}
  } catch {
    return {}
  }
}

function saveDrawings(drawings) {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(drawings))
  } catch (e) {
    console.warn('Failed to save drawings:', e)
  }
}

export default function DrawingOverlay({ chart, containerRef, symbol, timeframe }) {
  const canvasRef = useRef(null)
  const [isDrawing, setIsDrawing] = useState(false)
  const [drawingType, setDrawingType] = useState(DRAWING_TYPES.HORIZONTAL)
  const [drawings, setDrawings] = useState(() => loadDrawings())
  const [selectedDrawing, setSelectedDrawing] = useState(null)
  const [startPoint, setStartPoint] = useState(null)
  const key = `${symbol}:${timeframe}`

  const getCanvasPoint = useCallback((e) => {
    const canvas = canvasRef.current
    const rect = canvas.getBoundingClientRect()
    const x = e.clientX - rect.left
    const y = e.clientY - rect.top
    const time = chart?.timeScale().coordinateToTime(x)
    const price = chart?.priceScale().coordinateToPrice(y)
    return { x, y, time, price }
  }, [chart])

  const renderDrawings = useCallback(() => {
    const canvas = canvasRef.current
    if (!canvas || !chart) return
    const ctx = canvas.getContext('2d')
    const { width, height } = canvas
    
    ctx.clearRect(0, 0, width, height)
    
    const symbolDrawings = drawings[key] || []
    
    symbolDrawings.forEach((d, i) => {
      ctx.beginPath()
      ctx.lineWidth = d.type === DRAWING_TYPES.HORIZONTAL ? 1 : 2
      ctx.strokeStyle = d.selected ? '#FFD700' : (d.type === DRAWING_TYPES.FIBONACCI ? '#00BCD4' : '#4CAF50')
      
      if (d.type === DRAWING_TYPES.HORIZONTAL) {
        const price = d.price
        const y = chart.priceScale().priceToCoordinate(price)
        ctx.moveTo(0, y)
        ctx.lineTo(width, y)
      } else if (d.type === DRAWING_TYPES.TRENDLINE) {
        const x1 = chart.timeScale().timeToCoordinate(d.startTime)
        const y1 = chart.priceScale().priceToCoordinate(d.startPrice)
        const x2 = chart.timeScale().timeToCoordinate(d.endTime)
        const y2 = chart.priceScale().priceToCoordinate(d.endPrice)
        ctx.moveTo(x1, y1)
        ctx.lineTo(x2, y2)
      } else if (d.type === DRAWING_TYPES.FIBONACCI) {
        const levels = [0, 0.236, 0.382, 0.5, 0.618, 0.786, 1]
        const base = d.high
        const range = d.low - d.high
        levels.forEach((lvl, idx) => {
          const price = base + range * lvl
          const y = chart.priceScale().priceToCoordinate(price)
          ctx.moveTo(0, y)
          ctx.lineTo(width, y)
          ctx.fillStyle = ctx.strokeStyle
          ctx.fillText(`${(lvl * 100).toFixed(1)}%`, 5, y - 2)
        })
      }
      ctx.stroke()
    })
  }, [chart, drawings, key])

  useEffect(() => {
    const canvas = canvasRef.current
    if (!canvas || !containerRef.current) return
    
    const resize = () => {
      canvas.width = containerRef.current.clientWidth
      canvas.height = containerRef.current.clientHeight
      renderDrawings()
    }
    
    resize()
    window.addEventListener('resize', resize)
    return () => window.removeEventListener('resize', resize)
  }, [containerRef, renderDrawings])

  useEffect(() => {
    renderDrawings()
  }, [renderDrawings])

  useEffect(() => {
    const unsubscribe = chart?.subscribeCrosshairMove(param => {
      if (param.time && param.seriesData) {
        // Crosshair data available
      }
    })
    return () => unsubscribe?.()
  }, [chart])

  const handleMouseDown = (e) => {
    if (!chart) return
    const point = getCanvasPoint(e)
    setStartPoint(point)
    setIsDrawing(true)
  }

  const handleMouseMove = (e) => {
    if (!isDrawing || !startPoint || !chart) return
    
    const point = getCanvasPoint(e)
    const canvas = canvasRef.current
    const ctx = canvas.getContext('2d')
    const { width } = canvas
    
    ctx.clearRect(0, 0, width, canvas.height)
    
    // Redraw existing
    const symbolDrawings = drawings[key] || []
    symbolDrawings.forEach(d => {
      ctx.beginPath()
      ctx.lineWidth = 1
      ctx.strokeStyle = d.type === DRAWING_TYPES.FIBONACCI ? '#00BCD4' : '#4CAF50'
      
      if (d.type === DRAWING_TYPES.HORIZONTAL) {
        const y = chart.priceScale().priceToCoordinate(d.price)
        ctx.moveTo(0, y)
        ctx.lineTo(width, y)
      } else if (d.type === DRAWING_TYPES.TRENDLINE) {
        const x1 = chart.timeScale().timeToCoordinate(d.startTime)
        const y1 = chart.priceScale().priceToCoordinate(d.startPrice)
        const x2 = chart.timeScale().timeToCoordinate(d.endTime)
        const y2 = chart.priceScale().priceToCoordinate(d.endPrice)
        ctx.moveTo(x1, y1)
        ctx.lineTo(x2, y2)
      }
      ctx.stroke()
    })
    
    // Draw preview
    ctx.beginPath()
    ctx.strokeStyle = '#FFD700'
    ctx.setLineDash([5, 5])
    if (drawingType === DRAWING_TYPES.HORIZONTAL) {
      const y = chart.priceScale().priceToCoordinate(point.price)
      ctx.moveTo(0, y)
      ctx.lineTo(width, y)
    } else if (drawingType === DRAWING_TYPES.TRENDLINE) {
      const x1 = chart.timeScale().timeToCoordinate(startPoint.time)
      const y1 = chart.priceScale().priceToCoordinate(startPoint.price)
      const x2 = point.x
      const y2 = point.y
      ctx.moveTo(x1, y1)
      ctx.lineTo(x2, y2)
    }
    ctx.stroke()
    ctx.setLineDash([])
  }

  const handleMouseUp = (e) => {
    if (!isDrawing || !startPoint || !chart) {
      setIsDrawing(false)
      return
    }
    
    const point = getCanvasPoint(e)
    const newDrawing = {
      id: Date.now().toString(),
      type: drawingType,
      price: point.price,
      startTime: startPoint.time,
      startPrice: startPoint.price,
      endTime: point.time,
      endPrice: point.price,
    }
    
    if (drawingType === DRAWING_TYPES.FIBONACCI) {
      newDrawing.high = Math.max(startPoint.price, point.price)
      newDrawing.low = Math.min(startPoint.price, point.price)
    }
    
    const updated = { ...drawings }
    updated[key] = [...(updated[key] || []), newDrawing]
    setDrawings(updated)
    saveDrawings(updated)
    
    setIsDrawing(false)
    setStartPoint(null)
    renderDrawings()
  }

  const clearDrawings = () => {
    const updated = { ...drawings }
    delete updated[key]
    setDrawings(updated)
    saveDrawings(updated)
    renderDrawings()
  }

  return (
    <div className="drawing-toolbar">
      <select value={drawingType} onChange={(e) => setDrawingType(e.target.value)}>
        <option value={DRAWING_TYPES.HORIZONTAL}>Horizontal Line</option>
        <option value={DRAWING_TYPES.TRENDLINE}>Trendline</option>
        <option value={DRAWING_TYPES.FIBONACCI}>Fibonacci</option>
      </select>
      <button onClick={clearDrawings}>Clear</button>
      <span className="drawing-hint">Draw: Click and drag on chart</span>
      <canvas
        ref={canvasRef}
        style={{ position: 'absolute', top: 0, left: 0, pointerEvents: 'auto', zIndex: 10 }}
        onMouseDown={handleMouseDown}
        onMouseMove={handleMouseMove}
        onMouseUp={handleMouseUp}
        onMouseLeave={() => { if (isDrawing) setIsDrawing(false) }}
      />
    </div>
  )
}