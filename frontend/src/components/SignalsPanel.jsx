import React from 'react'
import '../styles/SignalsPanel.css'

export default function SignalsPanel({ signals }) {
  if (!signals || signals.length === 0) {
    return (
      <div className="signals-panel-content">
        <h3>Trading Signals</h3>
        <div className="no-signals">
          <p>No active signals</p>
        </div>
      </div>
    )
  }

  return (
    <div className="signals-panel-content">
      <h3>Trading Signals ({signals.length})</h3>
      <div className="signals-list">
        {signals.map((signal, idx) => (
          <div key={idx} className={`signal-item signal-${signal.signal_type?.toLowerCase()}`}>
            <div className="signal-header">
              <span className="signal-type">
                {signal.signal_type === 'BUY' ? '🟢' : signal.signal_type === 'SELL' ? '🔴' : '⚪'}
                {signal.signal_type}
              </span>
              <span className="signal-confidence">
                {(signal.confidence * 100).toFixed(0)}%
              </span>
            </div>
            <div className="signal-details">
              <p className="signal-indicator">{signal.indicator_source}</p>
              <p className="signal-price">
                @ ${signal.entry_price?.toFixed(2) || 'N/A'}
              </p>
            </div>
          </div>
        ))}
      </div>
    </div>
  )
}
