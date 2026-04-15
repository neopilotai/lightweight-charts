// src/ws/handler.rs
use axum::extract::ws::{WebSocket, Message};
use crate::AppState;
use tokio::sync::mpsc;
use std::collections::HashMap;
use std::fmt::Write;
use std::sync::Arc;
use crate::channels::MarketData;
use crate::utils::JsonBuffer;

#[derive(Clone, Debug)]
pub enum SubscriptionPlan {
    Free,
    Pro,
    Elite,
}

impl SubscriptionPlan {
    pub fn from_str(value: &str) -> Option<Self> {
        match value.to_lowercase().as_str() {
            "free" => Some(SubscriptionPlan::Free),
            "pro" => Some(SubscriptionPlan::Pro),
            "elite" => Some(SubscriptionPlan::Elite),
            _ => None,
        }
    }

    pub fn can_access_indicator(&self, indicator: &Indicator) -> bool {
        match self {
            SubscriptionPlan::Free => false,
            SubscriptionPlan::Pro => matches!(indicator, Indicator::Rsi | Indicator::Ema),
            SubscriptionPlan::Elite => true,
        }
    }

    pub fn allowed_indicators(&self) -> Vec<Indicator> {
        match self {
            SubscriptionPlan::Free => vec![],
            SubscriptionPlan::Pro => vec![Indicator::Rsi, Indicator::Ema],
            SubscriptionPlan::Elite => vec![Indicator::Rsi, Indicator::Ema, Indicator::Macd],
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Indicator {
    Rsi,
    Ema,
    Macd,
}

impl Indicator {
    pub fn from_str(value: &str) -> Option<Self> {
        match value.to_lowercase().as_str() {
            "rsi" => Some(Indicator::Rsi),
            "ema" => Some(Indicator::Ema),
            "macd" => Some(Indicator::Macd),
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct SubscriptionPreferences {
    pub user_id: Option<String>,
    pub symbol: String,
    pub plan: SubscriptionPlan,
    pub indicators: Vec<Indicator>,
}

impl SubscriptionPreferences {
    pub fn from_query(symbol: String, params: &HashMap<String, String>) -> Self {
        let plan = params
            .get("plan")
            .and_then(|value| SubscriptionPlan::from_str(value))
            .unwrap_or(SubscriptionPlan::Elite);

        let indicators = params
            .get("indicators")
            .map(|value| {
                value
                    .split(',')
                    .filter_map(|item| Indicator::from_str(item.trim()))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_else(|| plan.allowed_indicators());

        let indicators = indicators
            .into_iter()
            .filter(|indicator| plan.can_access_indicator(indicator))
            .collect();

        Self {
            user_id: params.get("user_id").cloned(),
            symbol,
            plan,
            indicators,
        }
    }
}

pub async fn handle_socket(mut socket: WebSocket, subscription: SubscriptionPreferences, state: AppState) {
    let symbol_lower = subscription.symbol.to_lowercase();
    let indicators = subscription.indicators.clone();

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
                                write!(buf, "{{\"time\":{},\"open\":{},\"high\":{},\"low\":{},\"close\":{}", candle.time, candle.open, candle.high, candle.low, candle.close).unwrap();

                                if indicators.contains(&Indicator::Rsi) {
                                    write!(buf, ",\"rsi\":").unwrap();
                                    write_optional(buf, candle.rsi).unwrap();
                                } else {
                                    write!(buf, ",\"rsi\":null").unwrap();
                                }

                                if indicators.contains(&Indicator::Ema) {
                                    write!(buf, ",\"ema12\":").unwrap();
                                    write_optional(buf, candle.ema12).unwrap();
                                    write!(buf, ",\"ema26\":").unwrap();
                                    write_optional(buf, candle.ema26).unwrap();
                                } else {
                                    write!(buf, ",\"ema12\":null,\"ema26\":null").unwrap();
                                }

                                if indicators.contains(&Indicator::Macd) {
                                    write!(buf, ",\"macd\":").unwrap();
                                    write_optional(buf, candle.macd).unwrap();
                                    write!(buf, ",\"signal\":").unwrap();
                                    write_optional(buf, candle.signal).unwrap();
                                    write!(buf, ",\"histogram\":").unwrap();
                                    write_optional(buf, candle.histogram).unwrap();
                                } else {
                                    write!(buf, ",\"macd\":null,\"signal\":null,\"histogram\":null").unwrap();
                                }

                                write!(buf, ",\"sequence\":{} }}", market_data.sequence).unwrap();
                            } else {
                                write!(buf, "{{\"time\":{},\"open\":{},\"high\":{},\"low\":{},\"close\":{},\"rsi\":null,\"ema12\":null,\"ema26\":null,\"macd\":null,\"signal\":null,\"histogram\":null,\"sequence\":{} }}", market_data.time, market_data.open, market_data.high, market_data.low, market_data.close, market_data.sequence).unwrap();
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