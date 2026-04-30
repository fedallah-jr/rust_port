//! Shared types for the claude-trader Rust port.
//!
//! Every type here mirrors a Python counterpart. Field names, defaults, and
//! semantics match the Python originals exactly. See:
//!   - `backtester/models.py`
//!   - `live/models.py`
//!   - `marketdata/models.py`
//!   - `backtester/pipeline.py`

pub mod backtester;
pub mod context;
pub mod cooldown;
pub mod funding;
pub mod live;
pub mod marketdata;

pub mod interval;

pub use backtester::*;
pub use context::{ContextKey, ContextMap, ContextValue, SeriesInput};
pub use cooldown::{CooldownKey, CooldownSpec};
pub use funding::{funding_context_at, FundingContext};
pub use interval::{
    floor_boundary, next_boundary, parse_interval_duration, parse_interval_seconds,
};
pub use live::{
    AccountTrade, ConfigError, ExchangeOrder, GeneratorBudget, LiveConfig, LivePosition,
    OrderSide, OrderStatus, OrderType, PositionStatus, LEGACY_TESTNET_BASE_URL, PROD_BASE_URL,
    TESTNET_BASE_URL,
};
pub use marketdata::*;

// ---------------------------------------------------------------------------
// DateTime / millisecond helpers
// ---------------------------------------------------------------------------

use chrono::{DateTime, TimeZone, Utc};

/// Convert `DateTime<Utc>` to milliseconds since epoch.
#[inline]
pub fn dt_to_ms(dt: DateTime<Utc>) -> i64 {
    dt.timestamp_millis()
}

/// Convert milliseconds since epoch to `DateTime<Utc>`.
///
/// Panics if `ms` is outside the representable chrono range. Use this for
/// timestamps that are already known-valid (e.g. produced by `dt_to_ms`,
/// read back from our own on-disk stores). For parsing external input
/// (Binance API responses, raw JSON), use `ms_to_dt_opt` and surface the
/// failure as a typed error — silently collapsing to `1970-01-01` masks
/// data bugs.
#[inline]
pub fn ms_to_dt(ms: i64) -> DateTime<Utc> {
    match Utc.timestamp_millis_opt(ms).single() {
        Some(dt) => dt,
        None => panic!("ms_to_dt: ms={ms} out of valid range"),
    }
}

/// Fallible version of `ms_to_dt` for external-input boundaries.
#[inline]
pub fn ms_to_dt_opt(ms: i64) -> Option<DateTime<Utc>> {
    Utc.timestamp_millis_opt(ms).single()
}

#[cfg(test)]
mod ms_conversion_tests {
    use super::*;

    #[test]
    fn ms_to_dt_opt_accepts_valid() {
        let got = ms_to_dt_opt(1_700_000_000_000).unwrap();
        assert_eq!(dt_to_ms(got), 1_700_000_000_000);
    }

    #[test]
    fn ms_to_dt_opt_rejects_out_of_range() {
        assert!(ms_to_dt_opt(i64::MAX).is_none());
        assert!(ms_to_dt_opt(i64::MIN).is_none());
    }

    #[test]
    #[should_panic(expected = "ms_to_dt: ms=")]
    fn ms_to_dt_panics_on_out_of_range() {
        let _ = ms_to_dt(i64::MAX);
    }
}
