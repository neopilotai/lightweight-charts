import React, { useState } from 'react'
import '../styles/MarketDepth.css'

export default function MarketDepth({ symbol }) {
  const [depth, setDepth] = useState({
    bids: [
      { price: 43450, amount: 1.5, percentage: 75 },
      { price: 43440, amount: 2.3, percentage: 60 },
      { price: 43430, amount: 3.1, percentage: 45 },
      { price: 43420, amount: 4.2, percentage: 30 },
      { price: 43410, amount: 5.1, percentage: 15 },
    ],
    asks: [
      { price: 43460, amount: 5.2, percentage: 20 },
      { price: 43470, amount: 4.1, percentage: 35 },
      { price: 43480, amount: 3.3, percentage: 50 },
      { price: 43490, amount: 2.5, percentage: 65 },
      { price: 43500, amount: 1.8, percentage: 80 },
    ],
  })

  return (
    <div className="market-depth">
      <h3>Market Depth - {symbol.toUpperCase()}</h3>
      
      <div className="depth-container">
        <div className="depth-side bids">
          <div className="depth-header">Bids (Buy)</div>
          {depth.bids.map((bid, idx) => (
            <div key={idx} className="depth-row">
              <div className="depth-bar" style={{ width: `${bid.percentage}%` }}></div>
              <span className="depth-price">${bid.price}</span>
              <span className="depth-amount">{bid.amount}</span>
            </div>
          ))}
        </div>

        <div className="depth-side asks">
          <div className="depth-header">Asks (Sell)</div>
          {depth.asks.map((ask, idx) => (
            <div key={idx} className="depth-row">
              <span className="depth-amount">{ask.amount}</span>
              <span className="depth-price">${ask.price}</span>
              <div className="depth-bar" style={{ width: `${ask.percentage}%` }}></div>
            </div>
          ))}
        </div>
      </div>
    </div>
  )
}
