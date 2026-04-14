// src/ws/handler.rs
use axum::extract::ws::{WebSocket, Message};
use futures::{SinkExt, StreamExt};
use std::sync::Arc;
use crate::AppState;

pub async fn handle_socket(mut socket: WebSocket, symbol: String, state: AppState) {
    let symbol_lower = symbol.to_lowercase();

    // Batch receive loop for performance
    loop {
        // Get batch of market data (avoids lock overhead)
        let batch = state.market_channel.recv_batch(16);

        for market_data in batch {
            if market_data.symbol.to_lowercase() == symbol_lower {
                let json_msg = serde_json::json!({
                    "time": market_data.time,
                    "open": market_data.open,
                    "high": market_data.high,
                    "low": market_data.low,
                    "close": market_data.close
                });

                if socket.send(Message::Text(json_msg.to_string())).await.is_err() {
                    return;
                }
            }
        }

        // Check for client disconnect
        tokio::select! {
            _ = socket.next() => {
                return;
            }
            _ = tokio::time::sleep(tokio::time::Duration::from_millis(1)) => {
                // Try again
                continue;
            }
        }
    }
}