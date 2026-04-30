//! Order-count rate tracking from `X-MBX-ORDER-COUNT-*` headers.
//!
//! Binance USD-M futures returns rolling counts of orders placed in the last
//! 10 seconds and 1 minute on every order-endpoint response. This module
//! observes those headers and lets callers gate the next placement when
//! utilization gets close to the published limits.
//!
//! v1 policy (matches plan v2 §5):
//!   - parse the headers, store last-seen counts + timestamp,
//!   - log warn at >80% utilization, error at >95%,
//!   - `pre_check` returns `Some(sleep)` when the *next* placement would
//!     exceed the limit; the caller sleeps and retries.
//!
//! Limits come from `GET /fapi/v1/exchangeInfo`'s `rateLimits[]` array (look
//! for `rateLimitType == "ORDERS"` entries with intervalNum 10 / interval
//! "SECOND" and intervalNum 1 / interval "MINUTE"). The Phase B client
//! plumbs that read at construction; the tracker holds the resolved limits.

use std::sync::Mutex;
use std::time::{Duration, Instant};

const LOG_WARN_THRESHOLD: f64 = 0.80;
const LOG_ERROR_THRESHOLD: f64 = 0.95;
/// Default Binance documented limits — used when exchangeInfo lookup fails.
/// These are conservative ("Binance lists 300/10s / 1200/min for futures").
const DEFAULT_LIMIT_10S: u32 = 300;
const DEFAULT_LIMIT_1M: u32 = 1200;

const HEADER_10S: &str = "X-MBX-ORDER-COUNT-10S";
const HEADER_1M: &str = "X-MBX-ORDER-COUNT-1M";

/// Read access to header maps in a way that is compatible with both
/// `reqwest::header::HeaderMap` and our test helpers (which use plain
/// `HashMap<String, String>`). Implementing it for both types means the
/// production hot path stays zero-copy on `HeaderMap` while tests can pass
/// hand-built maps.
pub trait HeaderLookup {
    fn header_lookup(&self, name: &str) -> Option<&str>;
}

impl HeaderLookup for reqwest::header::HeaderMap {
    fn header_lookup(&self, name: &str) -> Option<&str> {
        self.get(name).and_then(|v| v.to_str().ok())
    }
}

impl<'a, V: AsRef<str>> HeaderLookup for std::collections::HashMap<&'a str, V> {
    fn header_lookup(&self, name: &str) -> Option<&str> {
        self.get(name).map(|v| v.as_ref())
    }
}

/// Owned-key variant — used by the `HttpResponse.headers` map produced by
/// the transport layer. Case-insensitive name match because reqwest
/// lowercases headers and we may receive them in either form.
impl<V: AsRef<str>> HeaderLookup for std::collections::HashMap<String, V> {
    fn header_lookup(&self, name: &str) -> Option<&str> {
        for (k, v) in self {
            if k.eq_ignore_ascii_case(name) {
                return Some(v.as_ref());
            }
        }
        None
    }
}

#[derive(Debug)]
pub struct OrderCountTracker {
    inner: Mutex<Inner>,
}

#[derive(Debug)]
struct Inner {
    limit_10s: u32,
    limit_1m: u32,
    last_count_10s: Option<u32>,
    last_count_1m: Option<u32>,
    last_seen_at: Option<Instant>,
}

impl OrderCountTracker {
    /// Build with documented Binance defaults. Replace via `set_limits` after
    /// the exchangeInfo read succeeds.
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(Inner {
                limit_10s: DEFAULT_LIMIT_10S,
                limit_1m: DEFAULT_LIMIT_1M,
                last_count_10s: None,
                last_count_1m: None,
                last_seen_at: None,
            }),
        }
    }

    pub fn with_limits(limit_10s: u32, limit_1m: u32) -> Self {
        let t = Self::new();
        t.set_limits(limit_10s, limit_1m);
        t
    }

    pub fn set_limits(&self, limit_10s: u32, limit_1m: u32) {
        let mut inner = self.inner.lock().unwrap();
        inner.limit_10s = limit_10s.max(1);
        inner.limit_1m = limit_1m.max(1);
    }

    pub fn limits(&self) -> (u32, u32) {
        let i = self.inner.lock().unwrap();
        (i.limit_10s, i.limit_1m)
    }

    /// Parse rolling counts from a response and update internal state.
    /// Logs at the documented thresholds.
    pub fn record_response<H: HeaderLookup>(&self, headers: &H) {
        let count_10s = parse_u32(headers.header_lookup(HEADER_10S));
        let count_1m = parse_u32(headers.header_lookup(HEADER_1M));
        if count_10s.is_none() && count_1m.is_none() {
            return; // not an order-endpoint response (or headers stripped)
        }
        let mut inner = self.inner.lock().unwrap();
        if let Some(c) = count_10s {
            inner.last_count_10s = Some(c);
        }
        if let Some(c) = count_1m {
            inner.last_count_1m = Some(c);
        }
        inner.last_seen_at = Some(Instant::now());

        // Log at thresholds. Drop the guard before the log macros so a slow
        // log writer can't block other tracker callers.
        let limit_10s = inner.limit_10s;
        let limit_1m = inner.limit_1m;
        let last_10s = inner.last_count_10s;
        let last_1m = inner.last_count_1m;
        drop(inner);
        emit_threshold_log("10S", last_10s, limit_10s);
        emit_threshold_log("1M", last_1m, limit_1m);
    }

    /// Pre-flight check before placing an order. Returns `Some(sleep)` when
    /// adding one more order would push us past the limit; sleep duration is
    /// the time until the rolling window has rotated.
    pub fn pre_check(&self) -> Option<Duration> {
        let inner = self.inner.lock().unwrap();
        let now = Instant::now();
        if let (Some(last), Some(seen)) = (inner.last_count_10s, inner.last_seen_at) {
            if last + 1 >= inner.limit_10s {
                let elapsed = now.saturating_duration_since(seen);
                let window = Duration::from_secs(10);
                if elapsed < window {
                    return Some(window - elapsed + Duration::from_millis(100));
                }
            }
        }
        if let (Some(last), Some(seen)) = (inner.last_count_1m, inner.last_seen_at) {
            if last + 1 >= inner.limit_1m {
                let elapsed = now.saturating_duration_since(seen);
                let window = Duration::from_secs(60);
                if elapsed < window {
                    return Some(window - elapsed + Duration::from_millis(100));
                }
            }
        }
        None
    }
}

impl Default for OrderCountTracker {
    fn default() -> Self {
        Self::new()
    }
}

fn parse_u32(raw: Option<&str>) -> Option<u32> {
    raw?.trim().parse().ok()
}

fn emit_threshold_log(window: &str, count: Option<u32>, limit: u32) {
    let Some(c) = count else { return };
    if limit == 0 {
        return;
    }
    let util = c as f64 / limit as f64;
    if util >= LOG_ERROR_THRESHOLD {
        eprintln!(
            "ERROR: Binance order-count {window} = {c}/{limit} ({:.0}%); throttling imminent",
            util * 100.0,
        );
    } else if util >= LOG_WARN_THRESHOLD {
        eprintln!(
            "WARN: Binance order-count {window} = {c}/{limit} ({:.0}%)",
            util * 100.0,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn headers(pairs: &[(&'static str, &'static str)]) -> HashMap<&'static str, String> {
        pairs.iter().map(|(k, v)| (*k, v.to_string())).collect()
    }

    #[test]
    fn parses_both_headers() {
        let t = OrderCountTracker::with_limits(300, 1200);
        t.record_response(&headers(&[(HEADER_10S, "5"), (HEADER_1M, "12")]));
        let (l10, l1m) = t.limits();
        assert_eq!(l10, 300);
        assert_eq!(l1m, 1200);
        // pre_check returns None because 5+1 < 300 and 12+1 < 1200.
        assert!(t.pre_check().is_none());
    }

    #[test]
    fn ignores_non_order_responses() {
        let t = OrderCountTracker::with_limits(300, 1200);
        let h: HashMap<&'static str, String> = HashMap::new();
        t.record_response(&h);
        assert!(t.pre_check().is_none()); // no state recorded
    }

    #[test]
    fn pre_check_throttles_when_10s_count_at_limit() {
        let t = OrderCountTracker::with_limits(10, 100);
        // 9 used out of 10/10s → next placement would be 10/10s, hits limit.
        t.record_response(&headers(&[(HEADER_10S, "9"), (HEADER_1M, "9")]));
        let sleep = t.pre_check().expect("should throttle");
        // Sleep is bounded by the 10s window plus our 100ms slack.
        assert!(sleep <= Duration::from_secs(11));
        assert!(sleep > Duration::from_millis(99));
    }

    #[test]
    fn pre_check_throttles_when_1m_count_at_limit() {
        let t = OrderCountTracker::with_limits(10000, 100);
        t.record_response(&headers(&[(HEADER_10S, "1"), (HEADER_1M, "99")]));
        let sleep = t.pre_check().expect("should throttle");
        assert!(sleep <= Duration::from_secs(61));
    }

    #[test]
    fn pre_check_no_throttle_when_well_under_limit() {
        let t = OrderCountTracker::with_limits(300, 1200);
        t.record_response(&headers(&[(HEADER_10S, "5"), (HEADER_1M, "10")]));
        assert!(t.pre_check().is_none());
    }

    #[test]
    fn handles_malformed_headers_silently() {
        let t = OrderCountTracker::with_limits(300, 1200);
        t.record_response(&headers(&[(HEADER_10S, "not-a-number")]));
        assert!(t.pre_check().is_none());
    }

    #[test]
    fn limits_replaceable_post_construction() {
        let t = OrderCountTracker::new();
        let (l10, l1m) = t.limits();
        assert_eq!(l10, DEFAULT_LIMIT_10S);
        assert_eq!(l1m, DEFAULT_LIMIT_1M);
        t.set_limits(50, 500);
        assert_eq!(t.limits(), (50, 500));
    }

    #[test]
    fn limit_zero_is_clamped_to_one() {
        // Defensive: avoid division-by-zero in threshold calculation.
        let t = OrderCountTracker::with_limits(0, 0);
        let (l10, l1m) = t.limits();
        assert_eq!(l10, 1);
        assert_eq!(l1m, 1);
    }

    #[test]
    fn lookup_works_for_reqwest_header_map() {
        // Smoke test: the trait impl for HeaderMap should round-trip a value.
        let mut map = reqwest::header::HeaderMap::new();
        map.insert(
            HEADER_10S,
            reqwest::header::HeaderValue::from_static("42"),
        );
        let got = map.header_lookup(HEADER_10S);
        assert_eq!(got, Some("42"));
    }
}
