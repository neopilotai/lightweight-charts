use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct BinanceStreamMessage {
    pub stream: String,
    pub data: BinanceData,
}

#[derive(Debug, Deserialize)]
pub struct BinanceData {
    #[serde(rename = "k")]
    pub kline: BinanceKline,
}

#[derive(Debug, Deserialize)]
pub struct BinanceKline {
    #[serde(rename = "t")]
    pub start_time: u64,
    #[serde(rename = "o")]
    pub open: String,
    #[serde(rename = "h")]
    pub high: String,
    #[serde(rename = "l")]
    pub low: String,
    #[serde(rename = "c")]
    pub close: String,
    #[serde(rename = "T")]
    pub end_time: u64,
    #[serde(rename = "s")]
    pub symbol: String,
    #[serde(rename = "i")]
    pub interval: String,
    #[serde(rename = "f")]
    pub first_trade_id: i64,
    #[serde(rename = "L")]
    pub last_trade_id: i64,
    #[serde(rename = "v")]
    pub base_volume: String,
    #[serde(rename = "q")]
    pub quote_volume: String,
    #[serde(rename = "n")]
    pub number_of_trades: i64,
    #[serde(rename = "x")]
    pub is_closed: bool,
    #[serde(rename = "V")]
    pub taker_buy_base_volume: String,
    #[serde(rename = "Q")]
    pub taker_buy_quote_volume: String,
}
