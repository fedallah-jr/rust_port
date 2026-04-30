//! Token-bucket rate limiter matching Binance API limits.

use std::sync::Mutex;
use std::time::Instant;

pub struct RateLimiter {
    inner: Mutex<Inner>,
}

struct Inner {
    tokens: f64,
    limit_per_minute: f64,
    last_update: Instant,
}

impl RateLimiter {
    pub fn new(limit_per_minute: u32) -> Self {
        let limit = limit_per_minute as f64;
        Self {
            inner: Mutex::new(Inner {
                tokens: limit,
                limit_per_minute: limit,
                last_update: Instant::now(),
            }),
        }
    }

    /// Block until `weight` tokens are available, then consume them.
    ///
    /// Sleeps exactly as long as needed for the bucket to refill to `weight`
    /// instead of polling every 100 ms. The outer loop is retained so a
    /// spurious wake-up or `sync_from_server` adjustment re-checks availability.
    pub fn acquire(&self, weight: u32) {
        let w = weight as f64;
        loop {
            let sleep_for = {
                let mut inner = self.inner.lock().unwrap();
                inner.refill();
                if inner.tokens >= w {
                    inner.tokens -= w;
                    return;
                }
                let needed = w - inner.tokens;
                let tokens_per_sec = inner.limit_per_minute / 60.0;
                // Guard against pathological limits; fall back to a short sleep.
                if !tokens_per_sec.is_finite() || tokens_per_sec <= 0.0 {
                    std::time::Duration::from_millis(100)
                } else {
                    std::time::Duration::from_secs_f64(needed / tokens_per_sec)
                }
            };
            std::thread::sleep(sleep_for);
        }
    }

    /// Sync with server-reported used weight.
    pub fn sync_from_server(&self, used_weight: u32) {
        let mut inner = self.inner.lock().unwrap();
        inner.refill();
        let remaining = inner.limit_per_minute - used_weight as f64;
        if remaining < inner.tokens {
            inner.tokens = remaining.max(0.0);
        }
    }
}

impl Inner {
    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_update).as_secs_f64();
        let refill = (self.limit_per_minute / 60.0) * elapsed;
        self.tokens = (self.tokens + refill).min(self.limit_per_minute);
        self.last_update = now;
    }
}
