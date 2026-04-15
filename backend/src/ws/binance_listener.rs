// src/ws/binance_listener.rs
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message as TungsteniteMessage};
use url::Url;
use std::sync::Arc;
use crate::AppState;
use crate::models::candle::Candle;
use crate::models::indicators::update_indicators_last;
use crate::models::binance::*;
use crate::channels::MarketData;
use crate::metrics::{CANDLES_PROCESSED, PARSE_ERRORS, BINANCE_RECONNECTS, BINANCE_CONNECTED, MESSAGE_LATENCY};
use std::collections::VecDeque;
use futures::StreamExt;
use std::time::Instant;

fn calculate_backoff(attempt: u32) -> tokio::time::Duration {
    // Exponential backoff with jitter: 1s, 2s, 4s, 8s, 16s, 32s, 60s (max)
    let base = tokio::time::Duration::from_secs(1);
    let exp = base * 2_u32.pow(attempt.min(6)); // Cap at 64 seconds
    let jitter = tokio::time::Duration::from_millis(
        (fastrand::u64(0..1000) as u64) // Random 0-999ms jitter
    );
    exp + jitter
}

fn get_next_sequence(state: &AppState, symbol: &str) -> u64 {
    let tracker = state.sequence_tracker.lock();
    let entry = tracker.entry(symbol.to_string());
    let mut entry = entry.or_insert(0);
    *entry += 1;
    *entry
}

pub async fn start_binance_listener(state: AppState) {
    let symbols = vec!["btcusdt", "ethusdt", "solusdt"];
    let streams: Vec<String> = symbols.iter().map(|s| format!("{}@kline_1m", s)).collect();
    let stream_url = format!("wss://stream.binance.com:9443/stream?streams={}", streams.join("/"));

    let mut reconnect_attempt = 0u32;

    loop {
        let url = Url::parse(&stream_url).unwrap();
        let (ws_stream, _) = match connect_async(url).await {
            Ok(conn) => {
                // Reset reconnect counter on successful connection
                reconnect_attempt = 0;
                BINANCE_CONNECTED.set(1);
                tracing::info!("Connected to Binance WebSocket");
                conn
            },
            Err(e) => {
                let backoff = calculate_backoff(reconnect_attempt);
                BINANCE_RECONNECTS.inc();
                BINANCE_CONNECTED.set(0);
                tracing::warn!("Failed to connect to Binance: {:?}. Retrying in {:?}", e, backoff);
                tokio::time::sleep(backoff).await;
                reconnect_attempt = reconnect_attempt.saturating_add(1);
                continue;
            }
        };

        let (mut _write, mut read) = ws_stream.split();

        while let Some(msg) = read.next().await {
            let _start = Instant::now();
            match msg {
                Ok(TungsteniteMessage::Text(text)) => {
                    // Parse using serde structs for better error handling
                    let parsed: Result<BinanceStreamMessage, _> = serde_json::from_str(&text);
                    match parsed {
                        Ok(data) => {
                            let symbol = data.data.kline.symbol.clone();
                            let symbol_lower = symbol.to_lowercase();
                            
                            // Only process closed klines to avoid duplicate updates
                            if !data.data.kline.is_closed {
                                continue;
                            }
                            
                            let time = data.data.kline.start_time / 1000;
                            let open = match data.data.kline.open.parse::<f64>() {
                                Ok(val) => val,
                                Err(e) => {
                                    PARSE_ERRORS.inc();
                                    tracing::error!(error = %e, "Failed to parse open price");
                                    continue;
                                }
                            };
                            let high = match data.data.kline.high.parse::<f64>() {
                                Ok(val) => val,
                                Err(e) => {
                                    PARSE_ERRORS.inc();
                                    tracing::error!(error = %e, "Failed to parse high price");
                                    continue;
                                }
                            };
                            let low = match data.data.kline.low.parse::<f64>() {
                                Ok(val) => val,
                                Err(e) => {
                                    PARSE_ERRORS.inc();
                                    tracing::error!(error = %e, "Failed to parse low price");
                                    continue;
                                }
                            };
                            let close = match data.data.kline.close.parse::<f64>() {
                                Ok(val) => val,
                                Err(e) => {
                                    PARSE_ERRORS.inc();
                                    tracing::error!(error = %e, "Failed to parse close price");
                                    continue;
                                }
                            };

                            // Create market data (zero-copy Arc) with sequence number
                            let sequence = get_next_sequence(&state, &symbol);
                            let market_data = Arc::new(MarketData {
                                symbol,
                                sequence,
                                time,
                                open,
                                high,
                                low,
                                close,
                            });

                            // Track metrics
                            CANDLES_PROCESSED.inc();
                            tracing::debug!(symbol = %symbol_lower, time = time, "Processed new candle");

                            // Update cache with lock-free message
                            // Initialize if not exists
                            state.candles_cache.entry(symbol_lower.clone()).or_insert_with(VecDeque::new);

                            if let Some(mut candles) = state.candles_cache.get_mut(&symbol_lower) {
                                let len = candles.len();
                                if len > 0 {
                                    let last_time = candles.back().unwrap().time;
                                    if time == last_time {
                                        // Update existing candle
                                        if let Some(last) = candles.back_mut() {
                                            last.open = open;
                                            last.high = high;
                                            last.low = low;
                                            last.close = close;
                                        }
                                    } else {
                                        // Add new candle
                                        let new_candle = Candle {
                                            time,
                                            open,
                                            high,
                                            low,
                                            close,
                                            rsi: None,
                                            ema12: None,
                                            ema26: None,
                                            macd: None,
                                            signal: None,
                                            histogram: None,
                                        };
                                        candles.push_back(new_candle);
                                        // Limit size
                                        while candles.len() > crate::MAX_CANDLES {
                                            candles.pop_front();
                                        }
                                    }
                                    // Update indicators incrementally
                                    update_indicators_last(&mut candles);
                                } else {
                                    // First candle for this symbol
                                    let new_candle = Candle {
                                        time,
                                        open,
                                        high,
                                        low,
                                        close,
                                        rsi: None,
                                        ema12: None,
                                        ema26: None,
                                        macd: None,
                                        signal: None,
                                        histogram: None,
                                    };
                                    candles.push_back(new_candle);
                                    // Update indicators for the first candle
                                    update_indicators_last(&mut candles);
                                }
                            }

                            // Send to all connected clients via their individual channels
                            let client_senders = state.client_senders.lock().await;
                            for sender in client_senders.iter() {
                                let _ = sender.send(market_data.clone());
                            }
                        }
Err(e) => {
                            PARSE_ERRORS.inc();
                            tracing::error!(error = %e, "Failed to parse Binance message");
                            continue;
                        }
                    }
                }
Err(e) => {
                    tracing::error!(error = %e, "WebSocket error from Binance");
                    BINANCE_CONNECTED.set(0);
                    break;
                }
                _ => {}
            }
        }
        
        // Connection lost, prepare for reconnection
        tracing::warn!("Binance connection lost, reconnecting...");
        let backoff = calculate_backoff(reconnect_attempt);
        tokio::time::sleep(backoff).await;
        reconnect_attempt = reconnect_attempt.saturating_add(1);
    }
}