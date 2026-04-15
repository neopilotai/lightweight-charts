use rocksdb::{DB, Error, IteratorMode, Options};
use std::sync::Arc;
use crate::models::candle::Candle;

#[derive(Clone)]
pub struct DbStore {
    db: Arc<DB>,
}

impl DbStore {
    pub fn open(path: &str) -> Result<Self, Error> {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        let db = DB::open(&opts, path)?;
        Ok(Self { db: Arc::new(db) })
    }

    fn key(symbol: &str) -> String {
        format!("candles:{}", symbol)
    }

    pub fn save_candle_history(&self, symbol: &str, candles: &[Candle]) -> Result<(), Box<dyn std::error::Error>> {
        let key = Self::key(symbol);
        let data = serde_json::to_vec(candles)?;
        self.db.put(key, data)?;
        Ok(())
    }

    pub fn load_candle_history(&self, symbol: &str) -> Result<Vec<Candle>, Box<dyn std::error::Error>> {
        let key = Self::key(symbol);
        match self.db.get(key)? {
            Some(value) => {
                let candles: Vec<Candle> = serde_json::from_slice(&value)?;
                Ok(candles)
            }
            None => Ok(vec![]),
        }
    }

    pub fn list_symbols(&self) -> Result<Vec<String>, Error> {
        let iter = self.db.iterator(IteratorMode::Start);
        let mut symbols = Vec::new();

        for item in iter {
            let (key, _) = item?;
            if let Ok(key_str) = std::str::from_utf8(&key) {
                if let Some(symbol) = key_str.strip_prefix("candles:") {
                    symbols.push(symbol.to_string());
                }
            }
        }

        Ok(symbols)
    }

    pub fn flush(&self) -> Result<(), Error> {
        self.db.flush()
    }
}
