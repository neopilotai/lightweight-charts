import React, { useState, useEffect } from 'react'
import '../styles/MarketDepth.css'

export default function MarketDepth({ symbol }) {
  const [depth, setDepth] = useState({ bids: [], asks: [] })
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    const fetchOrderBook = async () => {
      try {
        const response = await fetch(`/api/orderbook?symbol=${symbol}`)
        const data = await response.json()
        
        const maxQty = Math.max(
          ...data.bids.map(b => b.quantity),
          ...data.asks.map(a => a.quantity),
          1
        )
        
        setDepth({
          bids: data.bids.map(b => ({
            price: b.price,
            amount: b.quantity,
            percentage: (b.quantity / maxQty) * 100
          })),
          asks: data.asks.map(a => ({
            price: a.price,
            amount: a.quantity,
            percentage: (a.quantity / maxQty) * 100
          }))
        })
        setLoading(false)
      } catch (error) {
        console.error('Error fetching orderbook:', error)
        setLoading(false)
      }
    }

    fetchOrderBook()
    const interval = setInterval(fetchOrderBook, 2000)
    return () => clearInterval(interval)
  }, [symbol])

  if (loading) {
    return <div className="market-depth"><h3>Market Depth - {symbol.toUpperCase()}</h3><p>Loading...</p></div>
  }

  return (
    <div className="market-depth">
      <h3>Market Depth - {symbol.toUpperCase()}</h3>
      
      <div className="depth-container">
        <div className="depth-side bids">
          <div className="depth-header">Bids (Buy)</div>
          {depth.bids.map((bid, idx) => (
            <div key={idx} className="depth-row">
              <div className="depth-bar" style={{ width: `${bid.percentage}%` }}></div>
              <span className="depth-price">${bid.price.toFixed(2)}</span>
              <span className="depth-amount">{bid.amount.toFixed(4)}</span>
            </div>
          ))}
        </div>

        <div className="depth-side asks">
          <div className="depth-header">Asks (Sell)</div>
          {depth.asks.map((ask, idx) => (
            <div key={idx} className="depth-row">
              <span className="depth-amount">{ask.amount.toFixed(4)}</span>
              <span className="depth-price">${ask.price.toFixed(2)}</span>
              <div className="depth-bar" style={{ width: `${ask.percentage}%` }}></div>
            </div>
          ))}
        </div>
      </div>
    </div>
  )
}
