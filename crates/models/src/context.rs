//! Unified point-in-time context channel for strategies.
//!
//! `ContextMap` is the single accessor strategies use for non-OHLCV signals
//! that are scalar at a point in time: BTC structural bias, key levels,
//! funding context. A strategy declares what it needs via
//! `ResearchStrategy::required_context()`; the runtime builds a `ContextMap`
//! once per run and hands clipped views to `generate_signals`.
//!
//! Invariant: for any bar at close_time `t`, the strategy can only observe
//! values sourced from events with `ts <= t`. Enforced because `context_at`
//! is the only public read path — there is no way to iterate raw series.

use std::collections::BTreeMap;

use chrono::{DateTime, Utc};

use crate::{FundingRate, KeyLevels, MarketBias};
use crate::funding::{funding_context_at, FundingContext};

/// What a strategy asks for in `required_context()`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ContextKey {
    BtcStructure,
    KeyLevels(String),
    Funding(String),
}

/// Value returned by `context_at`.
#[derive(Debug, Clone)]
pub enum ContextValue {
    Bias(MarketBias),
    KeyLevels(KeyLevels),
    Funding(FundingContext),
}

/// Input shape for `ContextMap::from_series`. Builders choose the
/// representation appropriate to the key.
pub enum SeriesInput {
    /// Cached point-in-time entries. Used by BtcStructure and KeyLevels.
    /// Must be strictly sorted by timestamp.
    Point(Vec<(DateTime<Utc>, ContextValue)>),
    /// Raw funding rates, held privately. `context_at` computes
    /// `funding_context_at(rates, t)` on demand so 7d/30d windows are
    /// evaluated against the query time `t`, not against the most recent
    /// funding event.
    FundingRaw(Vec<FundingRate>),
}

/// The heterogeneous internal representation. Not part of the public API —
/// `ContextMap` fields are private, so strategies can never reach a raw
/// series or bypass `context_at`.
#[derive(Debug, Clone)]
enum SeriesRepr {
    Point(Vec<(DateTime<Utc>, ContextValue)>),
    FundingRaw(Vec<FundingRate>),
}

/// The unified point-in-time context handed to strategies.
#[derive(Debug, Clone, Default)]
pub struct ContextMap {
    series: BTreeMap<ContextKey, SeriesRepr>,
}

impl ContextMap {
    /// Build a ContextMap from a set of series, one per key. Debug-panics if
    /// any series is not strictly sorted by timestamp. Release builds accept
    /// unsorted input (providers are in-tree and tested).
    pub fn from_series(entries: Vec<(ContextKey, SeriesInput)>) -> Self {
        let mut series: BTreeMap<ContextKey, SeriesRepr> = BTreeMap::new();
        for (key, input) in entries {
            let repr = match input {
                SeriesInput::Point(v) => {
                    debug_assert!(
                        v.windows(2).all(|w| w[0].0 < w[1].0),
                        "Point series for {key:?} not strictly sorted by timestamp",
                    );
                    SeriesRepr::Point(v)
                }
                SeriesInput::FundingRaw(rates) => {
                    debug_assert!(
                        rates.windows(2).all(|w| w[0].timestamp < w[1].timestamp),
                        "FundingRaw series for {key:?} not strictly sorted by timestamp",
                    );
                    SeriesRepr::FundingRaw(rates)
                }
            };
            series.insert(key, repr);
        }
        Self { series }
    }

    /// Look up context at time `t`.
    ///
    /// For `Point` series: returns the last `(ts, v)` with `ts <= t`, cloned.
    /// For `FundingRaw` series: returns `funding_context_at(rates, t)` wrapped
    /// in `ContextValue::Funding`. Uses `partition_point` internally, so the
    /// 7d/30d windows are always computed against `t`.
    ///
    /// Returns `None` if the key was not requested or no source event is
    /// visible at `t`.
    pub fn context_at(&self, key: &ContextKey, t: DateTime<Utc>) -> Option<ContextValue> {
        match self.series.get(key)? {
            SeriesRepr::Point(v) => {
                let idx = v.partition_point(|(ts, _)| *ts <= t);
                if idx == 0 {
                    None
                } else {
                    Some(v[idx - 1].1.clone())
                }
            }
            SeriesRepr::FundingRaw(rates) => {
                funding_context_at(rates, t).map(ContextValue::Funding)
            }
        }
    }

    /// Drop all source entries with `ts > end`. Used by the runtime and
    /// validator to materialise a per-signal / per-period view before handing
    /// it to a strategy.
    pub fn clip_in_place(&mut self, end: DateTime<Utc>) {
        for repr in self.series.values_mut() {
            match repr {
                SeriesRepr::Point(v) => {
                    let idx = v.partition_point(|(ts, _)| *ts <= end);
                    v.truncate(idx);
                }
                SeriesRepr::FundingRaw(rates) => {
                    let idx = rates.partition_point(|r| r.timestamp <= end);
                    rates.truncate(idx);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, TimeZone};

    fn ts(i: i64) -> DateTime<Utc> {
        Utc.timestamp_opt(i, 0).unwrap()
    }

    #[test]
    fn point_series_returns_none_before_first_stamp() {
        let ctx = ContextMap::from_series(vec![(
            ContextKey::BtcStructure,
            SeriesInput::Point(vec![(ts(100), ContextValue::Bias(MarketBias::Bullish))]),
        )]);
        assert!(ctx.context_at(&ContextKey::BtcStructure, ts(99)).is_none());
    }

    #[test]
    fn point_series_returns_last_event_at_or_before_t() {
        let ctx = ContextMap::from_series(vec![(
            ContextKey::BtcStructure,
            SeriesInput::Point(vec![
                (ts(100), ContextValue::Bias(MarketBias::Bullish)),
                (ts(200), ContextValue::Bias(MarketBias::Bearish)),
            ]),
        )]);
        match ctx.context_at(&ContextKey::BtcStructure, ts(150)) {
            Some(ContextValue::Bias(b)) => assert_eq!(b, MarketBias::Bullish),
            other => panic!("expected Bias(Bullish), got {other:?}"),
        }
        match ctx.context_at(&ContextKey::BtcStructure, ts(250)) {
            Some(ContextValue::Bias(b)) => assert_eq!(b, MarketBias::Bearish),
            other => panic!("expected Bias(Bearish), got {other:?}"),
        }
    }

    #[test]
    fn point_series_visible_at_exact_ts() {
        let ctx = ContextMap::from_series(vec![(
            ContextKey::BtcStructure,
            SeriesInput::Point(vec![(ts(100), ContextValue::Bias(MarketBias::Bullish))]),
        )]);
        assert!(ctx.context_at(&ContextKey::BtcStructure, ts(100)).is_some());
    }

    fn make_rates(hours_and_values: &[(i64, f64)]) -> Vec<FundingRate> {
        let base = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        hours_and_values
            .iter()
            .map(|&(h, r)| FundingRate {
                timestamp: base + Duration::hours(h),
                funding_rate: r,
                mark_price: None,
            })
            .collect()
    }

    #[test]
    fn funding_on_demand_matches_direct_call() {
        // 8h spacing, 20 rates over ~6.5 days
        let rates = make_rates(&[
            (0, 0.0001), (8, 0.0002), (16, 0.0003), (24, 0.0001),
            (32, 0.0002), (40, 0.0003), (48, 0.0001), (56, 0.0002),
            (64, 0.0003), (72, 0.0001), (80, 0.0002), (88, 0.0003),
            (96, 0.0004), (104, 0.0005), (112, 0.0006), (120, 0.0007),
            (128, 0.0008), (136, 0.0009), (144, 0.0010), (152, 0.0011),
        ]);
        let ctx = ContextMap::from_series(vec![(
            ContextKey::Funding("X".to_string()),
            SeriesInput::FundingRaw(rates.clone()),
        )]);

        let t = Utc.with_ymd_and_hms(2024, 1, 5, 0, 0, 0).unwrap();
        let expected = funding_context_at(&rates, t).unwrap();
        let got = match ctx.context_at(&ContextKey::Funding("X".to_string()), t).unwrap() {
            ContextValue::Funding(f) => f,
            other => panic!("expected Funding, got {other:?}"),
        };
        assert_eq!(got.rate, expected.rate);
        assert_eq!(got.cumulative_7d, expected.cumulative_7d);
        assert_eq!(got.zscore_30d, expected.zscore_30d);
        assert_eq!(got.rate_change, expected.rate_change);
    }

    #[test]
    fn funding_rolling_window_uses_query_time_not_last_event() {
        // Regression test for the v2 bug: precomputing at funding timestamps
        // and reusing the last one for arbitrary later `t` would drift as
        // old rates age out of the 7d window between funding events.
        //
        // Construct rates such that the cumulative_7d at t_query differs
        // from the cumulative_7d at the most recent funding event before it.
        let base = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        let mut hv: Vec<(i64, f64)> = Vec::new();
        // Rate at h=0 (will age out between event at h=168 and query t at h=168+5)
        hv.push((0, 0.01));
        // Rates every 8h from h=8 through h=176, small constant
        for h in (8..=176).step_by(8) {
            hv.push((h, 0.0001));
        }
        let rates = make_rates(&hv);

        let last_event_ts = base + Duration::hours(168);
        // Query slightly after h=168, but chosen so that (t - 7d) has just
        // passed h=0, aging the h=0 spike out of the 7d window.
        let t_query = base + Duration::hours(168) + Duration::hours(5);

        let at_event = funding_context_at(&rates, last_event_ts).unwrap();
        let at_query = funding_context_at(&rates, t_query).unwrap();
        // The spike at h=0 is in at_event's window but out of at_query's.
        assert!(
            at_event.cumulative_7d > at_query.cumulative_7d,
            "expected event.cumulative_7d ({}) > query.cumulative_7d ({})",
            at_event.cumulative_7d, at_query.cumulative_7d,
        );

        let ctx = ContextMap::from_series(vec![(
            ContextKey::Funding("X".into()),
            SeriesInput::FundingRaw(rates.clone()),
        )]);
        let got = match ctx.context_at(&ContextKey::Funding("X".into()), t_query).unwrap() {
            ContextValue::Funding(f) => f,
            other => panic!("expected Funding, got {other:?}"),
        };
        // ContextMap's context_at must match the on-demand value at t_query,
        // not the precomputed-at-event value.
        assert_eq!(got.cumulative_7d, at_query.cumulative_7d);
        assert_ne!(got.cumulative_7d, at_event.cumulative_7d);
    }

    #[test]
    fn funding_returns_none_with_fewer_than_three_rates() {
        let rates = make_rates(&[(0, 0.0001), (8, 0.0002)]);
        let ctx = ContextMap::from_series(vec![(
            ContextKey::Funding("X".into()),
            SeriesInput::FundingRaw(rates),
        )]);
        let t = Utc.with_ymd_and_hms(2024, 1, 1, 16, 0, 0).unwrap();
        assert!(ctx.context_at(&ContextKey::Funding("X".into()), t).is_none());
    }

    #[test]
    fn clip_in_place_drops_entries_after_end() {
        let mut ctx = ContextMap::from_series(vec![
            (
                ContextKey::BtcStructure,
                SeriesInput::Point(vec![
                    (ts(100), ContextValue::Bias(MarketBias::Bullish)),
                    (ts(200), ContextValue::Bias(MarketBias::Bearish)),
                    (ts(300), ContextValue::Bias(MarketBias::Neutral)),
                ]),
            ),
            (
                ContextKey::Funding("X".into()),
                SeriesInput::FundingRaw(make_rates(&[(0, 0.1), (8, 0.2), (16, 0.3)])),
            ),
        ]);
        ctx.clip_in_place(ts(250));
        // Point: event at 300 dropped; events at 100 and 200 kept.
        match ctx.context_at(&ContextKey::BtcStructure, ts(350)) {
            Some(ContextValue::Bias(b)) => assert_eq!(b, MarketBias::Bearish),
            other => panic!("expected Bias(Bearish), got {other:?}"),
        }
    }

    #[test]
    #[should_panic(expected = "not strictly sorted")]
    fn from_series_panics_in_debug_on_unsorted_point() {
        let _ = ContextMap::from_series(vec![(
            ContextKey::BtcStructure,
            SeriesInput::Point(vec![
                (ts(200), ContextValue::Bias(MarketBias::Bullish)),
                (ts(100), ContextValue::Bias(MarketBias::Bearish)),
            ]),
        )]);
    }

    #[test]
    fn missing_key_returns_none() {
        let ctx = ContextMap::default();
        assert!(ctx.context_at(&ContextKey::BtcStructure, ts(100)).is_none());
    }
}
