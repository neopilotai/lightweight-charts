import React, { useState } from 'react'
import '../styles/OrderPanel.css'

export default function OrderPanel() {
  const [orderType, setOrderType] = useState('limit') // limit, market, stop
  const [side, setSide] = useState('buy') // buy, sell
  const [formData, setFormData] = useState({
    quantity: '',
    price: '',
    stopPrice: '',
  })

  const handleInputChange = (e) => {
    const { name, value } = e.target
    setFormData(prev => ({
      ...prev,
      [name]: value,
    }))
  }

  const handlePlaceOrder = async (e) => {
    e.preventDefault()
    try {
      const response = await fetch('/api/orders', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          side,
          type: orderType,
          quantity: parseFloat(formData.quantity),
          price: parseFloat(formData.price),
          stopPrice: formData.stopPrice ? parseFloat(formData.stopPrice) : null,
        }),
      })
      if (response.ok) {
        alert('Order placed successfully!')
        setFormData({ quantity: '', price: '', stopPrice: '' })
      }
    } catch (error) {
      console.error('Error placing order:', error)
    }
  }

  return (
    <div className="order-panel">
      <h3>Place Order</h3>

      <div className="order-type-selector">
        {['limit', 'market', 'stop'].map(type => (
          <button
            key={type}
            className={`type-btn ${orderType === type ? 'active' : ''}`}
            onClick={() => setOrderType(type)}
          >
            {type.charAt(0).toUpperCase() + type.slice(1)}
          </button>
        ))}
      </div>

      <form onSubmit={handlePlaceOrder}>
        <div className="side-selector">
          {['buy', 'sell'].map(s => (
            <button
              key={s}
              type="button"
              className={`side-btn ${side === s ? 'active' : ''} ${s}`}
              onClick={() => setSide(s)}
            >
              {s.toUpperCase()}
            </button>
          ))}
        </div>

        <div className="form-group">
          <label>Quantity</label>
          <input
            type="number"
            name="quantity"
            value={formData.quantity}
            onChange={handleInputChange}
            placeholder="0.00"
            step="0.001"
            required
          />
        </div>

        {(orderType === 'limit' || orderType === 'stop') && (
          <div className="form-group">
            <label>{orderType === 'limit' ? 'Limit Price' : 'Stop Price'}</label>
            <input
              type="number"
              name={orderType === 'limit' ? 'price' : 'stopPrice'}
              value={orderType === 'limit' ? formData.price : formData.stopPrice}
              onChange={handleInputChange}
              placeholder="0.00"
              step="0.01"
              required
            />
          </div>
        )}

        {orderType === 'stop' && (
          <div className="form-group">
            <label>Limit Price (optional)</label>
            <input
              type="number"
              name="price"
              value={formData.price}
              onChange={handleInputChange}
              placeholder="0.00"
              step="0.01"
            />
          </div>
        )}

        <button type="submit" className={`submit-btn ${side}`}>
          Place {side.toUpperCase()} Order
        </button>
      </form>
    </div>
  )
}
