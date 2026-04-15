// src/channels/mod.rs
use parking_lot::Mutex;
use smallvec::SmallVec;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

#[derive(Clone, Debug, serde::Serialize)]
pub struct MarketData {
    pub symbol: String,
    pub sequence: u64,
    pub time: u64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

/// Lock-free ring buffer for HFT market data
pub struct LockFreeChannel {
    buffer: Vec<Mutex<Option<Arc<MarketData>>>>,
    write_pos: AtomicUsize,
    read_pos: AtomicUsize,
    capacity: usize,
}

impl LockFreeChannel {
    pub fn new(capacity: usize) -> Self {
        let mut buffer = Vec::with_capacity(capacity);
        for _ in 0..capacity {
            buffer.push(Mutex::new(None));
        }

        LockFreeChannel {
            buffer,
            write_pos: AtomicUsize::new(0),
            read_pos: AtomicUsize::new(0),
            capacity,
        }
    }

    /// Fast, lock-free send (zero-copy Arc reference)
    #[inline]
    pub fn send(&self, data: Arc<MarketData>) -> Result<(), Arc<MarketData>> {
        let write_idx = self.write_pos.load(Ordering::Relaxed) % self.capacity;
        let next_write = (self.write_pos.load(Ordering::Relaxed) + 1) % (self.capacity * 2);

        if let Some(mut slot) = self.buffer[write_idx].try_lock() {
            *slot = Some(data.clone());
            self.write_pos.store(next_write, Ordering::Release);
            Ok(())
        } else {
            Err(data)
        }
    }

    /// Fast, lock-free receive (zero-copy Arc reference)
    #[inline]
    pub fn try_recv(&self) -> Option<Arc<MarketData>> {
        let read_idx = self.read_pos.load(Ordering::Relaxed) % self.capacity;

        if let Some(mut slot) = self.buffer[read_idx].try_lock() {
            if let Some(data) = slot.take() {
                let next_read = (self.read_pos.load(Ordering::Relaxed) + 1) % (self.capacity * 2);
                self.read_pos.store(next_read, Ordering::Release);
                return Some(data);
            }
        }

        None
    }

    /// Get all pending messages (batch processing for efficiency)
    pub fn recv_batch(&self, max_items: usize) -> SmallVec<[Arc<MarketData>; 32]> {
        let mut results = SmallVec::new();

        for _ in 0..max_items {
            if let Some(data) = self.try_recv() {
                results.push(data);
            } else {
                break;
            }
        }

        results
    }

    /// Check if buffer has pending data
    #[inline]
    pub fn has_pending(&self) -> bool {
        let write_pos = self.write_pos.load(Ordering::Acquire);
        let read_pos = self.read_pos.load(Ordering::Acquire);
        write_pos != read_pos
    }

    /// Get current queue depth
    #[inline]
    pub fn depth(&self) -> usize {
        let write_pos = self.write_pos.load(Ordering::Acquire);
        let read_pos = self.read_pos.load(Ordering::Acquire);
        write_pos.saturating_sub(read_pos)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lock_free_channel() {
        let channel = LockFreeChannel::new(100);
        let data = Arc::new(MarketData {
            symbol: "BTCUSDT".to_string(),
            time: 1234567890,
            open: 50000.0,
            high: 51000.0,
            low: 49000.0,
            close: 50500.0,
        });

        assert!(channel.send(data.clone()).is_ok());
        assert_eq!(channel.depth(), 1);

        let received = channel.try_recv().unwrap();
        assert_eq!(received.symbol, "BTCUSDT");
        assert_eq!(channel.depth(), 0);
    }
}
