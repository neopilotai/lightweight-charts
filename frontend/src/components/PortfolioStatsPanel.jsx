import React, { useEffect, useState } from 'react'
import '../styles/PortfolioStatsPanel.css'

export default function PortfolioStatsPanel({ stats }) {
  const [portfolioData, setPortfolioData] = useState(null)

  useEffect(() => {
    const fetchPortfolioStats = async () => {
      try {
        // This would fetch from a portfolio endpoint
        // For now, showing mock data
        setPortfolioData({
          totalBalance: 100000,
          openPositions: 2,
          todayPnL: 1250,
          todayPnLPercent: 1.25,
          monthlyReturn: 5.8,
          winRate: 0.62,
          maxDrawdown: -3.5,
        })
      } catch (error) {
        console.error('Error fetching portfolio stats:', error)
      }
    }
    fetchPortfolioStats()
  }, [])

  if (!portfolioData) {
    return (
      <div className="portfolio-panel-content">
        <h3>Portfolio Stats</h3>
        <p className="loading">Loading...</p>
      </div>
    )
  }

  return (
    <div className="portfolio-panel-content">
      <h3>Portfolio Stats</h3>
      <div className="stats-grid">
        <div className="stat-item">
          <label>Total Balance</label>
          <div className="stat-value">${portfolioData.totalBalance.toLocaleString()}</div>
        </div>

        <div className="stat-item">
          <label>Today P&L</label>
          <div className={`stat-value ${portfolioData.todayPnL >= 0 ? 'positive' : 'negative'}`}>
            ${portfolioData.todayPnL.toFixed(2)}
            <span className="stat-percent">({portfolioData.todayPnLPercent.toFixed(2)}%)</span>
          </div>
        </div>

        <div className="stat-item">
          <label>Open Positions</label>
          <div className="stat-value">{portfolioData.openPositions}</div>
        </div>

        <div className="stat-item">
          <label>Win Rate</label>
          <div className="stat-value">{(portfolioData.winRate * 100).toFixed(1)}%</div>
        </div>

        <div className="stat-item">
          <label>Monthly Return</label>
          <div className={`stat-value ${portfolioData.monthlyReturn >= 0 ? 'positive' : 'negative'}`}>
            {portfolioData.monthlyReturn.toFixed(2)}%
          </div>
        </div>

        <div className="stat-item">
          <label>Max Drawdown</label>
          <div className="stat-value negative">{portfolioData.maxDrawdown.toFixed(2)}%</div>
        </div>
      </div>
    </div>
  )
}
