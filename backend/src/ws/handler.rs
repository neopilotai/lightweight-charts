// src/ws/handler.rs
use axum::extract::ws::{WebSocket, Message};
use crate::AppState;
use tokio::sync::mpsc;
use std::fmt::Write;
use std::sync::Arc;
use crate::channels::MarketData;
use crate::utils::JsonBuffer;

pub async fn handle_socket(mut socket: WebSocket, symbol: String, state: AppState) {
    let symbol_lower = symbol.to_lowercase();
    
    // Create a new channel for this client
    let (tx, mut rx) = mpsc::unbounded_channel::<Arc<MarketData>>();
    
    // Add this client's sender to the list of active clients
    {
        let mut client_senders = state.client_senders.lock().await;
        client_senders.push(tx);
    }
    
    let mut json_buffer = JsonBuffer::default();

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

                            let buf = json_buffer.as_mut_string();
                            buf.clear();

                            fn write_optional(buf: &mut String, value: Option<f64>) -> std::fmt::Result {
                                match value {
                                    Some(v) => write!(buf, "{}", v),
                                    None => {
                                        buf.push_str("null");
                                        Ok(())
                                    }
                                }
                            }

                            if let Some(candle) = latest_candle {
                                write!(buf, "{{\"time\":{},\"open\":{},\"high\":{},\"low\":{},\"close\":{},\"rsi\":", candle.time, candle.open, candle.high, candle.low, candle.close).unwrap();
                                write_optional(buf, candle.rsi).unwrap();
                                write!(buf, ",\"histogram\":").unwrap();
                                write_optional(buf, candle.histogram).unwrap();
                                write!(buf, ",\"macd_line\":").unwrap();
                                write_optional(buf, candle.signal).unwrap();
                                write!(buf, ",\"sequence\":{} }}", market_data.sequence).unwrap();
                            } else {
                                write!(buf, "{{\"time\":{},\"open\":{},\"high\":{},\"low\":{},\"close\":{},\"rsi\":null,\"histogram\":null,\"macd_line\":null,\"sequence\":{} }}", market_data.time, market_data.open, market_data.high, market_data.low, market_data.close, market_data.sequence).unwrap();
                            }

                            if socket.send(Message::Text(buf.clone())).await.is_err() {
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