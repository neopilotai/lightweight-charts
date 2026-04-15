// src/metrics.rs
use lazy_static::lazy_static;
use prometheus::{
    Encoder, Histogram, HistogramOpts as HistogramOptions, IntCounter, IntGauge, Registry,
};

lazy_static! {
    pub static ref REGISTRY: Registry = Registry::new();

    // Candle metrics
    pub static ref CANDLES_PROCESSED: IntCounter = IntCounter::new(
        "candles_processed_total",
        "Total number of candles processed from Binance"
    ).unwrap();

    pub static ref PARSE_ERRORS: IntCounter = IntCounter::new(
        "parse_errors_total",
        "Total number of JSON parse errors"
    ).unwrap();

    // WebSocket metrics
    pub static ref WS_CONNECTIONS: IntGauge = IntGauge::new(
        "websocket_connections_active",
        "Number of active WebSocket connections"
    ).unwrap();

    pub static ref WS_MESSAGES_SENT: IntCounter = IntCounter::new(
        "websocket_messages_sent_total",
        "Total WebSocket messages sent to clients"
    ).unwrap();

    // Performance metrics
    pub static ref MESSAGE_LATENCY: Histogram = Histogram::with_opts(
        HistogramOptions::new(
            "message_processing_latency_ms",
            "Time to process a single Binance message in milliseconds"
        ).buckets(vec![0.1, 0.5, 1.0, 5.0, 10.0, 50.0, 100.0, 500.0])
    ).unwrap();

    // Cache metrics
    pub static ref CACHE_SIZE: IntGauge = IntGauge::new(
        "candle_cache_size",
        "Number of candles in cache per symbol"
    ).unwrap();

    pub static ref SYMBOLS_TRACKED: IntGauge = IntGauge::new(
        "symbols_tracked",
        "Number of unique symbols being tracked"
    ).unwrap();

    // Connection metrics
    pub static ref BINANCE_RECONNECTS: IntCounter = IntCounter::new(
        "binance_reconnects_total",
        "Total number of Binance WebSocket reconnection attempts"
    ).unwrap();

    pub static ref BINANCE_CONNECTED: IntGauge = IntGauge::new(
        "binance_connected",
        "Whether connected to Binance (1 = yes, 0 = no)"
    ).unwrap();

    pub static ref CIRCUIT_BREAKER_STATE: IntGauge = IntGauge::new(
        "circuit_breaker_state",
        "Circuit breaker state: 0=closed, 1=open, 2=half_open"
    ).unwrap();
}

pub fn init_metrics() -> Result<(), Box<dyn std::error::Error>> {
    REGISTRY.register(Box::new(CANDLES_PROCESSED.clone()))?;
    REGISTRY.register(Box::new(PARSE_ERRORS.clone()))?;
    REGISTRY.register(Box::new(WS_CONNECTIONS.clone()))?;
    REGISTRY.register(Box::new(WS_MESSAGES_SENT.clone()))?;
    REGISTRY.register(Box::new(MESSAGE_LATENCY.clone()))?;
    REGISTRY.register(Box::new(CACHE_SIZE.clone()))?;
    REGISTRY.register(Box::new(SYMBOLS_TRACKED.clone()))?;
    REGISTRY.register(Box::new(BINANCE_RECONNECTS.clone()))?;
    REGISTRY.register(Box::new(BINANCE_CONNECTED.clone()))?;
    REGISTRY.register(Box::new(CIRCUIT_BREAKER_STATE.clone()))?;
    Ok(())
}

pub fn gather_metrics() -> Vec<u8> {
    let mut buffer = Vec::new();
    let metric_families = REGISTRY.gather();
    let encoder = prometheus::TextEncoder::new();
    encoder.encode(&metric_families, &mut buffer).unwrap();
    buffer
}
