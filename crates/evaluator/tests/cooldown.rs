//! Regression tests for `enforce_signal_cooldown`.
//!
//! Covers the public key constructors (symbol, symbol+side, symbol+pattern,
//! custom), the pass-through for `hours == 0.0`, the debug-build panic on
//! inconsistent hours for the same key, and the runtime panic on invalid
//! hours.

use std::sync::Arc;

use chrono::{DateTime, Duration, TimeZone, Utc};
use claude_trader_evaluator::cooldown::enforce_signal_cooldown;
use claude_trader_models::{
    CooldownKey, CooldownSpec, MarketType, PositionType, Signal,
};

fn t(offset_hours: i64) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap() + Duration::hours(offset_hours)
}

fn mk(ticker: &str, side: PositionType, pattern: &str, at: DateTime<Utc>) -> Arc<Signal> {
    Arc::new(Signal {
        signal_date: at,
        position_type: side,
        ticker: ticker.to_string(),
        pattern: pattern.to_string(),
        tp_pct: Some(3.0),
        sl_pct: Some(1.5),
        tp_price: None,
        sl_price: None,
        leverage: 1.0,
        market_type: MarketType::Futures,
        taker_fee_rate: 0.0005,
        entry_price: None,
        fill_timeout_seconds: 3600,
        entry_delay_seconds: None,
        max_holding_hours: 72,
        size_multiplier: 1.0,
        metadata: Default::default(),
    })
}

#[test]
fn symbol_only_blocks_opposite_side() {
    // key = Symbol(BTC) — long then short on the same symbol share the bucket.
    let signals = vec![
        mk("BTCUSDT", PositionType::Long, "a", t(0)),
        mk("BTCUSDT", PositionType::Short, "b", t(1)),
        mk("BTCUSDT", PositionType::Long, "c", t(25)),
    ];
    let out = enforce_signal_cooldown(&signals, |s| {
        CooldownSpec {
            key: CooldownKey::symbol(&s.ticker),
            hours: 24.0,
        }
    });
    assert_eq!(out.len(), 2);
    assert_eq!(out[0].pattern, "a");
    assert_eq!(out[1].pattern, "c");
}

#[test]
fn symbol_side_allows_opposite_side() {
    // Default grouping: long and short on the same symbol are independent buckets.
    let signals = vec![
        mk("BTCUSDT", PositionType::Long, "a", t(0)),
        mk("BTCUSDT", PositionType::Short, "b", t(1)),
        mk("BTCUSDT", PositionType::Long, "c", t(2)),
    ];
    let out = enforce_signal_cooldown(&signals, |s| CooldownSpec::symbol_side(s, 24.0));
    assert_eq!(out.len(), 2);
    assert_eq!(out[0].pattern, "a");
    assert_eq!(out[1].pattern, "b");
}

#[test]
fn symbol_pattern_allows_different_pattern() {
    let signals = vec![
        mk("BTCUSDT", PositionType::Long, "breakout", t(0)),
        mk("BTCUSDT", PositionType::Long, "mean_revert", t(1)),
        mk("BTCUSDT", PositionType::Long, "breakout", t(2)),
    ];
    let out = enforce_signal_cooldown(&signals, |s| CooldownSpec::symbol_pattern(s, 24.0));
    assert_eq!(out.len(), 2);
    assert_eq!(out[0].pattern, "breakout");
    assert_eq!(out[1].pattern, "mean_revert");
}

#[test]
fn custom_key_driven_by_metadata() {
    let mut signals = vec![
        mk("BTCUSDT", PositionType::Long, "a", t(0)),
        mk("BTCUSDT", PositionType::Long, "b", t(1)),
        mk("BTCUSDT", PositionType::Long, "c", t(2)),
    ];
    // Tag "tier:high" on signals 0 and 2; "tier:low" on signal 1.
    Arc::get_mut(&mut signals[0])
        .unwrap()
        .metadata
        .insert("tier".into(), serde_json::json!("high"));
    Arc::get_mut(&mut signals[1])
        .unwrap()
        .metadata
        .insert("tier".into(), serde_json::json!("low"));
    Arc::get_mut(&mut signals[2])
        .unwrap()
        .metadata
        .insert("tier".into(), serde_json::json!("high"));

    let out = enforce_signal_cooldown(&signals, |s| {
        let tier = s.metadata.get("tier").and_then(|v| v.as_str()).unwrap_or("unknown");
        CooldownSpec {
            key: CooldownKey::custom(tier),
            hours: 24.0,
        }
    });
    assert_eq!(out.len(), 2);
    assert_eq!(out[0].pattern, "a");
    assert_eq!(out[1].pattern, "b");
}

#[test]
fn zero_hours_passes_everything_through() {
    let signals = vec![
        mk("BTCUSDT", PositionType::Long, "a", t(0)),
        mk("BTCUSDT", PositionType::Long, "b", t(0)),
        mk("BTCUSDT", PositionType::Long, "c", t(0)),
    ];
    let out = enforce_signal_cooldown(&signals, |s| CooldownSpec::symbol_side(s, 0.0));
    assert_eq!(out.len(), 3);
}

#[test]
fn cooldown_boundary_is_inclusive() {
    // Exactly `hours` apart — the second signal must be admitted.
    let signals = vec![
        mk("BTCUSDT", PositionType::Long, "a", t(0)),
        mk("BTCUSDT", PositionType::Long, "b", t(24)),
    ];
    let out = enforce_signal_cooldown(&signals, |s| CooldownSpec::symbol_side(s, 24.0));
    assert_eq!(out.len(), 2);
}

#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "cooldown hours inconsistent")]
fn debug_asserts_on_hours_mismatch_for_same_key() {
    use std::cell::Cell;
    // Two signals share the same key but the spec returns different hours on
    // the second call. Debug builds must panic to surface the bug.
    let signals = vec![
        mk("BTCUSDT", PositionType::Long, "a", t(0)),
        mk("BTCUSDT", PositionType::Long, "b", t(48)),
    ];
    let call = Cell::new(0u32);
    enforce_signal_cooldown(&signals, |s| {
        call.set(call.get() + 1);
        let hours = if call.get() == 1 { 24.0 } else { 12.0 };
        CooldownSpec::symbol_side(s, hours)
    });
}

#[test]
#[should_panic(expected = "finite and non-negative")]
fn panics_on_nan_hours() {
    let signals = vec![mk("BTCUSDT", PositionType::Long, "a", t(0))];
    enforce_signal_cooldown(&signals, |s| CooldownSpec::symbol_side(s, f64::NAN));
}

#[test]
#[should_panic(expected = "finite and non-negative")]
fn panics_on_negative_hours() {
    let signals = vec![mk("BTCUSDT", PositionType::Long, "a", t(0))];
    enforce_signal_cooldown(&signals, |s| CooldownSpec::symbol_side(s, -1.0));
}

#[test]
#[should_panic(expected = "finite and non-negative")]
fn panics_on_infinite_hours() {
    let signals = vec![mk("BTCUSDT", PositionType::Long, "a", t(0))];
    enforce_signal_cooldown(&signals, |s| CooldownSpec::symbol_side(s, f64::INFINITY));
}

#[test]
fn preserves_input_order_among_survivors() {
    let signals = vec![
        mk("BTCUSDT", PositionType::Long, "a", t(0)),
        mk("ETHUSDT", PositionType::Long, "b", t(1)),
        mk("SOLUSDT", PositionType::Long, "c", t(2)),
    ];
    let out = enforce_signal_cooldown(&signals, |s| CooldownSpec::symbol_side(s, 24.0));
    assert_eq!(out.len(), 3);
    assert_eq!(out[0].ticker, "BTCUSDT");
    assert_eq!(out[1].ticker, "ETHUSDT");
    assert_eq!(out[2].ticker, "SOLUSDT");
}
