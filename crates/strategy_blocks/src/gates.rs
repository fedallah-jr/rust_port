//! Gate helpers: thin, pure wrappers over `MarketBias`, funding-context,
//! and 4h higher-timeframe context — the three filters that show up in
//! virtually every signal path.
//!
//! Everything here is a stateless function of its arguments.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use claude_trader_models::{funding_context_at, Candle, FundingRate, MarketBias};

// ---------------------------------------------------------------------------
// BTC bias
// ---------------------------------------------------------------------------

/// Returns `true` when the BTC daily-structure bias at time `t` is
/// `Bullish`. `None` / missing entries are treated as not-bullish.
#[inline]
pub fn btc_bias_bullish(map: &HashMap<DateTime<Utc>, MarketBias>, t: DateTime<Utc>) -> bool {
    matches!(map.get(&t), Some(MarketBias::Bullish))
}

/// Returns `true` when the bias is `Bearish`. `None` / missing treated
/// as not-bearish.
#[inline]
pub fn btc_bias_bearish(map: &HashMap<DateTime<Utc>, MarketBias>, t: DateTime<Utc>) -> bool {
    matches!(map.get(&t), Some(MarketBias::Bearish))
}

// ---------------------------------------------------------------------------
// Funding-rate positioning gates
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
pub struct FundingGateParams {
    /// Absolute value of the 30-day z-score required. Positive for
    /// `funding_crowded_long` (longs carrying cost), negative-implicit
    /// for `funding_crowded_short` (via `-z_min`).
    pub z_min: f64,
    /// Absolute value of the cumulative 7-day rate required.
    pub cum_7d_min: f64,
}

/// `true` when retail is crowded LONG: funding z-score > `z_min` OR
/// 7-day cumulative > `cum_7d_min`. Matches the opus47_2 v25 gate used
/// for short entries.
#[inline]
pub fn funding_crowded_long(
    rates: Option<&Vec<FundingRate>>,
    t: DateTime<Utc>,
    params: &FundingGateParams,
) -> bool {
    match rates.and_then(|r| funding_context_at(r, t)) {
        Some(ctx) => ctx.zscore_30d > params.z_min || ctx.cumulative_7d > params.cum_7d_min,
        None => false,
    }
}

/// Mirror for long entries: retail is crowded SHORT when z-score <
/// `-z_min` OR 7-day cumulative < `-cum_7d_min`.
#[inline]
pub fn funding_crowded_short(
    rates: Option<&Vec<FundingRate>>,
    t: DateTime<Utc>,
    params: &FundingGateParams,
) -> bool {
    match rates.and_then(|r| funding_context_at(r, t)) {
        Some(ctx) => ctx.zscore_30d < -params.z_min || ctx.cumulative_7d < -params.cum_7d_min,
        None => false,
    }
}

// ---------------------------------------------------------------------------
// 4h higher-timeframe soft filter (V20-style)
// ---------------------------------------------------------------------------

fn latest_and_prior_4h<'a>(
    additional_candles_4h: Option<&'a Vec<Candle>>,
    t: DateTime<Utc>,
) -> Option<(&'a Candle, &'a Candle)> {
    let bars = additional_candles_4h?;
    if bars.len() < 3 {
        return None;
    }
    let idx = bars.partition_point(|c| c.close_time < t);
    if idx < 2 {
        return None;
    }
    Some((&bars[idx - 1], &bars[idx - 2]))
}

/// "Soft bearish" 4h context: accept short entries when the latest 4h
/// candle is red OR closed below the prior 4h candle.
///
/// Matches the V20-style filter used in opus46's squeeze research and
/// opus47_2 / opus47_4's divergence-short 4h gate.
///
/// `None` for the 4h series means "no data; don't block" — returns
/// `true`.
#[inline]
pub fn htf_4h_not_strongly_bullish(
    additional_candles_4h: Option<&Vec<Candle>>,
    t: DateTime<Utc>,
) -> bool {
    match latest_and_prior_4h(additional_candles_4h, t) {
        Some((latest, prev)) => latest.close < latest.open || latest.close < prev.close,
        None => true,
    }
}

/// Mirror for longs: accept when the latest 4h candle is green OR
/// closed above the prior 4h candle. `None` → allow.
#[inline]
pub fn htf_4h_not_strongly_bearish(
    additional_candles_4h: Option<&Vec<Candle>>,
    t: DateTime<Utc>,
) -> bool {
    match latest_and_prior_4h(additional_candles_4h, t) {
        Some((latest, prev)) => latest.close > latest.open || latest.close > prev.close,
        None => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn ts(h: i64) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap() + chrono::Duration::hours(h)
    }

    #[test]
    fn bias_helpers() {
        let mut map = HashMap::new();
        map.insert(ts(0), MarketBias::Bullish);
        map.insert(ts(1), MarketBias::Bearish);
        map.insert(ts(2), MarketBias::Neutral);
        assert!(btc_bias_bullish(&map, ts(0)));
        assert!(!btc_bias_bearish(&map, ts(0)));
        assert!(btc_bias_bearish(&map, ts(1)));
        assert!(!btc_bias_bullish(&map, ts(1)));
        assert!(!btc_bias_bullish(&map, ts(2)));
        assert!(!btc_bias_bearish(&map, ts(2)));
        assert!(!btc_bias_bullish(&map, ts(99))); // missing entry
    }

    fn make_candle(t: DateTime<Utc>, open: f64, high: f64, low: f64, close: f64) -> Candle {
        Candle {
            open_time: t - chrono::Duration::hours(4),
            close_time: t,
            open,
            high,
            low,
            close,
            volume: 1.0,
            taker_buy_volume: 0.5,
        }
    }

    #[test]
    fn htf_4h_bullish_gate_accepts_red_candle() {
        let bars = vec![
            make_candle(ts(0), 100.0, 102.0, 99.0, 101.0),
            make_candle(ts(4), 101.0, 103.0, 100.0, 102.0), // prev 4h (green, rising)
            make_candle(ts(8), 102.0, 102.5, 100.0, 100.5), // latest 4h (red)
        ];
        // At t>=8 the latest is red -> not-strongly-bullish is TRUE.
        assert!(htf_4h_not_strongly_bullish(Some(&bars), ts(9)));
    }

    #[test]
    fn htf_4h_bullish_gate_rejects_green_hh_close() {
        let bars = vec![
            make_candle(ts(0), 100.0, 102.0, 99.0, 101.0),
            make_candle(ts(4), 101.0, 103.0, 100.0, 102.0),
            make_candle(ts(8), 102.0, 105.0, 101.0, 104.5), // latest 4h: green AND HH close
        ];
        assert!(!htf_4h_not_strongly_bullish(Some(&bars), ts(9)));
    }

    #[test]
    fn htf_4h_no_data_allows() {
        assert!(htf_4h_not_strongly_bullish(None, ts(9)));
        assert!(htf_4h_not_strongly_bearish(None, ts(9)));
    }
}
