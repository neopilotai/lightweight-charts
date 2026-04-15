// src/ws/handler.rs
use axum::extract::ws::{WebSocket, Message};
use futures::{SinkExt, StreamExt};
use crate::AppState;
use tokio::sync::mpsc;
use std::sync::Arc;
use crate::channels::MarketData;

pub async fn handle_socket(mut socket: WebSocket, symbol: String, state: AppState) {
    let symbol_lower = symbol.to_lowercase();
    
    // Create a new channel for this client
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Arc<MarketData>>();
    
    // Add this client's sender to the list of active clients
    {
        let mut client_senders = state.client_senders.lock().await;
        client_senders.push(tx);
    }
    
    loop {
        tokio::select! {
            // Receive market data from our dedicated channel
            result = rx.recv() => {
                match result {
                    Some(market_data) => {
                        if market_data.symbol.to_lowercase() == symbol_lower {
                            // Get the latest candle with indicators from cache
                            let latest_candle = if let Some(candles) = state.candles_cache.get(&symbol_lower) {
                                if let Some(candle) = candles.back() {
                                    Some(candle.clone())
                                } else {
                                    None
                                }
                            } else {
                                None
                            };
                            
                            // If we have cached candle data, use it; otherwise fall back to basic market data
                            let json_msg = if let Some(candle) = latest_candle {
                                serde_json::json!({
                                    "time": candle.time,
                                    "open": candle.open,
                                    "high": candle.high,
                                    "low": candle.low,
                                    "close": candle.close,
                                    "rsi": candle.rsi,
                                    "histogram": candle.histogram,
                                    "macd_line": candle.signal,
                                    "sequence": market_data.sequence
                                })
                            } else {
                                serde_json::json!({
                                    "time": market_data.time,
                                    "open": market_data.open,
                                    "high": market_data.high,
                                    "low": market_data.low,
                                    "close": market_data.close,
                                    "rsi": serde_json::Value::Null,
                                    "histogram": serde_json::Value::Null,
                                    "macd_line": serde_json::Value::Null,
                                    "sequence": market_data.sequence
                                })
                            };
                            
                            if socket.send(Message::Text(json_msg.to_string())).await.is_err() {
                                return;
                            }
                        }
                    }
                    None => {
                        // Channel closed, client is disconnecting
                        return;
                    }
                }
            }
            // Handle ping/pong for connection lifecycle
            result = socket.recv() => {
                match result {
                    Some(Ok(Message::Ping(payload))) => {
                        // Respond to ping with pong
                        let _ = socket.send(Message::Pong(payload)).await;
                    }
                    Some(Ok(Message::Pong(_))) => {
                        // Pong received, connection is alive
                    }
                    Some(Ok(Message::Close(_))) => {
                        // Client initiated close
                        return;
                    }
                    Some(Err(_)) => {
                        // Connection error
                        return;
                    }
                    None => {
                        // Connection closed
                        return;
                    }
                    _ => {
                        // Ignore other message types (text, binary)
                    }
                };
            }
            // Send periodic ping to keep connection alive
            _ = tokio::time::sleep(tokio::time::Duration::from_secs(30)) => {
                if socket.send(Message::Ping(vec![1, 2, 3])).await.is_err() {
                    return;
                }
            }
        }
    }
}