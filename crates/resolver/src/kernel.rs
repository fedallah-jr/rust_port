//! Full backtest kernel — backtest_signal() and backtest_signals().
//!
//! Wires together entry resolution, TP/SL computation, exit resolution,
//! timeout resolution, and PnL computation.
//!
//! Mirrors Python `backtester/engine.py::backtest_signal()`.

use std::cell::RefCell;
use std::sync::Arc;

use chrono::{DateTime, Duration, Utc};
use claude_trader_models::{
    AggTrade, Candle, ExitReason, PositionType, ResolutionLevel, Signal, TradeResult,
};

use crate::exit::{MiniCandle, MiniTrade};
use crate::stats::compute_stats;
use crate::{compute_pnl, compute_tp_sl_prices_from_signal, dt_to_ms, ms_to_dt};

// ---------------------------------------------------------------------------
// Data provider trait
// ---------------------------------------------------------------------------

/// Abstraction over market data fetching. Implementations can be backed by
/// disk cache, in-memory prepared context, or live API.
pub trait MarketDataProvider {
    fn fetch_analysis_candles(
        &mut self,
        symbol: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Vec<Candle>;

    fn fetch_minute_candles(
        &mut self,
        symbol: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Vec<Candle>;

    fn fetch_agg_trades(
        &mut self,
        symbol: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Vec<AggTrade>;
}

// ---------------------------------------------------------------------------
// backtest_signal
// ---------------------------------------------------------------------------

/// Backtest a single signal, resolving entry, exit, and computing PnL.
///
/// Mirrors Python `backtester/engine.py::backtest_signal()`.
pub fn backtest_signal(
    signal: &Arc<Signal>,
    provider: &mut dyn MarketDataProvider,
    approximate: bool,
    seed: Option<u64>,
    default_entry_delay: i64,
) -> TradeResult {
    // Step 1: Resolve entry
    let entry = if approximate {
        resolve_entry_approximate(signal, provider)
    } else if signal.entry_price.is_none() {
        resolve_entry_market(signal, provider, default_entry_delay)
    } else {
        resolve_entry_limit(signal, provider)
    };

    let (entry_price, entry_time, entry_fallback) = match entry {
        Some(e) => e,
        None => return unfilled_result(signal),
    };

    // Step 2: Compute TP/SL prices
    let (tp_price, sl_price) = match compute_tp_sl_prices_from_signal(entry_price, signal) {
        Ok(v) => v,
        Err(_) => return unfilled_result(signal),
    };

    // Step 3: Fetch hour candles for resolution window
    let resolution_end = entry_time + Duration::hours(signal.max_holding_hours);
    let hour_candles = provider.fetch_analysis_candles(&signal.ticker, entry_time, resolution_end);

    // Step 4: Exit resolution
    let is_long = signal.position_type.is_long();
    let entry_ms = dt_to_ms(entry_time);
    let end_ms = Some(dt_to_ms(resolution_end));

    let mini_hours: Vec<MiniCandle> = hour_candles.iter().map(MiniCandle::from_candle).collect();

    // Use RefCell to allow two closures to borrow provider
    let provider_cell = RefCell::new(provider);
    let ticker = &signal.ticker;

    let exit = if approximate {
        crate::exit::resolve_exit_approximate(
            &mini_hours,
            is_long,
            tp_price,
            sl_price,
            entry_ms,
            end_ms,
            &mut |start, end| {
                let candles = provider_cell.borrow_mut().fetch_minute_candles(
                    ticker,
                    ms_to_dt(start),
                    ms_to_dt(end),
                );
                candles.iter().map(MiniCandle::from_candle).collect()
            },
            seed.unwrap_or_else(|| {
                let ms = dt_to_ms(entry_time) as u64;
                ms | 1 // ensure non-zero so xorshift64 is not a fixed point
            }),
        )
    } else {
        crate::exit::resolve_exit(
            &mini_hours,
            is_long,
            tp_price,
            sl_price,
            entry_ms,
            end_ms,
            &mut |start, end| {
                let candles = provider_cell.borrow_mut().fetch_minute_candles(
                    ticker,
                    ms_to_dt(start),
                    ms_to_dt(end),
                );
                candles.iter().map(MiniCandle::from_candle).collect()
            },
            &mut |start, end| {
                let trades = provider_cell.borrow_mut().fetch_agg_trades(
                    ticker,
                    ms_to_dt(start),
                    ms_to_dt(end),
                );
                trades.iter().map(MiniTrade::from_agg_trade).collect()
            },
        )
    };

    // Recover provider from RefCell
    let provider = provider_cell.into_inner();

    let (exit_price, exit_time, exit_reason, resolution_level, exit_fallback, random_resolved) =
        match exit {
            Some(res) => (
                res.exit_price,
                res.exit_time,
                res.reason,
                res.resolution_level,
                res.exit_fallback,
                res.random_resolved,
            ),
            None => {
                // Timeout: no TP/SL hit within max_holding_hours
                let (t_price, t_time, t_level, t_fallback) = resolve_timeout_exit(
                    signal,
                    provider,
                    resolution_end,
                    entry_price,
                    approximate,
                );
                (
                    t_price,
                    t_time,
                    ExitReason::Timeout,
                    t_level,
                    t_fallback,
                    false,
                )
            }
        };

    // Step 5: Compute PnL
    let (net_pnl, gross_pnl, fee_drag) = compute_pnl(
        entry_price,
        exit_price,
        is_long,
        signal.leverage,
        signal.taker_fee_rate,
    );

    TradeResult {
        signal: Arc::clone(signal),
        entry_price,
        entry_time,
        exit_price,
        exit_time,
        exit_reason,
        resolution_level,
        tp_price,
        sl_price,
        pnl_pct: net_pnl,
        gross_pnl_pct: gross_pnl,
        fee_drag_pct: fee_drag,
        entry_fallback,
        exit_fallback,
        random_resolved,
    }
}

/// Backtest multiple signals sequentially, returning aggregated results.
///
/// For parallel execution, callers should partition signals and call this
/// per-partition, then merge. The Rust evaluator (M4) will handle threading.
pub fn backtest_signals(
    signals: &[Arc<Signal>],
    provider: &mut dyn MarketDataProvider,
    approximate: bool,
    seed: Option<u64>,
    default_entry_delay: i64,
) -> claude_trader_models::BacktestResult {
    let mut trades = Vec::with_capacity(signals.len());

    // Deterministic per-signal seeds from parent seed
    let mut rng_state = seed.unwrap_or(0);

    for signal in signals {
        let sig_seed = if seed.is_some() {
            rng_state = xorshift64(rng_state);
            Some(rng_state)
        } else {
            None
        };

        let trade = backtest_signal(signal, provider, approximate, sig_seed, default_entry_delay);
        trades.push(trade);
    }

    // Sort by entry_time
    trades.sort_by_key(|t| t.entry_time);

    compute_stats(&trades)
}

// ---------------------------------------------------------------------------
// Entry resolution helpers
// ---------------------------------------------------------------------------

fn resolve_entry_market(
    signal: &Signal,
    provider: &mut dyn MarketDataProvider,
    default_entry_delay: i64,
) -> Option<(f64, DateTime<Utc>, bool)> {
    let delay = signal.entry_delay_seconds.unwrap_or(default_entry_delay);
    let start = signal.signal_date + Duration::seconds(delay);
    let end = start + Duration::seconds(10);

    let trades = provider.fetch_agg_trades(&signal.ticker, signal.signal_date, end);

    // First trade at/after delay
    for t in &trades {
        if t.timestamp >= start {
            return Some((t.price, t.timestamp, false));
        }
    }

    // Fallback: minute candle covering the entry time. Use the candle's
    // open (price at minute start) as a conservative stand-in when no
    // agg-trades are available in the fill window.
    let start_ms = dt_to_ms(start);
    let min_start = ms_to_dt(start_ms - start_ms.rem_euclid(60_000));
    let min_end = min_start + Duration::minutes(1);
    let candles = provider.fetch_minute_candles(&signal.ticker, min_start, min_end);
    if let Some(c) = candles.first() {
        return Some((c.open, c.open_time, true));
    }

    None
}

fn resolve_entry_limit(
    signal: &Signal,
    provider: &mut dyn MarketDataProvider,
) -> Option<(f64, DateTime<Utc>, bool)> {
    let limit_price = signal.entry_price?;
    let deadline = signal.signal_date + Duration::seconds(signal.fill_timeout_seconds);
    let trades = provider.fetch_agg_trades(&signal.ticker, signal.signal_date, deadline);

    for t in &trades {
        if t.timestamp < signal.signal_date {
            continue;
        }
        let filled = match signal.position_type {
            PositionType::Long => t.price <= limit_price,
            PositionType::Short => t.price >= limit_price,
        };
        if filled {
            return Some((limit_price, t.timestamp, false));
        }
    }

    None
}

fn resolve_entry_approximate(
    signal: &Signal,
    provider: &mut dyn MarketDataProvider,
) -> Option<(f64, DateTime<Utc>, bool)> {
    // Approximate-mode entry uses 1-minute candles uniformly, regardless of
    // the strategy's native timeframe. For a signal fired inside minute `m`
    // (i.e. signal_date in `[m, m+1min)`), enter at the OPEN of the next 1m
    // candle whose open_time is `m + 1min`. This models "strategy saw the
    // minute-m close, earliest realistic fill is the next minute open"
    // without look-ahead or stale look-behind, and sidesteps the off-by-one
    // that Binance's close_time = open_time + interval - 1ms convention
    // creates under naive truncation. `entry_delay_seconds` is intentionally
    // ignored here — the minute-granularity approximation already subsumes
    // sub-minute delays; exact/market mode still honors it.
    let sig_ms = dt_to_ms(signal.signal_date);
    let minute_start_ms = sig_ms - sig_ms.rem_euclid(60_000);
    let next_open_ms = minute_start_ms + 60_000;
    let next_end_ms = next_open_ms + 60_000;

    let candles = provider.fetch_minute_candles(
        &signal.ticker,
        ms_to_dt(next_open_ms),
        ms_to_dt(next_end_ms),
    );

    candles
        .iter()
        .find(|c| dt_to_ms(c.open_time) == next_open_ms)
        .map(|c| (c.open, c.open_time, false))
}

// ---------------------------------------------------------------------------
// Timeout resolution
// ---------------------------------------------------------------------------

/// Resolve exit at timeout (no TP/SL hit within max_holding_hours).
/// Returns (exit_price, exit_time, resolution_level, exit_fallback).
fn resolve_timeout_exit(
    signal: &Signal,
    provider: &mut dyn MarketDataProvider,
    timeout_time: DateTime<Utc>,
    entry_price: f64,
    approximate: bool,
) -> (f64, DateTime<Utc>, ResolutionLevel, bool) {
    // Tier 1: Agg trades (skipped in approximate mode)
    if !approximate {
        let end = timeout_time + Duration::minutes(5);
        let trades = provider.fetch_agg_trades(&signal.ticker, timeout_time, end);
        for t in &trades {
            if t.timestamp >= timeout_time {
                return (t.price, t.timestamp, ResolutionLevel::Trade, false);
            }
        }
    }

    // Tier 2: Minute candles
    let min_start_ms = dt_to_ms(timeout_time) - dt_to_ms(timeout_time).rem_euclid(60_000);
    let min_start = ms_to_dt(min_start_ms);
    let min_end = min_start + Duration::minutes(2);
    let candles = provider.fetch_minute_candles(&signal.ticker, min_start, min_end);
    for c in &candles {
        if c.close_time >= timeout_time {
            return (c.close, c.close_time, ResolutionLevel::Minute, !approximate);
        }
    }

    // Tier 3: Hour candles
    let hour_start_ms = dt_to_ms(timeout_time) - dt_to_ms(timeout_time).rem_euclid(3_600_000);
    let hour_start = ms_to_dt(hour_start_ms);
    let hour_end = hour_start + Duration::hours(2);
    let candles = provider.fetch_analysis_candles(&signal.ticker, hour_start, hour_end);
    for c in &candles {
        if c.close_time >= timeout_time {
            return (c.close, c.close_time, ResolutionLevel::Hour, !approximate);
        }
    }

    // Final fallback: use entry price
    let level = if approximate {
        ResolutionLevel::Hour
    } else {
        ResolutionLevel::Trade
    };
    (entry_price, timeout_time, level, !approximate)
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn unfilled_result(signal: &Arc<Signal>) -> TradeResult {
    TradeResult {
        signal: Arc::clone(signal),
        entry_price: 0.0,
        entry_time: signal.signal_date,
        exit_price: 0.0,
        exit_time: signal.signal_date,
        exit_reason: ExitReason::Unfilled,
        resolution_level: ResolutionLevel::Hour,
        tp_price: 0.0,
        sl_price: 0.0,
        pnl_pct: 0.0,
        gross_pnl_pct: 0.0,
        fee_drag_pct: 0.0,
        entry_fallback: false,
        exit_fallback: false,
        random_resolved: false,
    }
}

#[inline]
fn xorshift64(mut state: u64) -> u64 {
    state ^= state << 13;
    state ^= state >> 7;
    state ^= state << 17;
    state
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use claude_trader_models::MarketType;
    use std::collections::HashMap;

    #[derive(Default)]
    struct TestProvider {
        minute_candles: Vec<Candle>,
    }

    impl MarketDataProvider for TestProvider {
        fn fetch_analysis_candles(
            &mut self,
            _symbol: &str,
            _start: DateTime<Utc>,
            _end: DateTime<Utc>,
        ) -> Vec<Candle> {
            Vec::new()
        }

        fn fetch_minute_candles(
            &mut self,
            _symbol: &str,
            start: DateTime<Utc>,
            end: DateTime<Utc>,
        ) -> Vec<Candle> {
            self.minute_candles
                .iter()
                .copied()
                .filter(|c| c.open_time >= start && c.close_time <= end)
                .collect()
        }

        fn fetch_agg_trades(
            &mut self,
            _symbol: &str,
            _start: DateTime<Utc>,
            _end: DateTime<Utc>,
        ) -> Vec<AggTrade> {
            Vec::new()
        }
    }

    fn signal_at(ts: DateTime<Utc>) -> Signal {
        Signal {
            signal_date: ts,
            position_type: PositionType::Long,
            ticker: "BTCUSDT".to_string(),
            pattern: "test".to_string(),
            tp_pct: Some(3.0),
            sl_pct: Some(1.5),
            tp_price: None,
            sl_price: None,
            leverage: 1.0,
            market_type: MarketType::Futures,
            taker_fee_rate: 0.0005,
            entry_price: None,
            fill_timeout_seconds: 3600,
            entry_delay_seconds: Some(15),
            max_holding_hours: 72,
            size_multiplier: 1.0,
            metadata: HashMap::new(),
        }
    }

    fn minute_candle(open_time: DateTime<Utc>, close_time: DateTime<Utc>, close: f64) -> Candle {
        Candle {
            open_time,
            close_time,
            open: close,
            high: close,
            low: close,
            close,
            volume: 0.0,
            taker_buy_volume: 0.0,
        }
    }

    #[test]
    fn test_unfilled_resolution_level_matches_python() {
        let ts = Utc.with_ymd_and_hms(2024, 1, 1, 12, 0, 30).unwrap();
        let signal = Arc::new(signal_at(ts));
        let mut provider = TestProvider::default();

        let trade = backtest_signal(&signal, &mut provider, false, None, 15);

        assert_eq!(trade.exit_reason, ExitReason::Unfilled);
        assert_eq!(trade.resolution_level, ResolutionLevel::Hour);
    }

    #[test]
    fn test_market_fallback_does_not_use_second_minute_candle() {
        let ts = Utc.with_ymd_and_hms(2024, 1, 1, 12, 0, 30).unwrap();
        let signal = Arc::new(signal_at(ts));
        let mut provider = TestProvider {
            minute_candles: vec![minute_candle(
                Utc.with_ymd_and_hms(2024, 1, 1, 12, 1, 0).unwrap(),
                Utc.with_ymd_and_hms(2024, 1, 1, 12, 2, 0).unwrap(),
                101.0,
            )],
        };

        let trade = backtest_signal(&signal, &mut provider, false, None, 15);

        assert_eq!(trade.exit_reason, ExitReason::Unfilled);
    }

    #[test]
    fn test_approximate_entry_uses_next_minute_open() {
        // Signal mid-minute at 12:02:37. Next-minute candle opens at 12:03:00;
        // approximate entry is priced at that candle's open and stamped with
        // its open_time.
        let ts = Utc.with_ymd_and_hms(2024, 1, 1, 12, 2, 37).unwrap();
        let signal = Arc::new(signal_at(ts));

        let next_open = Utc.with_ymd_and_hms(2024, 1, 1, 12, 3, 0).unwrap();
        let next_close = Utc.with_ymd_and_hms(2024, 1, 1, 12, 4, 0).unwrap();
        let mut next_candle = minute_candle(next_open, next_close, 100.0);
        next_candle.open = 123.45;

        let mut provider = TestProvider {
            minute_candles: vec![next_candle],
        };

        let trade = backtest_signal(&signal, &mut provider, true, None, 15);

        assert_eq!(trade.entry_price, 123.45);
        assert_eq!(trade.entry_time, next_open);
        assert!(!trade.entry_fallback);
    }

    #[test]
    fn test_approximate_entry_signal_on_minute_boundary() {
        // Signal at 12:02:00.000 lies inside minute 12:02 (half-open
        // [12:02, 12:03)), so the next-minute candle open is 12:03:00.
        let ts = Utc.with_ymd_and_hms(2024, 1, 1, 12, 2, 0).unwrap();
        let signal = Arc::new(signal_at(ts));

        let next_open = Utc.with_ymd_and_hms(2024, 1, 1, 12, 3, 0).unwrap();
        let next_close = Utc.with_ymd_and_hms(2024, 1, 1, 12, 4, 0).unwrap();
        let mut next_candle = minute_candle(next_open, next_close, 200.0);
        next_candle.open = 250.0;

        let mut provider = TestProvider {
            minute_candles: vec![next_candle],
        };

        let trade = backtest_signal(&signal, &mut provider, true, None, 15);

        assert_eq!(trade.entry_price, 250.0);
        assert_eq!(trade.entry_time, next_open);
    }

    #[test]
    fn test_approximate_entry_missing_next_minute_is_unfilled() {
        // Provider holds only the signal's own minute candle (12:02-12:03);
        // the next-minute candle is absent, so the trade must be unfilled
        // rather than silently approximating further.
        let ts = Utc.with_ymd_and_hms(2024, 1, 1, 12, 2, 37).unwrap();
        let signal = Arc::new(signal_at(ts));

        let own_open = Utc.with_ymd_and_hms(2024, 1, 1, 12, 2, 0).unwrap();
        let own_close = Utc.with_ymd_and_hms(2024, 1, 1, 12, 3, 0).unwrap();
        let own_candle = minute_candle(own_open, own_close, 100.0);

        let mut provider = TestProvider {
            minute_candles: vec![own_candle],
        };

        let trade = backtest_signal(&signal, &mut provider, true, None, 15);

        assert_eq!(trade.exit_reason, ExitReason::Unfilled);
    }
}
