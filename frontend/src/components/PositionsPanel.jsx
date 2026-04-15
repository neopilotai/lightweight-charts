import React, { useState, useEffect } from 'react'
import '../styles/OrderPanel.css'

export default function PositionsPanel() {
  const [positions, setPositions] = useState([])
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    fetchPositions()
  }, [])

  const fetchPositions = async () => {
    try {
      // Mock data for now - replace with actual API call
      setPositions([
        {
          id: 1,
          symbol: 'BTCUSDT',
          side: 'long',
          quantity: 0.5,
          entryPrice: 43500,
          currentPrice: 44200,
          pnl: 350,
          pnlPercent: 1.61,
          openTime: '2h ago',
        },
        {
          id: 2,
          symbol: 'ETHUSDT',
          side: 'short',
          quantity: 5,
          entryPrice: 2250,
          currentPrice: 2240,
          pnl: 50,
          pnlPercent: 0.22,
          openTime: '30m ago',
        },
      ])
    } catch (error) {
      console.error('Error fetching positions:', error)
    } finally {
      setLoading(false)
    }
  }

  return (
    <div className="positions-panel">
      <h3>Open Positions ({positions.length})</h3>

      {loading ? (
        <p className="loading">Loading positions...</p>
      ) : positions.length === 0 ? (
        <p className="no-positions">No open positions</p>
      ) : (
        <div className="positions-list">
          {positions.map(position => (
            <div key={position.id} className={`position-item ${position.side}`}>
              <div className="position-header">
                <span className="symbol">{position.symbol}</span>
                <span className={`pnl ${position.pnl >= 0 ? 'positive' : 'negative'}`}>
                  {position.pnl >= 0 ? '+' : ''}{position.pnl.toFixed(2)}
                </span>
              </div>
              <div className="position-details">
                <div className="detail-row">
                  <span className="label">Size:</span>
                  <span className="value">{position.quantity}</span>
                </div>
                <div className="detail-row">
                  <span className="label">Entry:</span>
                  <span className="value">${position.entryPrice.toFixed(2)}</span>
                </div>
                <div className="detail-row">
                  <span className="label">Current:</span>
                  <span className="value">${position.currentPrice.toFixed(2)}</span>
                </div>
                <div className="detail-row">
                  <span className="label">Return:</span>
                  <span className={`value ${position.pnlPercent >= 0 ? 'positive' : 'negative'}`}>
                    {position.pnlPercent >= 0 ? '+' : ''}{position.pnlPercent.toFixed(2)}%
                  </span>
                </div>
              </div>
              <button className="close-btn">Close</button>
            </div>
          ))}
        </div>
      )}
    </div>
  )
}
