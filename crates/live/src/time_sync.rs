//! Monotonic-anchored server-time clock.
//!
//! Mirrors `auth_client._sync_server_time / _ensure_time_sync /
//! _clock_jump_detected / _timestamp_ms` from the Python live runtime.
//!
//! ## Why
//!
//! Binance rejects signed requests whose `timestamp` falls outside
//! `[serverTime - 1000ms, serverTime + recvWindow]` (error `-1021`). System
//! wall clocks drift, are NTP-corrected, and freeze across VM
//! suspend/resume. We anchor `serverTime` once to a monotonic clock and
//! compute every subsequent timestamp as
//!     `serverTimeAtSync + (monotonic_now - monotonic_at_sync)`.
//! That is immune to wall-clock edits between syncs.
//!
//! Resync triggers (matching Python):
//!   - first call (lazy initial sync),
//!   - explicit `ensure_synced(force=true)` (used after `-1021`),
//!   - more than 5 minutes since last sync,
//!   - clock jump: `|wall_delta - monotonic_delta| > 1000 ms`
//!     (catches NTP corrections / VM resume).
//!
//! Logging matches Python — a one-line warning to stderr when |offset| > 500 ms.
//!
//! ## Locking
//!
//! One `Mutex<Inner>` guards the cached anchor data. The HTTP fetch happens
//! while holding the lock — same pattern as Python `_time_sync_lock`. With
//! a single engine thread doing the work and an unsigned, fast time endpoint,
//! the contention cost is negligible compared to the consistency it buys
//! (no two threads racing on `do_sync` and clobbering each other's state).

use std::fmt;
use std::sync::{Arc, Mutex};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use chrono::{DateTime, Utc};
use claude_trader_models::ms_to_dt_opt;

use crate::error::Result;

const SYNC_INTERVAL_MS: i64 = 300_000;
const CLOCK_JUMP_THRESHOLD_MS: i64 = 1_000;
const WALL_OFFSET_LOG_THRESHOLD_MS: i64 = 500;

// ---------------------------------------------------------------------------
// Trait surface — injection points for production HTTP and for tests
// ---------------------------------------------------------------------------

/// Source of "what time does Binance think it is" expressed as ms since
/// UNIX epoch. The production impl wraps `GET /fapi/v1/time`; tests inject
/// a canned fetcher.
pub trait ServerTimeFetcher: Send + Sync {
    fn fetch_ms(&self) -> Result<i64>;
}

/// Monotonic + wall clock pair. The monotonic value is opaque (only deltas
/// matter); the wall value is ms since UNIX epoch.
///
/// Real `std::time::Instant` cannot be constructed for tests, so we expose
/// our own ms-based abstraction.
pub trait Clock: Send + Sync {
    fn monotonic_ms(&self) -> i64;
    fn wall_ms(&self) -> i64;
}

/// Production `Clock`: monotonic is `Instant::elapsed_from_anchor`, wall is
/// `SystemTime::now() - UNIX_EPOCH`.
pub struct SystemClock {
    anchor: Instant,
}

impl SystemClock {
    pub fn new() -> Self {
        Self {
            anchor: Instant::now(),
        }
    }
}

impl Default for SystemClock {
    fn default() -> Self {
        Self::new()
    }
}

impl Clock for SystemClock {
    fn monotonic_ms(&self) -> i64 {
        self.anchor.elapsed().as_millis() as i64
    }
    fn wall_ms(&self) -> i64 {
        match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(d) => d.as_millis() as i64,
            // Pre-epoch wall clock (e.g. RTC reset to 1970). Negative offset
            // is fine for our delta math; cap at 0 here.
            Err(_) => 0,
        }
    }
}

// ---------------------------------------------------------------------------
// ServerTime
// ---------------------------------------------------------------------------

pub struct ServerTime {
    fetcher: Arc<dyn ServerTimeFetcher>,
    clock: Arc<dyn Clock>,
    inner: Mutex<Inner>,
}

struct Inner {
    initialized: bool,
    server_time_at_sync_ms: i64,
    monotonic_at_sync_ms: i64,
    last_sync_wall_ms: i64,
    last_sync_monotonic_ms: i64,
}

impl Inner {
    fn new() -> Self {
        Self {
            initialized: false,
            server_time_at_sync_ms: 0,
            monotonic_at_sync_ms: 0,
            last_sync_wall_ms: 0,
            last_sync_monotonic_ms: 0,
        }
    }
}

impl fmt::Debug for ServerTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ServerTime")
            .field("initialized", &self.inner.lock().unwrap().initialized)
            .finish()
    }
}

impl ServerTime {
    pub fn new(fetcher: Arc<dyn ServerTimeFetcher>, clock: Arc<dyn Clock>) -> Self {
        Self {
            fetcher,
            clock,
            inner: Mutex::new(Inner::new()),
        }
    }

    /// Convenience: production wiring with `SystemClock`.
    pub fn with_system_clock(fetcher: Arc<dyn ServerTimeFetcher>) -> Self {
        Self::new(fetcher, Arc::new(SystemClock::new()))
    }

    /// Current server time in ms since UNIX epoch. Lazily syncs on first call
    /// and on the documented resync triggers; never panics.
    pub fn now_ms(&self) -> i64 {
        self.ensure_synced(false);
        let inner = self.inner.lock().unwrap();
        if inner.initialized {
            let elapsed = self
                .clock
                .monotonic_ms()
                .saturating_sub(inner.monotonic_at_sync_ms);
            inner.server_time_at_sync_ms.saturating_add(elapsed)
        } else {
            // Sync failed and we have no anchor yet. Fall back to wall clock
            // — same as Python `_timestamp_ms`. Subsequent calls will retry.
            self.clock.wall_ms()
        }
    }

    pub fn now_utc(&self) -> DateTime<Utc> {
        ms_to_dt_opt(self.now_ms()).unwrap_or_else(Utc::now)
    }

    /// Trigger a resync if any of the documented conditions hold. Failure
    /// is logged to stderr; not propagated (Python parity — the engine keeps
    /// running on transient time-endpoint flakiness).
    pub fn ensure_synced(&self, force: bool) {
        let mut inner = self.inner.lock().unwrap();
        if !self.needs_sync(force, &inner) {
            return;
        }
        self.do_sync(&mut inner);
    }

    fn needs_sync(&self, force: bool, inner: &Inner) -> bool {
        if force || !inner.initialized {
            return true;
        }
        let mono_now = self.clock.monotonic_ms();
        let elapsed = mono_now - inner.last_sync_monotonic_ms;
        if elapsed >= SYNC_INTERVAL_MS {
            return true;
        }
        // Clock-jump detection: compare wall-clock delta to monotonic delta.
        // If they diverge by >1s the wall clock was edited (NTP step, manual
        // change, VM resume) — we trust the monotonic delta and need a fresh
        // anchor. The Python code's `last_sync_monotonic_ms > 0` guard was
        // redundant — `inner.initialized` already established that we have
        // valid anchor data.
        let wall_now = self.clock.wall_ms();
        let wall_delta = wall_now - inner.last_sync_wall_ms;
        let mono_delta = elapsed;
        if (wall_delta - mono_delta).abs() > CLOCK_JUMP_THRESHOLD_MS {
            eprintln!("Detected local clock jump; re-syncing with Binance.");
            return true;
        }
        false
    }

    fn do_sync(&self, inner: &mut Inner) {
        let mono_before = self.clock.monotonic_ms();
        let server_ms = match self.fetcher.fetch_ms() {
            Ok(ms) => ms,
            Err(e) => {
                eprintln!("Failed to sync server time: {e}");
                return;
            }
        };
        let mono_after = self.clock.monotonic_ms();
        let wall_now = self.clock.wall_ms();

        // Anchor at the midpoint of the round-trip — same as Python.
        inner.server_time_at_sync_ms = server_ms;
        inner.monotonic_at_sync_ms = (mono_before + mono_after) / 2;
        inner.last_sync_monotonic_ms = mono_after;
        inner.last_sync_wall_ms = wall_now;
        inner.initialized = true;

        let wall_offset = server_ms - wall_now;
        if wall_offset.abs() > WALL_OFFSET_LOG_THRESHOLD_MS {
            eprintln!("Clock offset: {wall_offset:+}ms (synced with Binance)");
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicI64, Ordering};

    /// Mock fetcher: returns `responses[i % len]` on call i, increments counter.
    struct MockFetcher {
        responses: Vec<Result<i64>>,
        calls: AtomicI64,
    }

    impl MockFetcher {
        fn always(ms: i64) -> Arc<Self> {
            Arc::new(Self {
                responses: vec![Ok(ms)],
                calls: AtomicI64::new(0),
            })
        }
        fn sequence(values: Vec<i64>) -> Arc<Self> {
            Arc::new(Self {
                responses: values.into_iter().map(Ok).collect(),
                calls: AtomicI64::new(0),
            })
        }
        fn calls(&self) -> i64 {
            self.calls.load(Ordering::SeqCst)
        }
    }

    impl ServerTimeFetcher for MockFetcher {
        fn fetch_ms(&self) -> Result<i64> {
            let i = self.calls.fetch_add(1, Ordering::SeqCst) as usize;
            // `Result<T>` from our error module isn't Clone, so re-build the
            // outcome by code branch instead of cloning.
            match &self.responses[i.min(self.responses.len() - 1)] {
                Ok(ms) => Ok(*ms),
                // Plumbed for completeness; not exercised by current tests.
                Err(e) => Err(crate::error::LiveError::Http(format!("mock: {e}"))),
            }
        }
    }

    /// Mock clock: caller advances monotonic and wall independently via setters.
    struct MockClock {
        monotonic: AtomicI64,
        wall: AtomicI64,
    }
    impl MockClock {
        fn new(initial_mono: i64, initial_wall: i64) -> Arc<Self> {
            Arc::new(Self {
                monotonic: AtomicI64::new(initial_mono),
                wall: AtomicI64::new(initial_wall),
            })
        }
        fn advance_mono(&self, ms: i64) {
            self.monotonic.fetch_add(ms, Ordering::SeqCst);
        }
        fn advance_wall(&self, ms: i64) {
            self.wall.fetch_add(ms, Ordering::SeqCst);
        }
    }
    impl Clock for MockClock {
        fn monotonic_ms(&self) -> i64 {
            self.monotonic.load(Ordering::SeqCst)
        }
        fn wall_ms(&self) -> i64 {
            self.wall.load(Ordering::SeqCst)
        }
    }

    #[test]
    fn first_call_triggers_sync() {
        let fetcher = MockFetcher::always(1_700_000_000_000);
        let clock = MockClock::new(1_000, 1_700_000_000_500);
        let st = ServerTime::new(fetcher.clone(), clock.clone());

        assert_eq!(fetcher.calls(), 0);
        let _ = st.now_ms();
        assert_eq!(fetcher.calls(), 1);
    }

    #[test]
    fn second_call_does_not_resync_within_window() {
        let fetcher = MockFetcher::always(1_700_000_000_000);
        let clock = MockClock::new(0, 1_700_000_000_000);
        let st = ServerTime::new(fetcher.clone(), clock.clone());

        let _ = st.now_ms();
        clock.advance_mono(60_000); // 60 s < 5 min
        clock.advance_wall(60_000);
        let _ = st.now_ms();
        assert_eq!(fetcher.calls(), 1);
    }

    #[test]
    fn five_minute_interval_triggers_resync() {
        let fetcher = MockFetcher::sequence(vec![1_700_000_000_000, 1_700_000_300_000]);
        let clock = MockClock::new(0, 1_700_000_000_000);
        let st = ServerTime::new(fetcher.clone(), clock.clone());

        let _ = st.now_ms();
        clock.advance_mono(SYNC_INTERVAL_MS);
        clock.advance_wall(SYNC_INTERVAL_MS);
        let _ = st.now_ms();
        assert_eq!(fetcher.calls(), 2);
    }

    #[test]
    fn forced_resync_fires_even_within_window() {
        let fetcher = MockFetcher::sequence(vec![1, 2, 3]);
        let clock = MockClock::new(0, 1_700_000_000_000);
        let st = ServerTime::new(fetcher.clone(), clock.clone());

        let _ = st.now_ms(); // call 1
        st.ensure_synced(true); // call 2
        st.ensure_synced(true); // call 3
        assert_eq!(fetcher.calls(), 3);
    }

    #[test]
    fn clock_jump_detection_triggers_resync() {
        let fetcher = MockFetcher::sequence(vec![1_700_000_000_000, 1_700_000_010_000]);
        let clock = MockClock::new(0, 1_700_000_000_000);
        let st = ServerTime::new(fetcher.clone(), clock.clone());

        let _ = st.now_ms();
        // Wall clock jumps forward by 10 s while monotonic only advanced 5 s.
        clock.advance_mono(5_000);
        clock.advance_wall(10_000);
        let _ = st.now_ms();
        assert_eq!(fetcher.calls(), 2, "clock jump should have forced resync");
    }

    #[test]
    fn no_resync_when_clock_jump_is_below_threshold() {
        let fetcher = MockFetcher::always(1_700_000_000_000);
        let clock = MockClock::new(0, 1_700_000_000_000);
        let st = ServerTime::new(fetcher.clone(), clock.clone());

        let _ = st.now_ms();
        // 700ms divergence — under the 1000ms threshold.
        clock.advance_mono(5_000);
        clock.advance_wall(5_700);
        let _ = st.now_ms();
        assert_eq!(fetcher.calls(), 1);
    }

    #[test]
    fn now_ms_returns_anchored_time_after_sync() {
        let server_at_sync = 1_700_000_000_000;
        let fetcher = MockFetcher::always(server_at_sync);
        let clock = MockClock::new(500, 1_700_000_000_000);
        let st = ServerTime::new(fetcher, clock.clone());

        let _ = st.now_ms(); // sync at monotonic_ms=500
        // Advance wall *and* mono together — that simulates real time passing
        // without triggering the jump-detection resync.
        clock.advance_mono(2_500);
        clock.advance_wall(2_500);
        let now = st.now_ms();
        // Allow ±1ms tolerance for the (mono_before+mono_after)/2 anchoring.
        assert!(
            (now - (server_at_sync + 2_500)).abs() <= 1,
            "expected ~{}, got {}",
            server_at_sync + 2_500,
            now,
        );
    }

    #[test]
    fn falls_back_to_wall_clock_when_initial_sync_fails() {
        struct FailingFetcher;
        impl ServerTimeFetcher for FailingFetcher {
            fn fetch_ms(&self) -> Result<i64> {
                Err(crate::error::LiveError::Http("offline".into()))
            }
        }
        let clock = MockClock::new(0, 1_700_000_111_222);
        let st = ServerTime::new(Arc::new(FailingFetcher), clock);
        // No anchor: now_ms must return wall_ms, not panic.
        assert_eq!(st.now_ms(), 1_700_000_111_222);
    }

    #[test]
    fn now_utc_returns_chrono_datetime() {
        let fetcher = MockFetcher::always(1_700_000_000_000);
        let clock = MockClock::new(0, 1_700_000_000_000);
        let st = ServerTime::new(fetcher, clock);
        let dt = st.now_utc();
        assert_eq!(dt.timestamp_millis(), 1_700_000_000_000);
    }
}
