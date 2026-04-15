import React, { useState } from 'react'
import '../styles/StrategyPanel.css'

const STRATEGY_TYPES = [
  'ma_crossover',
  'rsi_momentum',
  'macd',
  'multi_indicator',
]

export default function StrategyPanel({ strategies, onStrategiesUpdate }) {
  const [showForm, setShowForm] = useState(false)
  const [formData, setFormData] = useState({
    name: '',
    strategy_type: 'rsi_momentum',
    symbol: 'BTCUSDT',
    risk_percent: 2.0,
    stop_loss_pct: 2.0,
    take_profit_pct: 5.0,
  })
  const [loading, setLoading] = useState(false)

  const handleInputChange = (e) => {
    const { name, value } = e.target
    setFormData(prev => ({
      ...prev,
      [name]: name.includes('percent') ? parseFloat(value) : value,
    }))
  }

  const handleSubmit = async (e) => {
    e.preventDefault()
    setLoading(true)

    try {
      const response = await fetch('/api/trading/strategies', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(formData),
      })

      if (response.ok) {
        const newStrategy = await response.json()
        onStrategiesUpdate([...strategies, newStrategy])
        setFormData({
          name: '',
          strategy_type: 'rsi_momentum',
          symbol: 'BTCUSDT',
          risk_percent: 2.0,
          stop_loss_pct: 2.0,
          take_profit_pct: 5.0,
        })
        setShowForm(false)
      }
    } catch (error) {
      console.error('Error creating strategy:', error)
    } finally {
      setLoading(false)
    }
  }

  const handleDeleteStrategy = async (strategyId) => {
    try {
      await fetch(`/api/trading/strategies/${strategyId}`, { method: 'DELETE' })
      onStrategiesUpdate(strategies.filter(s => s.id !== strategyId))
    } catch (error) {
      console.error('Error deleting strategy:', error)
    }
  }

  const handleToggleStrategy = async (strategyId, enabled) => {
    try {
      const response = await fetch(`/api/trading/strategies/${strategyId}/toggle`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ enabled: !enabled }),
      })
      if (response.ok) {
        const updated = await response.json()
        onStrategiesUpdate(strategies.map(s => s.id === strategyId ? updated : s))
      }
    } catch (error) {
      console.error('Error toggling strategy:', error)
    }
  }

  return (
    <div className="strategy-panel-content">
      <div className="strategy-header">
        <h3>Strategies</h3>
        <button className="add-btn" onClick={() => setShowForm(!showForm)}>
          {showForm ? '✕' : '+ Add'}
        </button>
      </div>

      {showForm && (
        <form className="strategy-form" onSubmit={handleSubmit}>
          <div className="form-group">
            <label>Name</label>
            <input
              type="text"
              name="name"
              value={formData.name}
              onChange={handleInputChange}
              placeholder="Strategy name"
              required
            />
          </div>

          <div className="form-group">
            <label>Type</label>
            <select
              name="strategy_type"
              value={formData.strategy_type}
              onChange={handleInputChange}
            >
              {STRATEGY_TYPES.map(type => (
                <option key={type} value={type}>
                  {type.replace('_', ' ').toUpperCase()}
                </option>
              ))}
            </select>
          </div>

          <div className="form-group">
            <label>Symbol</label>
            <select name="symbol" value={formData.symbol} onChange={handleInputChange}>
              <option value="BTCUSDT">BTC/USDT</option>
              <option value="ETHUSDT">ETH/USDT</option>
              <option value="SOLUSDT">SOL/USDT</option>
            </select>
          </div>

          <div className="form-row">
            <div className="form-group">
              <label>Risk %</label>
              <input
                type="number"
                name="risk_percent"
                value={formData.risk_percent}
                onChange={handleInputChange}
                step="0.1"
              />
            </div>
            <div className="form-group">
              <label>Stop Loss %</label>
              <input
                type="number"
                name="stop_loss_pct"
                value={formData.stop_loss_pct}
                onChange={handleInputChange}
                step="0.1"
              />
            </div>
          </div>

          <div className="form-group">
            <label>Take Profit %</label>
            <input
              type="number"
              name="take_profit_pct"
              value={formData.take_profit_pct}
              onChange={handleInputChange}
              step="0.1"
            />
          </div>

          <button type="submit" className="submit-btn" disabled={loading}>
            {loading ? 'Creating...' : 'Create Strategy'}
          </button>
        </form>
      )}

      <div className="strategies-list">
        {strategies.length === 0 ? (
          <p className="no-strategies">No strategies yet</p>
        ) : (
          strategies.map(strategy => (
            <div key={strategy.id} className={`strategy-item ${strategy.enabled ? 'active' : 'inactive'}`}>
              <div className="strategy-info">
                <h4>{strategy.name}</h4>
                <p className="strategy-type">{strategy.config?.strategy_type}</p>
              </div>
              <div className="strategy-actions">
                <button
                  className={`toggle-btn ${strategy.enabled ? 'enabled' : 'disabled'}`}
                  onClick={() => handleToggleStrategy(strategy.id, strategy.enabled)}
                  title={strategy.enabled ? 'Disable' : 'Enable'}
                >
                  {strategy.enabled ? '✓' : '○'}
                </button>
                <button
                  className="delete-btn"
                  onClick={() => handleDeleteStrategy(strategy.id)}
                  title="Delete"
                >
                  🗑
                </button>
              </div>
            </div>
          ))
        )}
      </div>
    </div>
  )
}
