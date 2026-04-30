//! 3-level hierarchical exit resolver: HOUR → MINUTE → TRADE.
//!
//! Ported from `resolver_rs/src/lib.rs`. This version uses native Rust types
//! and data-passing (no PyO3 callbacks). Callers provide slices of candles
//! and trades directly.

use crate::ms_to_dt;
use claude_trader_models::{ExitReason, ExitResolution, ResolutionLevel};

// ---------------------------------------------------------------------------
// Internal types
// ---------------------------------------------------------------------------

#[derive(Clone, Copy)]
pub struct MiniCandle {
    pub open_time_ms: i64,
    pub close_time_ms: i64,
    pub high: f64,
    pub low: f64,
}

#[derive(Clone, Copy)]
pub struct MiniTrade {
    pub timestamp_ms: i64,
    pub price: f64,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Barrier {
    Open,
    Tp,
    Sl,
    Ambiguous,
}

#[derive(Clone, Copy)]
struct Resolution {
    is_tp: bool,
    time_ms: i64,
    price: f64,
    level: ResolutionLevel,
    exit_fallback: bool,
    random_resolved: bool,
}

impl Resolution {
    fn to_exit_resolution(&self) -> ExitResolution {
        ExitResolution {
            reason: if self.is_tp {
                ExitReason::Tp
            } else {
                ExitReason::Sl
            },
            exit_time: ms_to_dt(self.time_ms),
            exit_price: self.price,
            resolution_level: self.level,
            exit_fallback: self.exit_fallback,
            random_resolved: self.random_resolved,
        }
    }
}

// ---------------------------------------------------------------------------
// Time helpers
// ---------------------------------------------------------------------------

#[inline(always)]
fn floor_hour(ms: i64) -> i64 {
    ms - ms.rem_euclid(3_600_000)
}

#[inline(always)]
fn floor_minute(ms: i64) -> i64 {
    ms - ms.rem_euclid(60_000)
}

// ---------------------------------------------------------------------------
// XORShift64 PRNG (same as resolver_rs)
// ---------------------------------------------------------------------------

#[inline]
fn xorshift64(state: &mut u64) -> u64 {
    let mut x = *state;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    *state = x;
    x
}

// ---------------------------------------------------------------------------
// Barrier detection
// ---------------------------------------------------------------------------

#[inline]
fn barrier_outcome(c: &MiniCandle, is_long: bool, tp: f64, sl: f64) -> Barrier {
    let (tp_hit, sl_hit) = if is_long {
        (c.high >= tp, c.low <= sl)
    } else {
        (c.low <= tp, c.high >= sl)
    };
    match (tp_hit, sl_hit) {
        (false, false) => Barrier::Open,
        (true, false) => Barrier::Tp,
        (false, true) => Barrier::Sl,
        (true, true) => Barrier::Ambiguous,
    }
}

fn barrier_outcome_multi(candles: &[MiniCandle], is_long: bool, tp: f64, sl: f64) -> Barrier {
    let mut tp_hit = false;
    let mut sl_hit = false;
    for c in candles {
        if is_long {
            tp_hit = tp_hit || c.high >= tp;
            sl_hit = sl_hit || c.low <= sl;
        } else {
            tp_hit = tp_hit || c.low <= tp;
            sl_hit = sl_hit || c.high >= sl;
        }
        if tp_hit && sl_hit {
            return Barrier::Ambiguous;
        }
    }
    match (tp_hit, sl_hit) {
        (true, false) => Barrier::Tp,
        (false, true) => Barrier::Sl,
        _ => Barrier::Open,
    }
}

// ---------------------------------------------------------------------------
// Trade-level resolution
// ---------------------------------------------------------------------------

fn resolve_with_trades(
    trades: &[MiniTrade],
    is_long: bool,
    tp: f64,
    sl: f64,
    start_ms: i64,
) -> Option<Resolution> {
    for t in trades {
        if t.timestamp_ms < start_ms {
            continue;
        }
        let hit = if is_long {
            if t.price >= tp {
                Some(true)
            } else if t.price <= sl {
                Some(false)
            } else {
                None
            }
        } else if t.price <= tp {
            Some(true)
        } else if t.price >= sl {
            Some(false)
        } else {
            None
        };
        if let Some(is_tp) = hit {
            return Some(Resolution {
                is_tp,
                time_ms: t.timestamp_ms,
                price: if is_tp { tp } else { sl },
                level: ResolutionLevel::Trade,
                exit_fallback: false,
                random_resolved: false,
            });
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Minute-level resolution
// ---------------------------------------------------------------------------

fn resolve_candles_minute(
    candles: &[MiniCandle],
    is_long: bool,
    tp: f64,
    sl: f64,
    fetch_trades: &mut dyn FnMut(i64, i64) -> Vec<MiniTrade>,
) -> Option<Resolution> {
    for c in candles {
        match barrier_outcome(c, is_long, tp, sl) {
            Barrier::Open => continue,
            Barrier::Tp => {
                return Some(Resolution {
                    is_tp: true,
                    time_ms: c.close_time_ms,
                    price: tp,
                    level: ResolutionLevel::Minute,
                    exit_fallback: false,
                    random_resolved: false,
                });
            }
            Barrier::Sl => {
                return Some(Resolution {
                    is_tp: false,
                    time_ms: c.close_time_ms,
                    price: sl,
                    level: ResolutionLevel::Minute,
                    exit_fallback: false,
                    random_resolved: false,
                });
            }
            Barrier::Ambiguous => {
                let trades = fetch_trades(c.open_time_ms, c.close_time_ms);
                if let Some(r) = resolve_with_trades(&trades, is_long, tp, sl, c.open_time_ms) {
                    return Some(r);
                }
                return Some(Resolution {
                    is_tp: false,
                    time_ms: c.close_time_ms,
                    price: sl,
                    level: ResolutionLevel::Minute,
                    exit_fallback: true,
                    random_resolved: false,
                });
            }
        }
    }
    None
}

fn resolve_candles_minute_approx(
    candles: &[MiniCandle],
    is_long: bool,
    tp: f64,
    sl: f64,
    rng: &mut u64,
) -> Option<Resolution> {
    for c in candles {
        match barrier_outcome(c, is_long, tp, sl) {
            Barrier::Open => continue,
            Barrier::Tp => {
                return Some(Resolution {
                    is_tp: true,
                    time_ms: c.close_time_ms,
                    price: tp,
                    level: ResolutionLevel::Minute,
                    exit_fallback: false,
                    random_resolved: false,
                });
            }
            Barrier::Sl => {
                return Some(Resolution {
                    is_tp: false,
                    time_ms: c.close_time_ms,
                    price: sl,
                    level: ResolutionLevel::Minute,
                    exit_fallback: false,
                    random_resolved: false,
                });
            }
            Barrier::Ambiguous => {
                let is_tp = xorshift64(rng) % 2 == 0;
                return Some(Resolution {
                    is_tp,
                    time_ms: c.close_time_ms,
                    price: if is_tp { tp } else { sl },
                    level: ResolutionLevel::Minute,
                    exit_fallback: false,
                    random_resolved: true,
                });
            }
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Hour-level resolution
// ---------------------------------------------------------------------------

/// Generic hour-interval resolver. Both the exact and approximate modes share
/// the same TP/SL/Open dispatch; they differ only in how the `Ambiguous` arm
/// breaks the tie (trade-level scan vs RNG). The caller supplies that
/// resolver via `resolve_minute`.
fn resolve_hour_interval_generic<F>(
    candles: &[MiniCandle],
    is_long: bool,
    tp: f64,
    sl: f64,
    close_ms: i64,
    resolve_minute: F,
) -> Option<Resolution>
where
    F: FnOnce(&[MiniCandle]) -> Option<Resolution>,
{
    match barrier_outcome_multi(candles, is_long, tp, sl) {
        Barrier::Open => None,
        Barrier::Tp => Some(Resolution {
            is_tp: true,
            time_ms: close_ms,
            price: tp,
            level: ResolutionLevel::Hour,
            exit_fallback: false,
            random_resolved: false,
        }),
        Barrier::Sl => Some(Resolution {
            is_tp: false,
            time_ms: close_ms,
            price: sl,
            level: ResolutionLevel::Hour,
            exit_fallback: false,
            random_resolved: false,
        }),
        Barrier::Ambiguous => {
            if let Some(r) = resolve_minute(candles) {
                return Some(r);
            }
            Some(Resolution {
                is_tp: false,
                time_ms: close_ms,
                price: sl,
                level: ResolutionLevel::Hour,
                exit_fallback: true,
                random_resolved: false,
            })
        }
    }
}

fn resolve_hour_interval(
    candles: &[MiniCandle],
    is_long: bool,
    tp: f64,
    sl: f64,
    close_ms: i64,
    fetch_trades: &mut dyn FnMut(i64, i64) -> Vec<MiniTrade>,
) -> Option<Resolution> {
    resolve_hour_interval_generic(candles, is_long, tp, sl, close_ms, |c| {
        resolve_candles_minute(c, is_long, tp, sl, fetch_trades)
    })
}

fn resolve_hour_interval_approx(
    candles: &[MiniCandle],
    is_long: bool,
    tp: f64,
    sl: f64,
    close_ms: i64,
    rng: &mut u64,
) -> Option<Resolution> {
    resolve_hour_interval_generic(candles, is_long, tp, sl, close_ms, |c| {
        resolve_candles_minute_approx(c, is_long, tp, sl, rng)
    })
}

// ---------------------------------------------------------------------------
// Partial hour resolution
// ---------------------------------------------------------------------------

fn resolve_partial(
    start: i64,
    end: i64,
    is_long: bool,
    tp: f64,
    sl: f64,
    fetch_min: &mut dyn FnMut(i64, i64) -> Vec<MiniCandle>,
    fetch_trades: &mut dyn FnMut(i64, i64) -> Vec<MiniTrade>,
) -> Option<Resolution> {
    if start >= end {
        return None;
    }
    let full_min_end = floor_minute(end);
    if start < full_min_end {
        let mins = fetch_min(start, full_min_end);
        if let Some(r) = resolve_candles_minute(&mins, is_long, tp, sl, fetch_trades) {
            return Some(r);
        }
    }
    if full_min_end < end {
        let trades = fetch_trades(full_min_end, end);
        return resolve_with_trades(&trades, is_long, tp, sl, full_min_end);
    }
    None
}

// ---------------------------------------------------------------------------
// Main exit resolution — exact mode
// ---------------------------------------------------------------------------

/// Resolve the exit for a trade using hierarchical resolution.
///
/// Data is provided via closures:
/// - `fetch_min(start_ms, end_ms)` → minute candles for the range
/// - `fetch_trades(start_ms, end_ms)` → aggregate trades for the range
///
/// Returns `None` if no TP/SL hit within the candle range.
pub fn resolve_exit(
    hour_candles: &[MiniCandle],
    is_long: bool,
    tp_price: f64,
    sl_price: f64,
    entry_time_ms: i64,
    end_time_ms: Option<i64>,
    fetch_min: &mut dyn FnMut(i64, i64) -> Vec<MiniCandle>,
    fetch_trades: &mut dyn FnMut(i64, i64) -> Vec<MiniTrade>,
) -> Option<ExitResolution> {
    let first_hour_end = floor_hour(entry_time_ms) + 3_600_000;
    let entry_min_end = floor_minute(entry_time_ms) + 60_000;

    // Phase 1: Entry minute — exact trades
    let entry_trades = fetch_trades(entry_time_ms, entry_min_end);
    if let Some(r) = resolve_with_trades(&entry_trades, is_long, tp_price, sl_price, entry_time_ms)
    {
        return Some(r.to_exit_resolution());
    }

    // Phase 2: Remaining minutes in first hour
    let first_mins = fetch_min(entry_min_end, first_hour_end);
    if let Some(r) = resolve_hour_interval(
        &first_mins,
        is_long,
        tp_price,
        sl_price,
        first_hour_end,
        fetch_trades,
    ) {
        return Some(r.to_exit_resolution());
    }

    if matches!(end_time_ms, Some(e) if e <= first_hour_end) {
        return None;
    }

    // Phase 3: Subsequent full hours
    let final_hour = end_time_ms.map(floor_hour);
    for c in hour_candles {
        if c.open_time_ms < first_hour_end {
            continue;
        }
        if matches!(final_hour, Some(fh) if c.open_time_ms >= fh) {
            break;
        }
        match barrier_outcome(c, is_long, tp_price, sl_price) {
            Barrier::Open => continue,
            Barrier::Tp => {
                return Some(
                    Resolution {
                        is_tp: true,
                        time_ms: c.close_time_ms,
                        price: tp_price,
                        level: ResolutionLevel::Hour,
                        exit_fallback: false,
                        random_resolved: false,
                    }
                    .to_exit_resolution(),
                );
            }
            Barrier::Sl => {
                return Some(
                    Resolution {
                        is_tp: false,
                        time_ms: c.close_time_ms,
                        price: sl_price,
                        level: ResolutionLevel::Hour,
                        exit_fallback: false,
                        random_resolved: false,
                    }
                    .to_exit_resolution(),
                );
            }
            Barrier::Ambiguous => {
                let mins = fetch_min(c.open_time_ms, c.close_time_ms);
                if let Some(r) =
                    resolve_candles_minute(&mins, is_long, tp_price, sl_price, fetch_trades)
                {
                    return Some(r.to_exit_resolution());
                }
                // Minute resolution found no clear hit — fall back to SL,
                // matching resolve_hour_interval() behavior.
                return Some(
                    Resolution {
                        is_tp: false,
                        time_ms: c.close_time_ms,
                        price: sl_price,
                        level: ResolutionLevel::Hour,
                        exit_fallback: true,
                        random_resolved: false,
                    }
                    .to_exit_resolution(),
                );
            }
        }
    }

    // Phase 4: Trailing partial hour
    if let (Some(end), Some(fh)) = (end_time_ms, final_hour) {
        if end > fh {
            if let Some(r) = resolve_partial(
                fh,
                end,
                is_long,
                tp_price,
                sl_price,
                fetch_min,
                fetch_trades,
            ) {
                return Some(r.to_exit_resolution());
            }
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Main exit resolution — approximate mode
// ---------------------------------------------------------------------------

/// Resolve the exit using approximate mode (no trade fetching, RNG for
/// ambiguous minutes).
///
/// `seed` is the initial state for the XORShift64 PRNG.
pub fn resolve_exit_approximate(
    hour_candles: &[MiniCandle],
    is_long: bool,
    tp_price: f64,
    sl_price: f64,
    entry_time_ms: i64,
    end_time_ms: Option<i64>,
    fetch_min: &mut dyn FnMut(i64, i64) -> Vec<MiniCandle>,
    seed: u64,
) -> Option<ExitResolution> {
    let mut rng = seed;
    let first_hour_end = floor_hour(entry_time_ms) + 3_600_000;
    let entry_min_start = floor_minute(entry_time_ms);

    // Phase 1: First hour (includes entry minute)
    let first_mins = fetch_min(entry_min_start, first_hour_end);
    if let Some(r) = resolve_hour_interval_approx(
        &first_mins,
        is_long,
        tp_price,
        sl_price,
        first_hour_end,
        &mut rng,
    ) {
        return Some(r.to_exit_resolution());
    }

    if matches!(end_time_ms, Some(e) if e <= first_hour_end) {
        return None;
    }

    // Phase 2: Subsequent full hours
    let final_hour = end_time_ms.map(floor_hour);
    for c in hour_candles {
        if c.open_time_ms < first_hour_end {
            continue;
        }
        if matches!(final_hour, Some(fh) if c.open_time_ms >= fh) {
            break;
        }
        match barrier_outcome(c, is_long, tp_price, sl_price) {
            Barrier::Open => continue,
            Barrier::Tp => {
                return Some(
                    Resolution {
                        is_tp: true,
                        time_ms: c.close_time_ms,
                        price: tp_price,
                        level: ResolutionLevel::Hour,
                        exit_fallback: false,
                        random_resolved: false,
                    }
                    .to_exit_resolution(),
                );
            }
            Barrier::Sl => {
                return Some(
                    Resolution {
                        is_tp: false,
                        time_ms: c.close_time_ms,
                        price: sl_price,
                        level: ResolutionLevel::Hour,
                        exit_fallback: false,
                        random_resolved: false,
                    }
                    .to_exit_resolution(),
                );
            }
            Barrier::Ambiguous => {
                let mins = fetch_min(c.open_time_ms, c.close_time_ms);
                if let Some(r) =
                    resolve_candles_minute_approx(&mins, is_long, tp_price, sl_price, &mut rng)
                {
                    return Some(r.to_exit_resolution());
                }
                // Minute resolution found no clear hit — fall back to SL,
                // matching resolve_hour_interval_approx() behavior.
                return Some(
                    Resolution {
                        is_tp: false,
                        time_ms: c.close_time_ms,
                        price: sl_price,
                        level: ResolutionLevel::Hour,
                        exit_fallback: true,
                        random_resolved: false,
                    }
                    .to_exit_resolution(),
                );
            }
        }
    }

    // Phase 3: Trailing partial hour
    if let (Some(end), Some(fh)) = (end_time_ms, final_hour) {
        if end > fh {
            let full_min_end = floor_minute(end);
            if fh < full_min_end {
                let mins = fetch_min(fh, full_min_end);
                if let Some(r) =
                    resolve_candles_minute_approx(&mins, is_long, tp_price, sl_price, &mut rng)
                {
                    return Some(r.to_exit_resolution());
                }
            }
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Conversion helpers for callers
// ---------------------------------------------------------------------------

impl MiniCandle {
    pub fn from_candle(c: &claude_trader_models::Candle) -> Self {
        Self {
            open_time_ms: crate::dt_to_ms(c.open_time),
            close_time_ms: crate::dt_to_ms(c.close_time),
            high: c.high,
            low: c.low,
        }
    }
}

impl MiniTrade {
    pub fn from_agg_trade(t: &claude_trader_models::AggTrade) -> Self {
        Self {
            timestamp_ms: crate::dt_to_ms(t.timestamp),
            price: t.price,
        }
    }
}
