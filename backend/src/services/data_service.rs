use crate::models::candle::Candle;
use crate::models::indicators::calculate_indicators;
use reqwest::Error;
use serde_json::Value;

pub async fn get_historical_candles(symbol: &str) -> Result<Vec<Candle>, Error> {
    let url = format!("https://api.binance.com/api/v3/klines?symbol={}&interval=1m&limit=200", symbol);
    let response = reqwest::get(&url).await?;
    let data: Vec<Vec<Value>> = response.json().await?;

    let mut candles = data.into_iter().map(|kline| {
        Candle {
            time: kline[0].as_u64().unwrap() / 1000, // Convert ms to seconds
            open: kline[1].as_str().unwrap().parse().unwrap(),
            high: kline[2].as_str().unwrap().parse().unwrap(),
            low: kline[3].as_str().unwrap().parse().unwrap(),
            close: kline[4].as_str().unwrap().parse().unwrap(),
            rsi: None,
            ema12: None,
            ema26: None,
            macd: None,
            signal: None,
            histogram: None,
        }
    }).collect::<Vec<Candle>>();

    calculate_indicators(&mut candles);

    Ok(candles)
}