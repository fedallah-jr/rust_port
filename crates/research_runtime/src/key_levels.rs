//! Key levels computation — Rust port of `marketdata/key_levels.py`.
//!
//! Computes structural price levels (PDH/PDL, weekly, monthly, quarterly,
//! yearly, sessions, Monday range) from multi-timeframe candle data.
//! All values are lookahead-free: derived from completed candles only.

use chrono::{DateTime, Datelike, Timelike, Utc};
use claude_trader_data::{BinanceClient, CandleStore};
use claude_trader_models::{Candle, KeyLevels};

// Session boundaries (UTC hours) — matches Python constants.
const ASIA_START_HOUR: u32 = 0;
const ASIA_END_HOUR: u32 = 8;
const LONDON_START_HOUR: u32 = 8;
const LONDON_END_HOUR: u32 = 16;
const NY_START_HOUR: u32 = 13;
const NY_END_HOUR: u32 = 22;

// ---------------------------------------------------------------------------
// Internal structures
// ---------------------------------------------------------------------------

struct AggPeriod {
    start: DateTime<Utc>,
    open: f64,
    high: f64,
    low: f64,
}

struct YearlyRunning {
    year_open: f64,
    candle_close_times: Vec<DateTime<Utc>>,
    cum_highs: Vec<f64>,
    cum_lows: Vec<f64>,
}

struct CompletedSession {
    end_time: DateTime<Utc>,
    open: f64,
    high: f64,
    low: f64,
}

struct CompletedMonday {
    end_time: DateTime<Utc>,
    high: f64,
    low: f64,
    mid: f64,
}

// ---------------------------------------------------------------------------
// Helpers — standard timeframes (4H / D / W / M)
// ---------------------------------------------------------------------------

/// Return (current_open, prev_high, prev_low, eq) via binary search on open_time.
/// Retained as a parity reference for the cursor-based implementation.
#[cfg(test)]
fn open_prev_levels(
    candles: &[Candle],
    t: DateTime<Utc>,
) -> (Option<f64>, Option<f64>, Option<f64>, Option<f64>) {
    if candles.is_empty() {
        return (None, None, None, None);
    }
    let idx = bisect_right_by(candles, t, |c| c.open_time);
    if idx == 0 {
        return (None, None, None, None);
    }
    let current = &candles[idx - 1];
    if idx < 2 {
        return (Some(current.open), None, None, None);
    }
    let prev = &candles[idx - 2];
    let eq = (prev.high + prev.low) / 2.0;
    (
        Some(current.open),
        Some(prev.high),
        Some(prev.low),
        Some(eq),
    )
}

// ---------------------------------------------------------------------------
// Helpers — quarterly (aggregated from daily candles)
// ---------------------------------------------------------------------------

fn quarter_of(dt: DateTime<Utc>) -> (i32, u32) {
    (dt.year(), (dt.month() - 1) / 3 + 1)
}

fn aggregate_quarters(daily_candles: &[Candle]) -> Vec<AggPeriod> {
    if daily_candles.is_empty() {
        return Vec::new();
    }
    let mut result = Vec::new();
    let mut cur_q = quarter_of(daily_candles[0].open_time);
    let mut start = daily_candles[0].open_time;
    let mut q_open = daily_candles[0].open;
    let mut q_high = daily_candles[0].high;
    let mut q_low = daily_candles[0].low;

    for c in &daily_candles[1..] {
        let q = quarter_of(c.open_time);
        if q != cur_q {
            result.push(AggPeriod {
                start,
                open: q_open,
                high: q_high,
                low: q_low,
            });
            cur_q = q;
            start = c.open_time;
            q_open = c.open;
            q_high = c.high;
            q_low = c.low;
        } else {
            q_high = q_high.max(c.high);
            q_low = q_low.min(c.low);
        }
    }
    result.push(AggPeriod {
        start,
        open: q_open,
        high: q_high,
        low: q_low,
    });
    result
}

/// Parity reference for `quarterly_levels_cursor`.
#[cfg(test)]
fn quarterly_levels(
    quarters: &[AggPeriod],
    t: DateTime<Utc>,
) -> (Option<f64>, Option<f64>, Option<f64>, Option<f64>) {
    if quarters.is_empty() {
        return (None, None, None, None);
    }
    let idx = bisect_right_by(quarters, t, |q| q.start);
    if idx == 0 {
        return (None, None, None, None);
    }
    if idx < 2 {
        return (Some(quarters[0].open), None, None, None);
    }
    let prev = &quarters[idx - 2];
    let eq = (prev.high + prev.low) / 2.0;
    (
        Some(quarters[idx - 1].open),
        Some(prev.high),
        Some(prev.low),
        Some(eq),
    )
}

// ---------------------------------------------------------------------------
// Helpers — yearly (running cumulative high/low)
// ---------------------------------------------------------------------------

fn build_yearly_running(daily_candles: &[Candle]) -> std::collections::HashMap<i32, YearlyRunning> {
    let mut by_year: std::collections::HashMap<i32, Vec<&Candle>> =
        std::collections::HashMap::new();
    for c in daily_candles {
        by_year.entry(c.open_time.year()).or_default().push(c);
    }
    let mut result = std::collections::HashMap::new();
    for (year, candles) in by_year {
        let mut cum_highs = Vec::with_capacity(candles.len());
        let mut cum_lows = Vec::with_capacity(candles.len());
        let mut running_high = f64::NEG_INFINITY;
        let mut running_low = f64::INFINITY;
        let mut close_times = Vec::with_capacity(candles.len());
        for c in &candles {
            running_high = running_high.max(c.high);
            running_low = running_low.min(c.low);
            cum_highs.push(running_high);
            cum_lows.push(running_low);
            close_times.push(c.close_time);
        }
        result.insert(
            year,
            YearlyRunning {
                year_open: candles[0].open,
                candle_close_times: close_times,
                cum_highs,
                cum_lows,
            },
        );
    }
    result
}

/// Parity reference for `yearly_levels_cursor`.
#[cfg(test)]
fn yearly_levels(
    yearly_data: &std::collections::HashMap<i32, YearlyRunning>,
    t: DateTime<Utc>,
) -> (Option<f64>, Option<f64>, Option<f64>, Option<f64>) {
    let data = match yearly_data.get(&t.year()) {
        Some(d) => d,
        None => return (None, None, None, None),
    };
    let idx = bisect_right_dt(&data.candle_close_times, t);
    if idx == 0 {
        return (Some(data.year_open), None, None, None);
    }
    let yh = data.cum_highs[idx - 1];
    let yl = data.cum_lows[idx - 1];
    (
        Some(data.year_open),
        Some(yh),
        Some(yl),
        Some((yh + yl) / 2.0),
    )
}

// ---------------------------------------------------------------------------
// Helpers — sessions (Asia / London / New York)
// ---------------------------------------------------------------------------

fn build_completed_sessions(
    hourly_candles: &[Candle],
    start_hour: u32,
    end_hour: u32,
) -> Vec<CompletedSession> {
    let mut by_date: std::collections::BTreeMap<chrono::NaiveDate, Vec<&Candle>> =
        std::collections::BTreeMap::new();
    for c in hourly_candles {
        let hour = c.open_time.hour();
        if hour >= start_hour && hour < end_hour {
            let d = c.open_time.date_naive();
            by_date.entry(d).or_default().push(c);
        }
    }

    let expected = (end_hour - start_hour) as usize;
    let mut result = Vec::new();
    for (_d, mut candles) in by_date {
        if candles.len() < expected {
            continue;
        }
        candles.sort_by_key(|c| c.open_time);
        let open = candles[0].open;
        let high = candles
            .iter()
            .map(|c| c.high)
            .fold(f64::NEG_INFINITY, f64::max);
        let low = candles.iter().map(|c| c.low).fold(f64::INFINITY, f64::min);
        let end_time = candles.last().unwrap().close_time;
        result.push(CompletedSession {
            end_time,
            open,
            high,
            low,
        });
    }
    result
}

// ---------------------------------------------------------------------------
// Helpers — Monday range
// ---------------------------------------------------------------------------

fn build_completed_mondays(hourly_candles: &[Candle]) -> Vec<CompletedMonday> {
    let mut by_monday: std::collections::BTreeMap<chrono::NaiveDate, Vec<&Candle>> =
        std::collections::BTreeMap::new();
    for c in hourly_candles {
        if c.open_time.weekday() == chrono::Weekday::Mon {
            let d = c.open_time.date_naive();
            by_monday.entry(d).or_default().push(c);
        }
    }

    let mut result = Vec::new();
    for (_d, mut candles) in by_monday {
        if candles.len() < 24 {
            continue;
        }
        candles.sort_by_key(|c| c.open_time);
        let high = candles
            .iter()
            .map(|c| c.high)
            .fold(f64::NEG_INFINITY, f64::max);
        let low = candles.iter().map(|c| c.low).fold(f64::INFINITY, f64::min);
        let end_time = candles.last().unwrap().close_time;
        result.push(CompletedMonday {
            end_time,
            high,
            low,
            mid: (high + low) / 2.0,
        });
    }
    result
}

// ---------------------------------------------------------------------------
// Generic "most recent completed" lookup
// ---------------------------------------------------------------------------

/// Parity reference for `latest_by_cursor` on sessions.
#[cfg(test)]
fn latest_session_before<'a>(
    items: &'a [CompletedSession],
    t: DateTime<Utc>,
) -> Option<&'a CompletedSession> {
    if items.is_empty() {
        return None;
    }
    let idx = bisect_right_by(items, t, |s| s.end_time);
    if idx == 0 {
        None
    } else {
        Some(&items[idx - 1])
    }
}

/// Parity reference for `latest_by_cursor` on Monday ranges.
#[cfg(test)]
fn latest_monday_before<'a>(
    items: &'a [CompletedMonday],
    t: DateTime<Utc>,
) -> Option<&'a CompletedMonday> {
    if items.is_empty() {
        return None;
    }
    let idx = bisect_right_by(items, t, |m| m.end_time);
    if idx == 0 {
        None
    } else {
        Some(&items[idx - 1])
    }
}

// ---------------------------------------------------------------------------
// Binary search helpers
// ---------------------------------------------------------------------------

/// bisect_right equivalent: returns the index where `t` would be inserted
/// to keep the list sorted, using `key_fn` to extract the sort key.
/// Retained alongside the cursor path for parity testing.
#[cfg(test)]
fn bisect_right_by<T>(
    items: &[T],
    t: DateTime<Utc>,
    key_fn: impl Fn(&T) -> DateTime<Utc>,
) -> usize {
    items.partition_point(|item| key_fn(item) <= t)
}

#[cfg(test)]
fn bisect_right_dt(times: &[DateTime<Utc>], t: DateTime<Utc>) -> usize {
    times.partition_point(|&time| time <= t)
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Compute structural key levels for each timestamp.
///
/// All candle lists must be sorted by `open_time`.
/// `timestamps` must be sorted ascending.
///
/// Lookahead-free by construction: every value is derived from completed
/// candles/periods whose close_time <= the query timestamp.
///
/// Uses monotonic cursors across the sorted input structures: since every
/// keyed array is sorted and `timestamps` is non-decreasing, each cursor
/// advances only forward, giving an O(N + M) sweep instead of the O(N log M)
/// binary-search-per-timestamp path.
pub fn compute_key_levels_series(
    h4_candles: &[Candle],
    daily_candles: &[Candle],
    weekly_candles: &[Candle],
    monthly_candles: &[Candle],
    hourly_candles: &[Candle],
    timestamps: &[DateTime<Utc>],
) -> Vec<KeyLevels> {
    if timestamps.is_empty() {
        return Vec::new();
    }

    // Pre-compute derived structures once.
    let quarters = aggregate_quarters(daily_candles);
    let yearly = build_yearly_running(daily_candles);
    let mondays = build_completed_mondays(hourly_candles);
    let asia_sessions = build_completed_sessions(hourly_candles, ASIA_START_HOUR, ASIA_END_HOUR);
    let london_sessions =
        build_completed_sessions(hourly_candles, LONDON_START_HOUR, LONDON_END_HOUR);
    let ny_sessions = build_completed_sessions(hourly_candles, NY_START_HOUR, NY_END_HOUR);

    // Cursors — one per sorted structure. Each holds the partition_point for
    // the key-condition-relative-to-t; advanced lazily as `t` grows.
    let mut cur_h4 = 0usize;
    let mut cur_daily = 0usize;
    let mut cur_weekly = 0usize;
    let mut cur_monthly = 0usize;
    let mut cur_quarters = 0usize;
    let mut cur_mondays = 0usize;
    let mut cur_asia = 0usize;
    let mut cur_london = 0usize;
    let mut cur_ny = 0usize;

    // Yearly cursor is per-year; reset when `t` crosses a year boundary.
    let mut cur_year: Option<i32> = None;
    let mut cur_yearly_idx: usize = 0;

    let mut result = Vec::with_capacity(timestamps.len());
    for &t in timestamps {
        advance_by_key(h4_candles, &mut cur_h4, t, |c| c.open_time);
        advance_by_key(daily_candles, &mut cur_daily, t, |c| c.open_time);
        advance_by_key(weekly_candles, &mut cur_weekly, t, |c| c.open_time);
        advance_by_key(monthly_candles, &mut cur_monthly, t, |c| c.open_time);
        advance_by_key(&quarters, &mut cur_quarters, t, |q| q.start);
        advance_by_key(&mondays, &mut cur_mondays, t, |m| m.end_time);
        advance_by_key(&asia_sessions, &mut cur_asia, t, |s| s.end_time);
        advance_by_key(&london_sessions, &mut cur_london, t, |s| s.end_time);
        advance_by_key(&ny_sessions, &mut cur_ny, t, |s| s.end_time);

        let h4 = open_prev_levels_cursor(h4_candles, cur_h4);
        let dl = open_prev_levels_cursor(daily_candles, cur_daily);
        let wk = open_prev_levels_cursor(weekly_candles, cur_weekly);
        let mo = open_prev_levels_cursor(monthly_candles, cur_monthly);
        let qt = quarterly_levels_cursor(&quarters, cur_quarters);

        // Yearly: reset the per-year cursor whenever `t` crosses into a new year.
        let year = t.year();
        if Some(year) != cur_year {
            cur_year = Some(year);
            cur_yearly_idx = 0;
        }
        let yr = yearly_levels_cursor(&yearly, year, &mut cur_yearly_idx, t);

        let mon = latest_by_cursor(&mondays, cur_mondays);
        let asia = latest_by_cursor(&asia_sessions, cur_asia);
        let ldn = latest_by_cursor(&london_sessions, cur_london);
        let ny = latest_by_cursor(&ny_sessions, cur_ny);

        result.push(KeyLevels {
            h4_open: h4.0,
            prev_h4_high: h4.1,
            prev_h4_low: h4.2,
            h4_eq: h4.3,
            daily_open: dl.0,
            pdh: dl.1,
            pdl: dl.2,
            daily_eq: dl.3,
            weekly_open: wk.0,
            prev_week_high: wk.1,
            prev_week_low: wk.2,
            weekly_eq: wk.3,
            monthly_open: mo.0,
            prev_month_high: mo.1,
            prev_month_low: mo.2,
            monthly_eq: mo.3,
            quarterly_open: qt.0,
            prev_quarter_high: qt.1,
            prev_quarter_low: qt.2,
            quarterly_eq: qt.3,
            yearly_open: yr.0,
            yearly_high: yr.1,
            yearly_low: yr.2,
            yearly_eq: yr.3,
            monday_high: mon.map(|m| m.high),
            monday_low: mon.map(|m| m.low),
            monday_mid: mon.map(|m| m.mid),
            asia_open: asia.map(|s| s.open),
            asia_high: asia.map(|s| s.high),
            asia_low: asia.map(|s| s.low),
            london_open: ldn.map(|s| s.open),
            london_high: ldn.map(|s| s.high),
            london_low: ldn.map(|s| s.low),
            ny_open: ny.map(|s| s.open),
            ny_high: ny.map(|s| s.high),
            ny_low: ny.map(|s| s.low),
        });
    }

    result
}

/// Advance `cursor` so that it equals `partition_point(|item| key(item) <= t)`
/// on a sorted slice. Only valid when `t` is non-decreasing across calls.
#[inline]
fn advance_by_key<T>(
    items: &[T],
    cursor: &mut usize,
    t: DateTime<Utc>,
    key: impl Fn(&T) -> DateTime<Utc>,
) {
    while *cursor < items.len() && key(&items[*cursor]) <= t {
        *cursor += 1;
    }
}

/// Cursor-based equivalent of `open_prev_levels` — same semantics as the
/// bisect variant, given `cursor == partition_point(|c| c.open_time <= t)`.
fn open_prev_levels_cursor(
    candles: &[Candle],
    cursor: usize,
) -> (Option<f64>, Option<f64>, Option<f64>, Option<f64>) {
    if cursor == 0 {
        return (None, None, None, None);
    }
    let current = &candles[cursor - 1];
    if cursor < 2 {
        return (Some(current.open), None, None, None);
    }
    let prev = &candles[cursor - 2];
    let eq = (prev.high + prev.low) / 2.0;
    (
        Some(current.open),
        Some(prev.high),
        Some(prev.low),
        Some(eq),
    )
}

/// Cursor-based `quarterly_levels`.
fn quarterly_levels_cursor(
    quarters: &[AggPeriod],
    cursor: usize,
) -> (Option<f64>, Option<f64>, Option<f64>, Option<f64>) {
    if cursor == 0 {
        return (None, None, None, None);
    }
    if cursor < 2 {
        return (Some(quarters[0].open), None, None, None);
    }
    let prev = &quarters[cursor - 2];
    let eq = (prev.high + prev.low) / 2.0;
    (
        Some(quarters[cursor - 1].open),
        Some(prev.high),
        Some(prev.low),
        Some(eq),
    )
}

/// Cursor-based `yearly_levels`. `cursor` is per-year; caller resets it on
/// year transitions (the per-year arrays have independent keyspaces).
fn yearly_levels_cursor(
    yearly_data: &std::collections::HashMap<i32, YearlyRunning>,
    year: i32,
    cursor: &mut usize,
    t: DateTime<Utc>,
) -> (Option<f64>, Option<f64>, Option<f64>, Option<f64>) {
    let data = match yearly_data.get(&year) {
        Some(d) => d,
        None => return (None, None, None, None),
    };
    while *cursor < data.candle_close_times.len() && data.candle_close_times[*cursor] <= t {
        *cursor += 1;
    }
    if *cursor == 0 {
        return (Some(data.year_open), None, None, None);
    }
    let yh = data.cum_highs[*cursor - 1];
    let yl = data.cum_lows[*cursor - 1];
    (
        Some(data.year_open),
        Some(yh),
        Some(yl),
        Some((yh + yl) / 2.0),
    )
}

/// Cursor-based "last completed item before t".
#[inline]
fn latest_by_cursor<T>(items: &[T], cursor: usize) -> Option<&T> {
    if cursor == 0 {
        None
    } else {
        Some(&items[cursor - 1])
    }
}

/// Fetch multi-TF candles and compute key levels for a single symbol.
///
/// Uses `ensure_candles` to load each timeframe through the CandleStore,
/// avoiding redundant API calls when data is already cached.
/// Returns `Err` if any required timeframe has no data after fetch attempt.
pub fn fetch_symbol_key_levels(
    store: &mut CandleStore,
    client: &BinanceClient,
    symbol: &str,
    kl_start: DateTime<Utc>,
    hourly_start: DateTime<Utc>,
    end: DateTime<Utc>,
    timestamps: &[DateTime<Utc>],
) -> Result<Vec<(DateTime<Utc>, KeyLevels)>, String> {
    let h4 = crate::ensure_candles(store, client, symbol, "4h", kl_start, end)?;
    if h4.is_empty() {
        return Err(format!("4h: no data for {symbol} after fetch"));
    }
    let daily = crate::ensure_candles(store, client, symbol, "1d", kl_start, end)?;
    if daily.is_empty() {
        return Err(format!("1d: no data for {symbol} after fetch"));
    }
    let weekly = crate::ensure_candles(store, client, symbol, "1w", kl_start, end)?;
    if weekly.is_empty() {
        return Err(format!("1w: no data for {symbol} after fetch"));
    }
    let monthly = crate::ensure_candles(store, client, symbol, "1M", kl_start, end)?;
    if monthly.is_empty() {
        return Err(format!("1M: no data for {symbol} after fetch"));
    }
    // 1h already loaded by caller via ensure_candles; this will hit the store.
    let hourly = crate::ensure_candles(store, client, symbol, "1h", hourly_start, end)?;
    if hourly.is_empty() {
        return Err(format!("1h: no data for {symbol} after fetch"));
    }

    let kl_series = compute_key_levels_series(&h4, &daily, &weekly, &monthly, &hourly, timestamps);
    Ok(timestamps.iter().copied().zip(kl_series).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, TimeZone};

    fn dt(y: i32, m: u32, d: u32, h: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(y, m, d, h, 0, 0).unwrap()
    }

    fn mk_candle(open_time: DateTime<Utc>, dur: Duration, base: f64) -> Candle {
        Candle {
            open_time,
            close_time: open_time + dur,
            open: base,
            high: base + 2.0,
            low: base - 1.0,
            close: base + 0.5,
            volume: 100.0,
            taker_buy_volume: 50.0,
        }
    }

    fn synth_candles(start: DateTime<Utc>, end: DateTime<Utc>, step: Duration) -> Vec<Candle> {
        let mut out = Vec::new();
        let mut t = start;
        let mut base = 100.0_f64;
        while t < end {
            out.push(mk_candle(t, step, base));
            base += 0.25;
            t = t + step;
        }
        out
    }

    /// Binary-search-per-timestamp reference implementation — matches the
    /// pre-cursor behavior of `compute_key_levels_series`.
    fn compute_key_levels_series_binsearch(
        h4_candles: &[Candle],
        daily_candles: &[Candle],
        weekly_candles: &[Candle],
        monthly_candles: &[Candle],
        hourly_candles: &[Candle],
        timestamps: &[DateTime<Utc>],
    ) -> Vec<KeyLevels> {
        if timestamps.is_empty() {
            return Vec::new();
        }
        let quarters = aggregate_quarters(daily_candles);
        let yearly = build_yearly_running(daily_candles);
        let mondays = build_completed_mondays(hourly_candles);
        let asia = build_completed_sessions(hourly_candles, ASIA_START_HOUR, ASIA_END_HOUR);
        let ldn = build_completed_sessions(hourly_candles, LONDON_START_HOUR, LONDON_END_HOUR);
        let ny = build_completed_sessions(hourly_candles, NY_START_HOUR, NY_END_HOUR);

        let mut out = Vec::with_capacity(timestamps.len());
        for &t in timestamps {
            let h4 = open_prev_levels(h4_candles, t);
            let dl = open_prev_levels(daily_candles, t);
            let wk = open_prev_levels(weekly_candles, t);
            let mo = open_prev_levels(monthly_candles, t);
            let qt = quarterly_levels(&quarters, t);
            let yr = yearly_levels(&yearly, t);
            let mon = latest_monday_before(&mondays, t);
            let a = latest_session_before(&asia, t);
            let l = latest_session_before(&ldn, t);
            let n = latest_session_before(&ny, t);
            out.push(KeyLevels {
                h4_open: h4.0,
                prev_h4_high: h4.1,
                prev_h4_low: h4.2,
                h4_eq: h4.3,
                daily_open: dl.0,
                pdh: dl.1,
                pdl: dl.2,
                daily_eq: dl.3,
                weekly_open: wk.0,
                prev_week_high: wk.1,
                prev_week_low: wk.2,
                weekly_eq: wk.3,
                monthly_open: mo.0,
                prev_month_high: mo.1,
                prev_month_low: mo.2,
                monthly_eq: mo.3,
                quarterly_open: qt.0,
                prev_quarter_high: qt.1,
                prev_quarter_low: qt.2,
                quarterly_eq: qt.3,
                yearly_open: yr.0,
                yearly_high: yr.1,
                yearly_low: yr.2,
                yearly_eq: yr.3,
                monday_high: mon.map(|m| m.high),
                monday_low: mon.map(|m| m.low),
                monday_mid: mon.map(|m| m.mid),
                asia_open: a.map(|s| s.open),
                asia_high: a.map(|s| s.high),
                asia_low: a.map(|s| s.low),
                london_open: l.map(|s| s.open),
                london_high: l.map(|s| s.high),
                london_low: l.map(|s| s.low),
                ny_open: n.map(|s| s.open),
                ny_high: n.map(|s| s.high),
                ny_low: n.map(|s| s.low),
            });
        }
        out
    }

    fn eq_opt(a: Option<f64>, b: Option<f64>) -> bool {
        match (a, b) {
            (None, None) => true,
            (Some(x), Some(y)) => (x - y).abs() < 1e-12,
            _ => false,
        }
    }

    fn assert_key_levels_eq(cur: &[KeyLevels], ref_: &[KeyLevels]) {
        assert_eq!(cur.len(), ref_.len());
        for (i, (a, b)) in cur.iter().zip(ref_.iter()).enumerate() {
            let fields: [(Option<f64>, Option<f64>, &str); 35] = [
                (a.h4_open, b.h4_open, "h4_open"),
                (a.prev_h4_high, b.prev_h4_high, "prev_h4_high"),
                (a.prev_h4_low, b.prev_h4_low, "prev_h4_low"),
                (a.h4_eq, b.h4_eq, "h4_eq"),
                (a.daily_open, b.daily_open, "daily_open"),
                (a.pdh, b.pdh, "pdh"),
                (a.pdl, b.pdl, "pdl"),
                (a.daily_eq, b.daily_eq, "daily_eq"),
                (a.weekly_open, b.weekly_open, "weekly_open"),
                (a.prev_week_high, b.prev_week_high, "prev_week_high"),
                (a.prev_week_low, b.prev_week_low, "prev_week_low"),
                (a.weekly_eq, b.weekly_eq, "weekly_eq"),
                (a.monthly_open, b.monthly_open, "monthly_open"),
                (a.prev_month_high, b.prev_month_high, "prev_month_high"),
                (a.prev_month_low, b.prev_month_low, "prev_month_low"),
                (a.monthly_eq, b.monthly_eq, "monthly_eq"),
                (a.quarterly_open, b.quarterly_open, "quarterly_open"),
                (a.prev_quarter_high, b.prev_quarter_high, "prev_quarter_high"),
                (a.prev_quarter_low, b.prev_quarter_low, "prev_quarter_low"),
                (a.quarterly_eq, b.quarterly_eq, "quarterly_eq"),
                (a.yearly_open, b.yearly_open, "yearly_open"),
                (a.yearly_high, b.yearly_high, "yearly_high"),
                (a.yearly_low, b.yearly_low, "yearly_low"),
                (a.yearly_eq, b.yearly_eq, "yearly_eq"),
                (a.monday_high, b.monday_high, "monday_high"),
                (a.monday_low, b.monday_low, "monday_low"),
                (a.monday_mid, b.monday_mid, "monday_mid"),
                (a.asia_open, b.asia_open, "asia_open"),
                (a.asia_high, b.asia_high, "asia_high"),
                (a.asia_low, b.asia_low, "asia_low"),
                (a.london_open, b.london_open, "london_open"),
                (a.london_high, b.london_high, "london_high"),
                (a.london_low, b.london_low, "london_low"),
                (a.ny_open, b.ny_open, "ny_open"),
                (a.ny_high, b.ny_high, "ny_high"),
            ];
            for (x, y, name) in fields.iter() {
                assert!(eq_opt(*x, *y), "[{i}] {name}: cursor={x:?} ref={y:?}");
            }
            // ny_low is the 36th; split because array literal size above was 35.
            assert!(eq_opt(a.ny_low, b.ny_low), "[{i}] ny_low");
        }
    }

    #[test]
    fn cursor_matches_binsearch_multi_year() {
        let start = dt(2022, 10, 1, 0);
        let end = dt(2024, 2, 1, 0);
        let h4 = synth_candles(start, end, Duration::hours(4));
        let daily = synth_candles(start, end, Duration::days(1));
        let weekly = synth_candles(start, end, Duration::weeks(1));
        let monthly = synth_candles(start, end, Duration::days(30));
        let hourly = synth_candles(start, end, Duration::hours(1));

        // Probe timestamps at irregular strides to exercise cursor advancement.
        let mut timestamps = Vec::new();
        let mut t = start + Duration::hours(2);
        let strides = [
            Duration::hours(3),
            Duration::hours(5),
            Duration::hours(1),
            Duration::days(1),
            Duration::hours(11),
            Duration::days(7) + Duration::hours(2),
        ];
        let mut s = 0usize;
        while t < end {
            timestamps.push(t);
            t = t + strides[s % strides.len()];
            s += 1;
        }

        let cur = compute_key_levels_series(&h4, &daily, &weekly, &monthly, &hourly, &timestamps);
        let naive =
            compute_key_levels_series_binsearch(&h4, &daily, &weekly, &monthly, &hourly, &timestamps);
        assert_key_levels_eq(&cur, &naive);
    }

    #[test]
    fn cursor_handles_year_boundary() {
        // Deliberately straddle Dec 30 → Jan 2 to force yearly cursor reset.
        let start = dt(2023, 12, 28, 0);
        let end = dt(2024, 1, 5, 0);
        let daily = synth_candles(start, end, Duration::days(1));
        let hourly = synth_candles(start, end, Duration::hours(1));
        let h4 = synth_candles(start, end, Duration::hours(4));

        let mut timestamps = Vec::new();
        let mut t = start + Duration::hours(1);
        while t < end {
            timestamps.push(t);
            t = t + Duration::hours(6);
        }

        let cur =
            compute_key_levels_series(&h4, &daily, &[], &[], &hourly, &timestamps);
        let naive =
            compute_key_levels_series_binsearch(&h4, &daily, &[], &[], &hourly, &timestamps);
        assert_key_levels_eq(&cur, &naive);
    }

    #[test]
    fn cursor_empty_timestamps() {
        let out = compute_key_levels_series(&[], &[], &[], &[], &[], &[]);
        assert!(out.is_empty());
    }
}
