import React, { useState, useEffect } from 'react'
import '../styles/TradesPanel.css'

export default function TradesPanel({ symbol }) {
  const [trades, setTrades] = useState([])
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    const fetchTrades = async () => {
      try {
        const response = await fetch(`/api/trades?symbol=${symbol}`)
        const data = await response.json()
        setTrades(data.slice(0, 20))
        setLoading(false)
      } catch (error) {
        console.error('Error fetching trades:', error)
        setLoading(false)
      }
    }

    fetchTrades()
    const interval = setInterval(fetchTrades, 3000)
    return () => clearInterval(interval)
  }, [symbol])

  const formatTime = (timestamp) => {
    const date = new Date(timestamp * 1000)
    return date.toLocaleTimeString()
  }

  if (loading) {
    return <div className="trades-panel"><h3>Recent Trades</h3><p>Loading...</p></div>
  }

  return (
    <div className="trades-panel">
      <h3>Recent Trades - {symbol.toUpperCase()}</h3>
      <div className="trades-header">
        <span>Price</span>
        <span>Amount</span>
        <span>Time</span>
      </div>
      <div className="trades-list">
        {trades.map((trade, idx) => (
          <div key={trade.id || idx} className={`trade-row ${trade.is_buyer_maker ? 'sell' : 'buy'}`}>
            <span className={`trade-price ${trade.is_buyer_maker ? 'down' : 'up'}`}>
              ${trade.price.toFixed(2)}
            </span>
            <span className="trade-amount">{trade.quantity.toFixed(4)}</span>
            <span className="trade-time">{formatTime(trade.time)}</span>
          </div>
        ))}
      </div>
    </div>
  )
}