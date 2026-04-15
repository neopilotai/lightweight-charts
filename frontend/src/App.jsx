import React, { useState, useEffect } from 'react'
import ChartPanel from './components/ChartPanel'
import StrategyPanel from './components/StrategyPanel'
import SignalsPanel from './components/SignalsPanel'
import PortfolioStatsPanel from './components/PortfolioStatsPanel'
import SymbolSelector from './components/SymbolSelector'
import './styles/App.css'

export default function App() {
  const [selectedSymbol, setSelectedSymbol] = useState('btcusdt')
  const [signals, setSignals] = useState([])
  const [strategies, setStrategies] = useState([])
  const [portfolioStats, setPortfolioStats] = useState(null)

  // Fetch signals when symbol changes
  useEffect(() => {
    const fetchSignals = async () => {
      try {
        const response = await fetch(`/api/trading/signals?symbol=${selectedSymbol}`)
        const data = await response.json()
        setSignals(data || [])
      } catch (error) {
        console.error('Error fetching signals:', error)
      }
    }
    fetchSignals()
  }, [selectedSymbol])

  // Fetch strategies
  useEffect(() => {
    const fetchStrategies = async () => {
      try {
        const response = await fetch('/api/trading/strategies/list')
        const data = await response.json()
        setStrategies(data || [])
      } catch (error) {
        console.error('Error fetching strategies:', error)
      }
    }
    fetchStrategies()
  }, [])

  return (
    <div className="app">
      <header className="app-header">
        <h1>📈 Trading Dashboard</h1>
        <SymbolSelector selected={selectedSymbol} onSelect={setSelectedSymbol} />
      </header>

      <div className="app-layout">
        {/* Main Chart Area */}
        <div className="chart-container">
          <ChartPanel symbol={selectedSymbol} />
        </div>

        {/* Right Sidebar */}
        <div className="sidebar">
          {/* Signals Panel */}
          <div className="panel signals-panel">
            <SignalsPanel signals={signals} />
          </div>

          {/* Portfolio Stats Panel */}
          <div className="panel portfolio-panel">
            <PortfolioStatsPanel stats={portfolioStats} />
          </div>

          {/* Strategy Panel */}
          <div className="panel strategy-panel">
            <StrategyPanel strategies={strategies} onStrategiesUpdate={setStrategies} />
          </div>
        </div>
      </div>
    </div>
  )
}
