import React, { useState } from 'react'
import '../styles/SymbolSelector.css'

const SYMBOLS = [
  { id: 'btcusdt', name: 'BTC/USDT', icon: '₿' },
  { id: 'ethusdt', name: 'ETH/USDT', icon: 'Ξ' },
  { id: 'solusdt', name: 'SOL/USDT', icon: '◆' },
]

export default function SymbolSelector({ selected, onSelect }) {
  const [isOpen, setIsOpen] = useState(false)

  const selectedSymbol = SYMBOLS.find(s => s.id === selected)

  return (
    <div className="symbol-selector">
      <button
        className="symbol-button"
        onClick={() => setIsOpen(!isOpen)}
      >
        <span className="symbol-icon">{selectedSymbol?.icon}</span>
        <span className="symbol-name">{selectedSymbol?.name}</span>
        <span className="dropdown-arrow">▼</span>
      </button>

      {isOpen && (
        <div className="symbol-dropdown">
          {SYMBOLS.map(symbol => (
            <button
              key={symbol.id}
              className={`symbol-option ${selected === symbol.id ? 'active' : ''}`}
              onClick={() => {
                onSelect(symbol.id)
                setIsOpen(false)
              }}
            >
              <span className="symbol-icon">{symbol.icon}</span>
              <span className="symbol-name">{symbol.name}</span>
            </button>
          ))}
        </div>
      )}
    </div>
  )
}
