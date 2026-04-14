// src/ws/binance_listener.rs
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message as TungsteniteMessage};
use url::Url;
use serde_json::Value;
use std::sync::Arc;
use crate::AppState;
use crate::models::candle::Candle;
use crate::models::indicators::update_indicators_last;
use crate::channels::MarketData;
use std::collections::VecDeque;
use futures::StreamExt;

pub async fn start_binance_listener(state: AppState) {
    let symbols = vec!["btcusdt", "ethusdt", "solusdt"];
    let streams: Vec<String> = symbols.iter().map(|s| format!("{}@kline_1m", s)).collect();
    let stream_url = format!("wss://stream.binance.com:9443/stream?streams={}", streams.join("/"));

    loop {
        let url = Url::parse(&stream_url).unwrap();
        let (ws_stream, _) = match connect_async(url).await {
            Ok(conn) => conn,
            Err(e) => {
                eprintln!("Failed to connect to Binance: {:?}", e);
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                continue;
            }
        };

        let (mut _write, mut read) = ws_stream.split();

        while let Some(msg) = read.next().await {
            match msg {
                Ok(TungsteniteMessage::Text(text)) => {
                    let data: Value = serde_json::from_str(&text).unwrap();
                    if let Some(stream) = data.get("stream").and_then(|s| s.as_str()) {
                        let symbol = stream.split('@').next().unwrap().to_uppercase();
                        let symbol_lower = symbol.to_lowercase();
                        if let Some(kline_data) = data.get("data").and_then(|d| d.get("k")) {
                            let time = kline_data["t"].as_u64().unwrap() / 1000;
                            let open = kline_data["o"].as_str().unwrap().parse::<f64>().unwrap();
                            let high = kline_data["h"].as_str().unwrap().parse::<f64>().unwrap();
                            let low = kline_data["l"].as_str().unwrap().parse::<f64>().unwrap();
                            let close = kline_data["c"].as_str().unwrap().parse::<f64>().unwrap();

                            // Create market data (zero-copy Arc)
                            let market_data = Arc::new(MarketData {
                                symbol,
                                time,
                                open,
                                high,
                                low,
                                close,
                            });

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
                                }
                            }

                            // Send via lock-free channel (retry on contention)
                            let mut retries = 0;
                            while let Err(_data) = state.market_channel.send(market_data.clone()) {
                                retries += 1;
                                if retries > 3 {
                                    eprintln!("Channel full, dropping message");
                                    break;
                                }
                                tokio::time::sleep(tokio::time::Duration::from_micros(10)).await;
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("WebSocket error: {:?}", e);
                    break;
                }
                _ => {}
            }
        }

        eprintln!("Binance connection lost, reconnecting...");
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
}