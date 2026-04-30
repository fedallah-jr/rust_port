//! Interval parsing utilities and candle-boundary arithmetic.
//!
//! Parses Binance-style interval strings ("1m", "15m", "1h", "4h", "1d")
//! into exact durations. Errors on invalid input — mirrors Python's
//! `interval_to_seconds` which raises on bad intervals.
//!
//! Boundary helpers (`floor_boundary`, `next_boundary`) mirror
//! `backtester/preview.py` and are shared between live and backtest paths.

use chrono::{DateTime, Duration, TimeZone, Utc};

/// Parse an interval string into a `chrono::Duration`.
///
/// Supports `m` (minutes), `h` (hours), and `d` (days) suffixes.
/// Returns `Err` for empty strings, unknown suffixes, or non-numeric values.
pub fn parse_interval_duration(interval: &str) -> Result<Duration, String> {
    let secs = parse_interval_seconds(interval)?;
    Ok(Duration::seconds(secs))
}

/// Parse an interval string into seconds.
///
/// Supports `m` (minutes), `h` (hours), and `d` (days) suffixes.
/// Returns `Err` for empty strings, unknown suffixes, or non-numeric values.
pub fn parse_interval_seconds(interval: &str) -> Result<i64, String> {
    if interval.is_empty() {
        return Err("empty interval string".to_string());
    }
    if !interval.is_ascii() {
        return Err(format!("interval must be ASCII: {interval:?}"));
    }
    let (num_str, suffix) = interval.split_at(interval.len() - 1);
    let value: i64 = num_str
        .parse()
        .map_err(|_| format!("invalid interval number: {interval:?}"))?;
    if value <= 0 {
        return Err(format!("interval value must be positive: {interval:?}"));
    }
    match suffix {
        "m" => Ok(value * 60),
        "h" => Ok(value * 3600),
        "d" => Ok(value * 86400),
        _ => Err(format!("unsupported interval suffix: {interval:?}")),
    }
}

/// Floor a UTC instant down to the most recent boundary of `interval`.
///
/// Mirrors Python `backtester.preview.floor_boundary`. Boundaries are aligned
/// to the Unix epoch — `floor_boundary(2026-04-30T12:34:56Z, "1h")` returns
/// `2026-04-30T12:00:00Z`, `..., "15m")` returns `12:30:00Z`, `..., "1d")`
/// returns `2026-04-30T00:00:00Z`.
///
/// Panics only for invalid `interval` strings; valid intervals always produce
/// a representable timestamp because we operate in seconds-since-epoch.
pub fn floor_boundary(dt: DateTime<Utc>, interval: &str) -> Result<DateTime<Utc>, String> {
    let chunk_seconds = parse_interval_seconds(interval)?;
    let ts = dt.timestamp();
    let floored = ts - ts.rem_euclid(chunk_seconds);
    Utc.timestamp_opt(floored, 0)
        .single()
        .ok_or_else(|| format!("floored timestamp {floored} out of range"))
}

/// Advance to the next boundary strictly after `dt`.
///
/// Equal to `floor_boundary(dt, interval) + interval`. If `dt` already sits
/// exactly on a boundary, returns the next one (matches Python).
pub fn next_boundary(dt: DateTime<Utc>, interval: &str) -> Result<DateTime<Utc>, String> {
    let floor = floor_boundary(dt, interval)?;
    let step = parse_interval_duration(interval)?;
    Ok(floor + step)
}

#[cfg(test)]
mod boundary_tests {
    use super::*;
    use chrono::TimeZone;

    fn ts(s: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(s)
            .unwrap()
            .with_timezone(&Utc)
    }

    #[test]
    fn floor_1h() {
        assert_eq!(
            floor_boundary(ts("2026-04-30T12:34:56Z"), "1h").unwrap(),
            ts("2026-04-30T12:00:00Z"),
        );
    }

    #[test]
    fn floor_15m() {
        assert_eq!(
            floor_boundary(ts("2026-04-30T12:34:56Z"), "15m").unwrap(),
            ts("2026-04-30T12:30:00Z"),
        );
        assert_eq!(
            floor_boundary(ts("2026-04-30T12:14:59Z"), "15m").unwrap(),
            ts("2026-04-30T12:00:00Z"),
        );
    }

    #[test]
    fn floor_1d() {
        assert_eq!(
            floor_boundary(ts("2026-04-30T12:34:56Z"), "1d").unwrap(),
            ts("2026-04-30T00:00:00Z"),
        );
    }

    #[test]
    fn floor_on_boundary_is_idempotent() {
        let exact = ts("2026-04-30T12:00:00Z");
        assert_eq!(floor_boundary(exact, "1h").unwrap(), exact);
    }

    #[test]
    fn next_advances_past_boundary() {
        assert_eq!(
            next_boundary(ts("2026-04-30T12:00:00Z"), "1h").unwrap(),
            ts("2026-04-30T13:00:00Z"),
        );
        assert_eq!(
            next_boundary(ts("2026-04-30T12:34:56Z"), "1h").unwrap(),
            ts("2026-04-30T13:00:00Z"),
        );
    }

    #[test]
    fn floor_handles_pre_epoch() {
        // 1969-12-31T23:59:59Z is one second before the epoch. With
        // rem_euclid the floor must round toward negative infinity, not
        // toward zero (which would land on 1970-01-01 — a bug).
        let pre = Utc.timestamp_opt(-1, 0).single().unwrap();
        let floored = floor_boundary(pre, "1h").unwrap();
        assert_eq!(floored, Utc.timestamp_opt(-3600, 0).single().unwrap());
    }

    #[test]
    fn boundary_propagates_invalid_interval() {
        assert!(floor_boundary(ts("2026-04-30T12:00:00Z"), "1w").is_err());
        assert!(next_boundary(ts("2026-04-30T12:00:00Z"), "").is_err());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_minutes() {
        assert_eq!(parse_interval_seconds("1m").unwrap(), 60);
        assert_eq!(parse_interval_seconds("15m").unwrap(), 900);
        assert_eq!(parse_interval_seconds("30m").unwrap(), 1800);
    }

    #[test]
    fn test_hours() {
        assert_eq!(parse_interval_seconds("1h").unwrap(), 3600);
        assert_eq!(parse_interval_seconds("4h").unwrap(), 14400);
    }

    #[test]
    fn test_days() {
        assert_eq!(parse_interval_seconds("1d").unwrap(), 86400);
        assert_eq!(parse_interval_seconds("7d").unwrap(), 604800);
    }

    #[test]
    fn test_duration() {
        assert_eq!(
            parse_interval_duration("15m").unwrap(),
            Duration::minutes(15)
        );
        assert_eq!(parse_interval_duration("4h").unwrap(), Duration::hours(4));
        assert_eq!(parse_interval_duration("1d").unwrap(), Duration::days(1));
    }

    #[test]
    fn test_invalid() {
        assert!(parse_interval_seconds("").is_err());
        assert!(parse_interval_seconds("xyz").is_err());
        assert!(parse_interval_seconds("1w").is_err());
        assert!(parse_interval_seconds("0h").is_err());
        assert!(parse_interval_seconds("-1h").is_err());
        assert!(parse_interval_seconds("h").is_err());
    }

    #[test]
    fn test_warmup_math_4h() {
        // Verify that warmup_bars * interval_duration produces correct offsets.
        // 100 warmup bars at 4h = 400 hours = 16.67 days
        let dur = parse_interval_duration("4h").unwrap();
        let warmup = dur * 100i32;
        assert_eq!(warmup, Duration::hours(400));
    }

    #[test]
    fn test_warmup_math_15m() {
        // 100 warmup bars at 15m = 1500 minutes = 25 hours
        let dur = parse_interval_duration("15m").unwrap();
        let warmup = dur * 100i32;
        assert_eq!(warmup, Duration::minutes(1500));
        assert_eq!(warmup, Duration::hours(25));
    }

    #[test]
    fn test_calibration_bars_math() {
        // ceil(72 lookback hours / 4h) = 18 bars
        let secs = parse_interval_seconds("4h").unwrap();
        let bars = (72.0 * 3600.0 / secs as f64).ceil() as usize;
        assert_eq!(bars, 18);

        // ceil(72 lookback hours / 15m) = 288 bars
        let secs = parse_interval_seconds("15m").unwrap();
        let bars = (72.0 * 3600.0 / secs as f64).ceil() as usize;
        assert_eq!(bars, 288);

        // ceil(72 lookback hours / 1h) = 72 bars (identity)
        let secs = parse_interval_seconds("1h").unwrap();
        let bars = (72.0 * 3600.0 / secs as f64).ceil() as usize;
        assert_eq!(bars, 72);
    }
}
