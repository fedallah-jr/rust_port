//! Build `ContextMap` from raw per-symbol sources.
//!
//! `build_context` is called once per run (after `fetch_sources` and
//! `build_btc_events`) to assemble the unified point-in-time channel that
//! strategies consume via `ctx.context_at(key, t)`.

use std::collections::HashMap;

use chrono::{DateTime, Duration, Utc};

use claude_trader_models::{
    ContextKey, ContextMap, ContextValue, FundingRate, KeyLevels, MarketBias, SeriesInput,
};

/// Warmup each context source needs before the evaluation window starts.
///
/// Funding: 30 days — `ensure_funding_rates` does not self-pad, so the caller
/// must fetch 30d before the first bar for `funding_context_at`'s z-score
/// window.
///
/// BtcStructure: 0 — `build_btc_events` internally extends the daily candle
/// fetch by 60 days.
///
/// KeyLevels: 0 — the key-levels fetch internally extends the 1h candle fetch
/// by 7 days.
pub fn context_warmup(key: &ContextKey) -> Duration {
    match key {
        ContextKey::Funding(_) => Duration::days(30),
        ContextKey::BtcStructure => Duration::zero(),
        ContextKey::KeyLevels(_) => Duration::zero(),
    }
}

/// Build the `ContextMap` for a run from raw sources.
///
/// `fetch_sources` is the sole producer of `funding_rates` and `key_levels`,
/// and only fetches symbols declared in `needed`. This function enforces the
/// contract: a `Funding(sym)` or `KeyLevels(sym)` in `needed` with no
/// corresponding entry in the respective source map is a fetcher bug, not a
/// silent empty series. It panics so the break is loud.
pub fn build_context(
    needed: &[ContextKey],
    funding_rates: &HashMap<String, Vec<FundingRate>>,
    key_levels: &HashMap<String, Vec<(DateTime<Utc>, KeyLevels)>>,
    btc_events: &[(DateTime<Utc>, MarketBias)],
) -> ContextMap {
    let mut entries: Vec<(ContextKey, SeriesInput)> = Vec::new();
    for key in needed {
        match key {
            ContextKey::BtcStructure => {
                let series: Vec<(DateTime<Utc>, ContextValue)> = btc_events
                    .iter()
                    .map(|(ts, bias)| (*ts, ContextValue::Bias(*bias)))
                    .collect();
                entries.push((key.clone(), SeriesInput::Point(series)));
            }
            ContextKey::KeyLevels(sym) => {
                let v = key_levels.get(sym).unwrap_or_else(|| {
                    panic!(
                        "build_context: KeyLevels({sym}) requested but not fetched \
                         — fetch_sources invariant broken"
                    )
                });
                let series: Vec<(DateTime<Utc>, ContextValue)> = v
                    .iter()
                    .map(|(ts, kl)| (*ts, ContextValue::KeyLevels(*kl)))
                    .collect();
                entries.push((key.clone(), SeriesInput::Point(series)));
            }
            ContextKey::Funding(sym) => {
                let rates = funding_rates
                    .get(sym)
                    .unwrap_or_else(|| {
                        panic!(
                            "build_context: Funding({sym}) requested but not fetched \
                             — fetch_sources invariant broken"
                        )
                    })
                    .clone();
                entries.push((key.clone(), SeriesInput::FundingRaw(rates)));
            }
        }
    }
    ContextMap::from_series(entries)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn warmup_is_30_days_for_funding_zero_for_others() {
        assert_eq!(
            context_warmup(&ContextKey::Funding("X".into())),
            Duration::days(30),
        );
        assert_eq!(context_warmup(&ContextKey::BtcStructure), Duration::zero());
        assert_eq!(
            context_warmup(&ContextKey::KeyLevels("X".into())),
            Duration::zero(),
        );
    }

    #[test]
    fn btc_structure_timestamps_are_source_events() {
        let d1 = Utc.with_ymd_and_hms(2024, 1, 1, 23, 59, 59).unwrap();
        let d2 = Utc.with_ymd_and_hms(2024, 1, 2, 23, 59, 59).unwrap();
        let events = vec![(d1, MarketBias::Bullish), (d2, MarketBias::Bearish)];
        let ctx = build_context(
            &[ContextKey::BtcStructure],
            &HashMap::new(),
            &HashMap::new(),
            &events,
        );
        // Query at an analysis-interval timestamp not equal to any source event.
        let t = Utc.with_ymd_and_hms(2024, 1, 2, 5, 0, 0).unwrap();
        match ctx.context_at(&ContextKey::BtcStructure, t) {
            Some(ContextValue::Bias(b)) => assert_eq!(b, MarketBias::Bullish),
            other => panic!("expected Bias(Bullish), got {other:?}"),
        }
    }

    #[test]
    #[should_panic(expected = "Funding(ETHUSDT) requested but not fetched")]
    fn funding_requested_but_not_fetched_panics() {
        let _ = build_context(
            &[ContextKey::Funding("ETHUSDT".into())],
            &HashMap::new(),
            &HashMap::new(),
            &[],
        );
    }

    #[test]
    #[should_panic(expected = "KeyLevels(ETHUSDT) requested but not fetched")]
    fn key_levels_requested_but_not_fetched_panics() {
        let _ = build_context(
            &[ContextKey::KeyLevels("ETHUSDT".into())],
            &HashMap::new(),
            &HashMap::new(),
            &[],
        );
    }
}
