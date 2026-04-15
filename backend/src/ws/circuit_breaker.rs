// src/ws/circuit_breaker.rs
use parking_lot::Mutex;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, Instant};

pub struct CircuitBreaker {
    state: AtomicU32,
    failures: AtomicU32,
    last_failure_time: Mutex<Option<Instant>>,
    failure_threshold: u32,
    timeout: Duration,
}

impl CircuitBreaker {
    pub fn new(failure_threshold: u32, timeout_secs: u64) -> Self {
        Self {
            state: AtomicU32::new(0), // 0=Closed, 1=Open, 2=HalfOpen
            failures: AtomicU32::new(0),
            last_failure_time: Mutex::new(None),
            failure_threshold,
            timeout: Duration::from_secs(timeout_secs),
        }
    }

    pub fn state(&self) -> &'static str {
        match self.state.load(Ordering::Relaxed) {
            0 => "closed",
            1 => "open",
            2 => "half_open",
            _ => "unknown",
        }
    }

    pub fn can_execute(&self) -> bool {
        let state = self.state.load(Ordering::Relaxed);

        if state == 0 {
            return true;
        }

        if state == 2 {
            return true;
        }

        if state == 1 {
            let last_failure = self.last_failure_time.lock();
            if let Some(time) = *last_failure {
                if time.elapsed() > self.timeout {
                    self.state.store(2, Ordering::Relaxed);
                    return true;
                }
            }
            return false;
        }

        false
    }

    pub fn record_success(&self) {
        self.failures.store(0, Ordering::Relaxed);
        self.state.store(0, Ordering::Relaxed);
    }

    pub fn record_failure(&self) {
        let failures = self.failures.fetch_add(1, Ordering::Relaxed) + 1;
        *self.last_failure_time.lock() = Some(Instant::now());

        if failures >= self.failure_threshold {
            self.state.store(1, Ordering::Relaxed);
        }
    }
}

impl Clone for CircuitBreaker {
    fn clone(&self) -> Self {
        Self {
            state: AtomicU32::new(self.state.load(Ordering::Relaxed)),
            failures: AtomicU32::new(self.failures.load(Ordering::Relaxed)),
            last_failure_time: Mutex::new(*self.last_failure_time.lock()),
            failure_threshold: self.failure_threshold,
            timeout: self.timeout,
        }
    }
}
