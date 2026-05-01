//! Live adapter for the `opus46_max_16apr26_1` research strategy.
//!
//! The research strategy implements `ResearchStrategy::generate_signals` —
//! a batch operation over a prepared `BTreeMap<symbol, &[Candle]>` plus a
//! `ContextMap` and `HtfData`. This crate wraps that batch entry point in a
//! `LiveSignalGenerator` that:
//!
//!   1. Warms 1h candles for every declared symbol,
//!   2. Warms 4h candles + computes the strategy's 4h indicator set,
//!   3. Warms BTC daily candles and runs `DailyStructureProvider` to derive
//!      `BtcStructure` market bias,
//!   4. Warms those inputs once, then keeps them hot with incremental
//!      boundary refreshes: the normal hourly poll fetches only the newly
//!      closed 1h bar per symbol; 4h/funding/BTC are skipped until they can
//!      have changed,
//!   5. Calls `generate_signals` with `start = latest_closed_bar.close_time`
//!      and `end = start + 1ms` so only the freshest closed bar can emit,
//!   6. Validates each signal and dedupes by `(ticker, signal_date)` so a
//!      retry within the same hour cannot place the same trade twice.
//!
//! Funding-rate context is fetched per declared symbol via
//! `LiveMarketClient::fetch_funding_rates` (Binance USD-M `/fapi/v1/fundingRate`)
//! over a 30d lookback so `funding_context_at` has enough history for both
//! the 7d sum and 30d z-score; opus46 only reads `f.rate`, but the same shape
//! is reused so other strategies wired through this adapter see consistent
//! statistics. A symbol whose fetch fails or returns sparse data is treated
//! as a missing required-context input — the poll fails recoverable rather
//! than emitting trades with mismatched sizing.

use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;
use std::thread;
use std::time::Instant;

use std::path::PathBuf;

use chrono::{DateTime, Duration, TimeZone, Utc};
use claude_trader_btc_structure::{engine::FeatureValue, DailyStructureProvider};
use claude_trader_indicators::{compute_indicators, required_warmup, OhlcvFrame};
use claude_trader_live::market_client::LiveMarketClient;
use claude_trader_live::signal_generator::{LiveSignalGenerator, SignalError};
use claude_trader_models::{
    floor_boundary, Candle, ContextKey, ContextMap, ContextValue, FundingRate, HtfData, MarketBias,
    SeriesInput, Signal,
};
use claude_trader_research_runtime::ResearchStrategy;
use ct_research_opus46_max_16apr26_1::Opus46Max16apr261;

mod cooldown;
pub use cooldown::CooldownStore;

const STRATEGY_ID: &str = "opus46_max_16apr26_1_v24";
const ANALYSIS_INTERVAL: &str = "1h";

const INDICATOR_COLUMNS_1H: &[&str] = &[
    "atr_14",
    "atr_ratio",
    "kc_upper",
    "kc_lower",
    "squeeze_on",
    "squeeze_count",
    "mom_slope",
    "ema_20",
    "vol_ratio",
    "body_ratio",
];
const INDICATOR_COLUMNS_4H: &[&str] = &[
    "atr_14",
    "atr_ratio",
    "kc_upper",
    "kc_lower",
    "squeeze_on",
    "squeeze_count",
    "mom_slope",
    "ema_20",
    "vol_ratio",
    "body_ratio",
];

/// Live universe — mirrors the live-safe research universe.
///
/// `MATICUSDT` was delisted from Binance USD-M futures after the Polygon ->
/// POL token migration; `POLUSDT` is the on-exchange replacement. We do NOT
/// silently substitute `POLUSDT` because:
///   - `try_fetch_funding` is fail-closed on missing-symbol funding data,
///     and `MATICUSDT` would block every poll.
///   - Adding `POLUSDT` to live without re-running the backtest changes
///     the strategy's evaluation universe in ways the operator hasn't
///     validated. Quietly swapping symbols is a parity violation.
///
/// Action when the operator wants to re-include POL: re-backtest opus46
/// with the updated symbol list, then bump this constant.
const SYMBOLS: &[&str] = &[
    "BTCUSDT",
    "ETHUSDT",
    "SOLUSDT",
    "NEARUSDT",
    "DOTUSDT",
    "ADAUSDT",
    "APTUSDT",
    "LTCUSDT",
    "AVAXUSDT",
    "LINKUSDT",
    "BNBUSDT",
    "ARBUSDT",
    "INJUSDT",
    "RENDERUSDT",
    "XRPUSDT",
    "DOGEUSDT",
    "SUIUSDT",
    "TONUSDT",
    "FILUSDT",
    "OPUSDT",
    "TRXUSDT",
];

/// Extra 1h bars beyond `required_warmup(INDICATOR_COLUMNS_1H)`. Mirrors the
/// research strategy's `extra_warmup_bars()` (120) so live indicator values
/// at the decision bar match the backtest exactly. Below 120 the EWMs and
/// rolling stats can drift from their backtested values.
const EXTRA_1H_BARS: i64 = 120;
/// Extra 4h bars beyond `required_warmup(INDICATOR_COLUMNS_4H)`. Mirrors the
/// research strategy's `extra_warmup_bars_per_interval()["4h"] = 80`.
const EXTRA_4H_BARS: i64 = 80;
/// BTCUSDT perpetual listing date on Binance. Live computes BTC structure
/// from full available futures history so the path-dependent structure state
/// is not seeded from a short rolling window.
const BTC_LISTING_START_YMD: (i32, u32, u32) = (2019, 9, 8);
/// Lookback for funding-rate fetch. Funding posts every 8h on Binance USD-M
/// futures, and `funding_context_at` references a 30-day window to compute
/// z-score. opus46 only reads `f.rate` (not the windowed metrics), so 30
/// days is overkill for parity — but it keeps the live `FundingContext`
/// shape identical to the research one and gives downstream strategies
/// consistent z-scores if reused.
const FUNDING_LOOKBACK_DAYS: i64 = 30;

pub struct Opus46Live {
    symbols: Vec<String>,
    strategy: Opus46Max16apr261,
    market_client: Option<Arc<dyn LiveMarketClient>>,
    poll_time: DateTime<Utc>,
    candles_1h: HashMap<String, Vec<Candle>>,
    candles_4h: HashMap<String, Vec<Candle>>,
    btc_events: Vec<(DateTime<Utc>, ContextValue)>,
    funding: HashMap<String, Vec<FundingRate>>,
    last_refresh: Option<DateTime<Utc>>,
    cooldown: CooldownStore,
    cooldown_path: PathBuf,
}

struct RefreshOutcome {
    fetched: usize,
    rows: usize,
    failures: Vec<String>,
}

impl Opus46Live {
    pub fn new() -> Self {
        let path = CooldownStore::default_path(STRATEGY_ID);
        Self {
            symbols: SYMBOLS.iter().map(|s| s.to_string()).collect(),
            strategy: Opus46Max16apr261,
            market_client: None,
            poll_time: DateTime::<Utc>::MIN_UTC,
            candles_1h: HashMap::new(),
            candles_4h: HashMap::new(),
            btc_events: Vec::new(),
            funding: HashMap::new(),
            last_refresh: None,
            cooldown: CooldownStore::load(&path),
            cooldown_path: path,
        }
    }

    /// Override the cooldown state file location. Tests use this to point
    /// at an isolated tmp directory.
    pub fn set_cooldown_path(&mut self, path: PathBuf) {
        self.cooldown = CooldownStore::load(&path);
        self.cooldown_path = path;
    }

    fn client(&self) -> Result<&Arc<dyn LiveMarketClient>, SignalError> {
        self.market_client
            .as_ref()
            .ok_or_else(|| SignalError::fatal("setup() not called before poll()"))
    }

    /// Try fetching BTC daily candles for structure derivation. Mirrors the
    /// research_runtime fallback chain: 1d first, then synthesise 1d from
    /// 1h if 1d returns empty/errors. If both routes yield no candles the
    /// strategy MUST NOT trade — the BTC bias filter is required input,
    /// and falling open here means non-BTC shorts in a bullish regime
    /// trade harder than the backtest. Returns Recoverable so the engine
    /// logs and skips the poll while staying alive.
    fn try_build_btc_events(
        client: &Arc<dyn LiveMarketClient>,
        now: DateTime<Utc>,
    ) -> Result<Vec<(DateTime<Utc>, ContextValue)>, SignalError> {
        let start = btc_listing_start();
        let mut candles: Vec<Candle> = Vec::new();
        match client.fetch_klines("BTCUSDT", "1d", start, now) {
            Ok(c) if !c.is_empty() => candles = c,
            Ok(_) => log::warn!("opus46: BTC 1d empty; trying 1h synthesis fallback"),
            Err(e) => log::warn!(
                "opus46: BTC 1d fetch failed: {}; trying 1h synthesis fallback",
                e
            ),
        }
        if candles.is_empty() {
            match client.fetch_klines("BTCUSDT", "1h", start, now) {
                Ok(hourly) if !hourly.is_empty() => {
                    candles = synthesize_daily_from_hourly(&hourly);
                }
                Ok(_) => log::warn!("opus46: BTC 1h synthesis fallback returned no candles"),
                Err(e) => log::warn!("opus46: BTC 1h synthesis fallback fetch failed: {}", e),
            }
        }
        if candles.is_empty() {
            return Err(SignalError::recoverable(
                "opus46: BTC structure required-context unavailable: \
                 both 1d and 1h synthesis fallback returned no candles",
            ));
        }
        candles.retain(|c| c.close_time <= now);
        if candles.is_empty() {
            return Err(SignalError::recoverable(
                "opus46: BTC structure has no closed daily candles at poll time",
            ));
        }

        let mut provider = DailyStructureProvider::new();
        provider.compute_from_candles(&candles);
        let matrix = match provider.feature_matrix() {
            Some(m) => m,
            None => {
                return Err(SignalError::recoverable(
                    "opus46: BTC structure provider produced no feature matrix",
                ));
            }
        };

        let mut events: Vec<(DateTime<Utc>, ContextValue)> = Vec::with_capacity(candles.len());
        for (i, c) in candles.iter().enumerate() {
            if i >= matrix.len() {
                break;
            }
            if let Some(FeatureValue::Str(s)) = matrix[i].get("market_bias_after_close") {
                let bias = MarketBias::from_lowercase_str(s.as_ref());
                events.push((c.close_time, ContextValue::Bias(bias)));
            }
        }
        if events.is_empty() {
            return Err(SignalError::recoverable(
                "opus46: BTC structure produced 0 events from non-empty candle history",
            ));
        }
        Ok(events)
    }

    fn warmup_1h_bars() -> i64 {
        required_warmup(INDICATOR_COLUMNS_1H) as i64 + EXTRA_1H_BARS
    }

    fn warmup_4h_bars() -> i64 {
        required_warmup(INDICATOR_COLUMNS_4H) as i64 + EXTRA_4H_BARS
    }

    fn min_closed_for_interval(interval: &str) -> usize {
        if interval == "1h" {
            Self::warmup_1h_bars() as usize
        } else {
            Self::warmup_4h_bars() as usize
        }
    }

    fn merge_candles(dst: &mut Vec<Candle>, incoming: Vec<Candle>, max_len: usize) {
        let mut by_open: BTreeMap<DateTime<Utc>, Candle> = BTreeMap::new();
        for c in dst.drain(..).chain(incoming) {
            by_open.insert(c.open_time, c);
        }
        *dst = by_open.into_values().collect();
        if dst.len() > max_len {
            let drop = dst.len() - max_len;
            dst.drain(0..drop);
        }
    }

    fn merge_funding(dst: &mut Vec<FundingRate>, incoming: Vec<FundingRate>, now: DateTime<Utc>) {
        let mut by_ts: BTreeMap<DateTime<Utc>, FundingRate> = BTreeMap::new();
        for r in dst.drain(..).chain(incoming) {
            by_ts.insert(r.timestamp, r);
        }
        *dst = by_ts.into_values().collect();
        let cutoff = now - Duration::days(FUNDING_LOOKBACK_DAYS + 2);
        let keep_from = dst.partition_point(|r| r.timestamp < cutoff);
        if keep_from > 0 {
            dst.drain(0..keep_from);
        }
    }

    fn closed_count(candles: &[Candle], now: DateTime<Utc>) -> usize {
        candles.iter().filter(|c| c.close_time <= now).count()
    }

    fn interval_unit_hours(interval: &str) -> i64 {
        match interval {
            "4h" => 4,
            _ => 1,
        }
    }

    fn indicator_at(indicators: &HashMap<String, Vec<f64>>, col: &str, idx: usize) -> Option<f64> {
        indicators
            .get(col)
            .and_then(|v| v.get(idx).copied())
            .filter(|v| !v.is_nan())
    }

    fn validate_indicator_warmup(
        symbol: &str,
        interval: &str,
        candles: &[Candle],
        columns: &[&str],
        min_closed: usize,
        now: DateTime<Utc>,
    ) -> Result<usize, String> {
        let last_closed_idx = candles
            .iter()
            .rposition(|c| c.close_time <= now)
            .ok_or_else(|| format!("{symbol} {interval}: no closed candles at {now}"))?;
        let closed = last_closed_idx + 1;
        if closed < min_closed {
            return Err(format!(
                "{symbol} {interval}: only {closed} closed candles, need {min_closed}"
            ));
        }
        let unit_hours = Self::interval_unit_hours(interval);
        let latest_expected_open = floor_boundary(now, interval)
            .map(|b| b - Duration::hours(unit_hours))
            .map_err(|e| format!("{symbol} {interval}: invalid interval: {e}"))?;
        if !candles
            .iter()
            .any(|c| c.open_time == latest_expected_open && c.close_time <= now)
        {
            return Err(format!(
                "{symbol} {interval}: missing latest expected closed candle open={} at {}",
                latest_expected_open.format("%Y-%m-%d %H:%M:%S"),
                now.format("%Y-%m-%d %H:%M:%S"),
            ));
        }
        let ohlcv = OhlcvFrame {
            open: candles.iter().map(|c| c.open).collect(),
            high: candles.iter().map(|c| c.high).collect(),
            low: candles.iter().map(|c| c.low).collect(),
            close: candles.iter().map(|c| c.close).collect(),
            volume: candles.iter().map(|c| c.volume).collect(),
            taker_buy_volume: candles.iter().map(|c| c.taker_buy_volume).collect(),
        };
        let indicators = compute_indicators(&ohlcv, columns)
            .map_err(|e| format!("{symbol} {interval}: indicator compute failed: {e}"))?;
        if Self::indicator_at(&indicators, "mom_slope", last_closed_idx).is_none()
            || Self::indicator_at(&indicators, "atr_ratio", last_closed_idx).is_none()
        {
            return Err(format!(
                "{symbol} {interval}: warmup indicators incomplete after {closed} closed candles"
            ));
        }
        Ok(closed)
    }

    fn interval_fetch_start(
        candles: Option<&Vec<Candle>>,
        interval: &str,
        unit_hours: i64,
        bars: i64,
        min_closed: usize,
        now: DateTime<Utc>,
        last_refresh: Option<DateTime<Utc>>,
    ) -> Option<DateTime<Utc>> {
        let full_start = now - Duration::hours(unit_hours * (bars + 2));
        let Some(candles) = candles else {
            return Some(full_start);
        };
        if Self::closed_count(candles, now) < min_closed {
            return Some(full_start);
        }
        let latest_expected_open = floor_boundary(now, interval)
            .map(|b| b - Duration::hours(unit_hours))
            .unwrap_or_else(|_| {
                candles
                    .iter()
                    .rev()
                    .find(|c| c.close_time <= now)
                    .map(|c| c.open_time)
                    .unwrap_or(full_start)
            });

        let latest_closed = candles
            .iter()
            .rev()
            .find(|c| c.close_time <= now && c.open_time == latest_expected_open);
        if latest_closed.is_none() {
            return Some(latest_expected_open);
        }

        let Some(last_refresh) = last_refresh else {
            return Some(full_start);
        };
        let interval_ms = Duration::hours(unit_hours).num_milliseconds();
        candles
            .iter()
            .find(|c| {
                c.open_time >= latest_expected_open
                    && c.close_time > last_refresh
                    && c.close_time <= now
                    && (c.close_time - c.open_time).num_milliseconds() <= interval_ms
            })
            .map(|c| c.open_time)
    }

    fn refresh_interval_cache(
        client: &Arc<dyn LiveMarketClient>,
        symbols: &[String],
        cache: &mut HashMap<String, Vec<Candle>>,
        interval: &str,
        unit_hours: i64,
        bars: i64,
        now: DateTime<Utc>,
        last_refresh: Option<DateTime<Utc>>,
    ) -> RefreshOutcome {
        let min_closed = Self::min_closed_for_interval(interval);
        let max_len = (bars + 12).max(bars) as usize;
        let mut plan: Vec<(String, DateTime<Utc>)> = Vec::new();
        for sym in symbols {
            let Some(fetch_start) = Self::interval_fetch_start(
                cache.get(sym),
                interval,
                unit_hours,
                bars,
                min_closed,
                now,
                last_refresh,
            ) else {
                continue;
            };
            plan.push((sym.clone(), fetch_start));
        }

        let fetched = plan.len();
        let mut failures: Vec<String> = Vec::new();
        let mut fetched_rows: Vec<(String, Vec<Candle>)> = Vec::new();
        thread::scope(|scope| {
            let mut handles = Vec::with_capacity(plan.len());
            for (sym, fetch_start) in plan {
                let client = client.clone();
                let interval = interval.to_string();
                handles.push(scope.spawn(move || {
                    let result = client.fetch_klines(&sym, &interval, fetch_start, now);
                    (sym, interval, result)
                }));
            }
            for handle in handles {
                let (sym, interval, result) = handle.join().expect("kline fetch thread panicked");
                match result {
                    Ok(rows) if !rows.is_empty() => fetched_rows.push((sym, rows)),
                    Ok(_) => failures.push(format!("{sym} {interval}: fetch returned 0 candles")),
                    Err(e) => failures.push(format!("{sym} {interval}: fetch failed: {e}")),
                }
            }
        });
        let rows = fetched_rows.iter().map(|(_, rows)| rows.len()).sum();
        for (sym, rows) in fetched_rows {
            let entry = cache.entry(sym).or_default();
            Self::merge_candles(entry, rows, max_len);
        }
        RefreshOutcome {
            fetched,
            rows,
            failures,
        }
    }

    fn refresh_funding_cache(
        &mut self,
        client: &Arc<dyn LiveMarketClient>,
        now: DateTime<Utc>,
    ) -> RefreshOutcome {
        let mut plan: Vec<(String, DateTime<Utc>)> = Vec::new();
        for sym in &self.symbols {
            match self.funding.get(sym).and_then(|rates| rates.last()) {
                Some(last) if self.funding.get(sym).map_or(0, Vec::len) >= 3 => {
                    // Binance USD-M funding posts every 8h. Between funding
                    // timestamps, the existing cached value is exactly the
                    // value context_at() should see.
                    if now - last.timestamp < Duration::hours(8) {
                        continue;
                    }
                    plan.push((sym.clone(), last.timestamp + Duration::milliseconds(1)));
                }
                _ => plan.push((sym.clone(), now - Duration::days(FUNDING_LOOKBACK_DAYS))),
            }
        }

        let fetched = plan.len();
        let mut failures: Vec<String> = Vec::new();
        let mut fetched_rows: Vec<(String, Vec<FundingRate>)> = Vec::new();
        thread::scope(|scope| {
            let mut handles = Vec::with_capacity(plan.len());
            for (sym, start) in plan {
                let client = client.clone();
                handles.push(scope.spawn(move || {
                    let result = client.fetch_funding_rates(&sym, start, now);
                    (sym, result)
                }));
            }
            for handle in handles {
                let (sym, result) = handle.join().expect("funding fetch thread panicked");
                match result {
                    Ok(rows) => fetched_rows.push((sym, rows)),
                    Err(e) => failures.push(format!("{sym} funding fetch failed: {e}")),
                }
            }
        });
        let rows = fetched_rows.iter().map(|(_, rows)| rows.len()).sum();
        for (sym, rows) in fetched_rows {
            if !rows.is_empty() {
                let entry = self.funding.entry(sym).or_default();
                Self::merge_funding(entry, rows, now);
            }
        }

        let missing: Vec<String> = self
            .symbols
            .iter()
            .filter(|sym| self.funding.get(*sym).map_or(0, Vec::len) < 3)
            .cloned()
            .collect();
        if !missing.is_empty() {
            let preview: Vec<String> = missing.iter().take(3).cloned().collect();
            failures.push(format!(
                "opus46: required funding context missing for {}/{} symbols (e.g. {})",
                missing.len(),
                self.symbols.len(),
                preview.join(","),
            ));
        }
        RefreshOutcome {
            fetched,
            rows,
            failures,
        }
    }

    fn refresh_btc_events_if_needed(
        &mut self,
        client: &Arc<dyn LiveMarketClient>,
        now: DateTime<Utc>,
    ) -> Result<bool, SignalError> {
        if let Some((latest_visible, _)) = self.btc_events.iter().rev().find(|(ts, _)| *ts <= now) {
            if now <= *latest_visible + Duration::days(1) {
                return Ok(false);
            }
        }
        self.btc_events = Self::try_build_btc_events(client, now)?;
        Ok(true)
    }

    fn validate_1h_cache(&self, now: DateTime<Utc>) -> Vec<String> {
        let min_closed = Self::warmup_1h_bars() as usize;
        let mut failures = Vec::new();
        for sym in &self.symbols {
            match self.candles_1h.get(sym) {
                Some(cs) => {
                    if let Err(e) = Self::validate_indicator_warmup(
                        sym,
                        "1h",
                        cs,
                        INDICATOR_COLUMNS_1H,
                        min_closed,
                        now,
                    ) {
                        failures.push(e);
                    }
                }
                None => failures.push(format!("{sym} 1h: no warmup cache")),
            }
        }
        failures
    }

    fn validate_4h_cache(&self, now: DateTime<Utc>) -> Vec<String> {
        let min_closed = Self::warmup_4h_bars() as usize;
        let mut failures = Vec::new();
        for sym in &self.symbols {
            match self.candles_4h.get(sym) {
                Some(cs) => {
                    if let Err(e) = Self::validate_indicator_warmup(
                        sym,
                        "4h",
                        cs,
                        INDICATOR_COLUMNS_4H,
                        min_closed,
                        now,
                    ) {
                        failures.push(e);
                    }
                }
                None => failures.push(format!("{sym} 4h: no warmup cache")),
            }
        }
        failures
    }

    fn build_htf_4h_from_cache(&self, end: DateTime<Utc>) -> HtfData {
        let mut candles_4h: HashMap<String, Vec<Candle>> = HashMap::new();
        let mut indicators_4h: HashMap<String, HashMap<String, Vec<f64>>> = HashMap::new();
        for (sym, cs) in &self.candles_4h {
            let end_idx = cs.partition_point(|c| c.close_time < end);
            let closed = &cs[..end_idx];
            if closed.is_empty() {
                continue;
            }
            let ohlcv = OhlcvFrame {
                open: closed.iter().map(|c| c.open).collect(),
                high: closed.iter().map(|c| c.high).collect(),
                low: closed.iter().map(|c| c.low).collect(),
                close: closed.iter().map(|c| c.close).collect(),
                volume: closed.iter().map(|c| c.volume).collect(),
                taker_buy_volume: closed.iter().map(|c| c.taker_buy_volume).collect(),
            };
            match compute_indicators(&ohlcv, INDICATOR_COLUMNS_4H) {
                Ok(ind) => {
                    candles_4h.insert(sym.clone(), closed.to_vec());
                    indicators_4h.insert(sym.clone(), ind);
                }
                Err(e) => log::warn!("opus46: {sym} 4h indicators failed: {e}"),
            }
        }
        let mut htf = HtfData::default();
        htf.additional_candles.insert("4h".to_string(), candles_4h);
        htf.additional_indicators
            .insert("4h".to_string(), indicators_4h);
        htf
    }

    fn refresh_market_state(&mut self, now: DateTime<Utc>) -> Result<(), SignalError> {
        let client = self.client()?.clone();
        let t = Instant::now();

        // Eagerly attempt every source before returning an error. That keeps
        // the operator log complete for the poll, but unchanged daily/funding/
        // 4h sources are skipped so the boundary path only fetches what can
        // have changed.
        let btc_t = Instant::now();
        let btc_refreshed = self.refresh_btc_events_if_needed(&client, now);
        let btc_ms = btc_t.elapsed().as_secs_f64() * 1000.0;

        let funding_t = Instant::now();
        let funding_outcome = self.refresh_funding_cache(&client, now);
        let funding_ms = funding_t.elapsed().as_secs_f64() * 1000.0;

        let oneh_t = Instant::now();
        let oneh_failures = Self::refresh_interval_cache(
            &client,
            &self.symbols,
            &mut self.candles_1h,
            "1h",
            1,
            Self::warmup_1h_bars(),
            now,
            self.last_refresh,
        );
        let oneh_ms = oneh_t.elapsed().as_secs_f64() * 1000.0;

        let fourh_t = Instant::now();
        let fourh_failures = Self::refresh_interval_cache(
            &client,
            &self.symbols,
            &mut self.candles_4h,
            "4h",
            4,
            Self::warmup_4h_bars(),
            now,
            self.last_refresh,
        );
        let fourh_ms = fourh_t.elapsed().as_secs_f64() * 1000.0;

        let validate_t = Instant::now();
        let mut failures = Vec::new();
        let btc_refreshed_ok = matches!(&btc_refreshed, Ok(true));
        match btc_refreshed {
            Ok(_) => {}
            Err(e) => failures.push(e.message().to_string()),
        }
        failures.extend(funding_outcome.failures);
        failures.extend(oneh_failures.failures);
        failures.extend(fourh_failures.failures);
        failures.extend(self.validate_1h_cache(now));
        failures.extend(self.validate_4h_cache(now));
        let validate_ms = validate_t.elapsed().as_secs_f64() * 1000.0;

        if !failures.is_empty() {
            let preview = failures
                .iter()
                .take(6)
                .cloned()
                .collect::<Vec<_>>()
                .join("; ");
            return Err(SignalError::recoverable(format!(
                "opus46 market warmup incomplete for poll {} UTC: {} issue(s): {}",
                now.format("%Y-%m-%d %H:%M:%S"),
                failures.len(),
                preview,
            )));
        }

        self.last_refresh = Some(now);
        let funding_rows: usize = self.funding.values().map(|v| v.len()).sum();
        eprintln!(
            "opus46 data ready at {} UTC | 1h={}/{} fetched={} rows={} 4h={}/{} fetched={} rows={} funding={} rows/{} syms fetched={} rows={} btc_events={} btc_refreshed={} timings_ms btc={:.0} funding={:.0} 1h={:.0} 4h={:.0} validate={:.0} total={:.0}",
            now.format("%H:%M:%S"),
            self.candles_1h.len(),
            self.symbols.len(),
            oneh_failures.fetched,
            oneh_failures.rows,
            self.candles_4h.len(),
            self.symbols.len(),
            fourh_failures.fetched,
            fourh_failures.rows,
            funding_rows,
            self.funding.len(),
            funding_outcome.fetched,
            funding_outcome.rows,
            self.btc_events.len(),
            btc_refreshed_ok,
            btc_ms,
            funding_ms,
            oneh_ms,
            fourh_ms,
            validate_ms,
            t.elapsed().as_secs_f64() * 1000.0,
        );
        Ok(())
    }

    fn cache_status(&self, now: DateTime<Utc>) -> (usize, usize, usize, usize) {
        let min_1h = Self::warmup_1h_bars() as usize;
        let min_4h = Self::warmup_4h_bars() as usize;
        let oneh_ready = self
            .symbols
            .iter()
            .filter(|sym| {
                self.candles_1h
                    .get(*sym)
                    .map(|cs| Self::closed_count(cs, now) >= min_1h)
                    .unwrap_or(false)
            })
            .count();
        let fourh_ready = self
            .symbols
            .iter()
            .filter(|sym| {
                self.candles_4h
                    .get(*sym)
                    .map(|cs| Self::closed_count(cs, now) >= min_4h)
                    .unwrap_or(false)
            })
            .count();
        let funding_ready = self.funding.len();
        let btc_events = self.btc_events.len();
        (oneh_ready, fourh_ready, funding_ready, btc_events)
    }
}

impl Default for Opus46Live {
    fn default() -> Self {
        Self::new()
    }
}

fn btc_listing_start() -> DateTime<Utc> {
    let (year, month, day) = BTC_LISTING_START_YMD;
    Utc.with_ymd_and_hms(year, month, day, 0, 0, 0)
        .single()
        .expect("valid BTC listing start")
}

/// Synthesize 1d candles from a sorted slice of 1h candles by grouping on
/// `close_time.date_naive()`. Mirrors `research_runtime::synthesize_daily_candles`
/// — kept inline here to avoid widening the research_runtime public surface
/// for one helper.
fn synthesize_daily_from_hourly(hourly: &[Candle]) -> Vec<Candle> {
    use std::collections::BTreeMap;
    let mut by_day: BTreeMap<chrono::NaiveDate, Vec<&Candle>> = BTreeMap::new();
    for c in hourly {
        by_day.entry(c.close_time.date_naive()).or_default().push(c);
    }
    by_day
        .into_iter()
        .map(|(_date, bars)| {
            let open = bars.first().unwrap().open;
            let close = bars.last().unwrap().close;
            let high = bars
                .iter()
                .map(|b| b.high)
                .fold(f64::NEG_INFINITY, f64::max);
            let low = bars.iter().map(|b| b.low).fold(f64::INFINITY, f64::min);
            let volume: f64 = bars.iter().map(|b| b.volume).sum();
            let taker_buy_volume: f64 = bars.iter().map(|b| b.taker_buy_volume).sum();
            Candle {
                open_time: bars.first().unwrap().open_time,
                close_time: bars.last().unwrap().close_time,
                open,
                high,
                low,
                close,
                volume,
                taker_buy_volume,
            }
        })
        .collect()
}

impl LiveSignalGenerator for Opus46Live {
    fn strategy_id(&self) -> &str {
        STRATEGY_ID
    }

    fn symbols(&self) -> &[String] {
        &self.symbols
    }

    fn analysis_interval(&self) -> &str {
        ANALYSIS_INTERVAL
    }

    fn leverage(&self) -> f64 {
        1.0
    }

    fn setup(&mut self, client: Arc<dyn LiveMarketClient>) -> Result<(), SignalError> {
        let now = Utc::now();
        self.market_client = Some(client);
        eprintln!(
            "opus46 setup warmup starting | symbols={} 1h_bars={} 4h_bars={} funding_lookback={}d btc_start={}",
            self.symbols.len(),
            Self::warmup_1h_bars(),
            Self::warmup_4h_bars(),
            FUNDING_LOOKBACK_DAYS,
            btc_listing_start().format("%Y-%m-%d"),
        );
        self.refresh_market_state(now).map_err(|e| {
            SignalError::fatal(format!("opus46 setup warmup failed: {}", e.message()))
        })?;
        for sym in &self.symbols {
            let oneh = self
                .candles_1h
                .get(sym)
                .map(|cs| Self::closed_count(cs, now))
                .unwrap_or(0);
            let fourh = self
                .candles_4h
                .get(sym)
                .map(|cs| Self::closed_count(cs, now))
                .unwrap_or(0);
            eprintln!("  {sym}: warmed 1h={oneh} closed bars 4h={fourh} closed bars");
        }
        eprintln!(
            "opus46 setup OK | strategy_id={} btc_events={} funding_symbols={}",
            STRATEGY_ID,
            self.btc_events.len(),
            self.funding.len(),
        );
        Ok(())
    }

    fn set_poll_time(&mut self, now: DateTime<Utc>) {
        self.poll_time = now;
    }

    fn prepare_poll(&mut self, next_boundary: DateTime<Utc>) -> Result<(), SignalError> {
        let status_time = self
            .last_refresh
            .unwrap_or(next_boundary - Duration::milliseconds(1));
        let (oneh_ready, fourh_ready, funding_ready, btc_events) = self.cache_status(status_time);
        let last_refresh = self
            .last_refresh
            .map(|t| t.format("%H:%M:%S").to_string())
            .unwrap_or_else(|| "never".to_string());
        eprintln!(
            "opus46 prepoll cache | boundary={} UTC last_refresh={} 1h_history={}/{} 4h_history={}/{} funding={}/{} btc_events={} final_bar_refresh=signal_poll",
            next_boundary.format("%H:%M:%S"),
            last_refresh,
            oneh_ready,
            self.symbols.len(),
            fourh_ready,
            self.symbols.len(),
            funding_ready,
            self.symbols.len(),
            btc_events,
        );
        if oneh_ready == self.symbols.len()
            && fourh_ready == self.symbols.len()
            && funding_ready == self.symbols.len()
            && btc_events > 0
        {
            Ok(())
        } else {
            Err(SignalError::recoverable(
                "opus46 prepoll cache is incomplete; signal poll will refresh and fail closed if still incomplete",
            ))
        }
    }

    fn poll(&mut self) -> Result<Vec<Signal>, SignalError> {
        let now = self.poll_time;
        self.refresh_market_state(now)?;

        if self.candles_1h.is_empty() {
            return Err(SignalError::recoverable(
                "opus46: no 1h candles for any symbol this poll",
            ));
        }

        // Phase 1 — assemble ContextMap from the warmed live caches.
        let mut entries: Vec<(ContextKey, SeriesInput)> =
            Vec::with_capacity(1 + self.funding.len());
        entries.push((
            ContextKey::BtcStructure,
            SeriesInput::Point(self.btc_events.clone()),
        ));
        for (sym, rates) in self.funding.clone() {
            entries.push((ContextKey::Funding(sym), SeriesInput::FundingRaw(rates)));
        }
        let ctx = ContextMap::from_series(entries);

        // Phase 2 — pick the latest fully-closed 1h bar across symbols and
        // run the research strategy's generate_signals() against it.
        let latest_close = match self
            .candles_1h
            .values()
            .filter_map(|cs| cs.iter().rev().find(|c| c.close_time <= now))
            .map(|c| c.close_time)
            .max()
        {
            Some(t) => t,
            None => {
                return Err(SignalError::recoverable(
                    "opus46: no closed 1h bar found at poll time",
                ));
            }
        };
        let start = latest_close;
        let end = latest_close + Duration::milliseconds(1);
        let missing_decision_bar: Vec<String> = self
            .symbols
            .iter()
            .filter(|sym| {
                !self
                    .candles_1h
                    .get(*sym)
                    .map(|cs| cs.iter().any(|c| c.close_time == latest_close))
                    .unwrap_or(false)
            })
            .cloned()
            .collect();
        if !missing_decision_bar.is_empty() {
            return Err(SignalError::recoverable(format!(
                "opus46: latest 1h decision bar {} UTC missing for {}/{} symbols (e.g. {})",
                latest_close.format("%Y-%m-%d %H:%M:%S"),
                missing_decision_bar.len(),
                self.symbols.len(),
                missing_decision_bar
                    .iter()
                    .take(3)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(","),
            )));
        }
        let htf = self.build_htf_4h_from_cache(end);

        let mut tree: BTreeMap<String, &[Candle]> = BTreeMap::new();
        for (sym, cs) in &self.candles_1h {
            let end_idx = cs.partition_point(|c| c.close_time < end);
            tree.insert(sym.clone(), &cs[..end_idx]);
        }
        let active_params: HashMap<String, serde_json::Value> = HashMap::new();
        let generation_t = Instant::now();
        let raw = self
            .strategy
            .generate_signals(&tree, start, end, &active_params, &ctx, &htf);
        let generation_ms = generation_t.elapsed().as_secs_f64() * 1000.0;
        eprintln!(
            "opus46 generated {} raw signal(s) for closed bar {} UTC | generation_ms={:.0}",
            raw.len(),
            latest_close.format("%Y-%m-%d %H:%M:%S"),
            generation_ms,
        );

        self.commit_signals(raw)
    }
}

impl Opus46Live {
    /// Validate each signal, gate on cooldown, then persist+emit atomically.
    /// Extracted from `poll()` so a regression test can drive the exact
    /// fail-closed contract without engineering a 22-symbol synthetic OHLCV
    /// dataset that produces real opus46 entries.
    ///
    /// Contract:
    ///   - Signals failing `validate()` are dropped with a warn log.
    ///   - Signals where `cooldown.is_blocked(spec, signal_date)` are dropped.
    ///   - The remaining signals are committed via the
    ///     `clone-store → record → save → swap-store` sequence so that a
    ///     save failure leaves `self.cooldown` untouched and returns
    ///     `SignalError::Fatal`. This prevents a restart from re-emitting
    ///     within the 6h cooldown window.
    fn commit_signals(&mut self, raw: Vec<Signal>) -> Result<Vec<Signal>, SignalError> {
        let mut proposed: Vec<(Signal, claude_trader_models::CooldownSpec)> =
            Vec::with_capacity(raw.len());
        for sig in raw {
            if let Err(e) = sig.validate() {
                log::warn!(
                    "opus46: dropping invalid signal for {} pattern={}: {}",
                    sig.ticker,
                    sig.pattern,
                    e
                );
                continue;
            }
            let spec = self.strategy.cooldown_spec(&sig);
            if self.cooldown.is_blocked(&spec, sig.signal_date) {
                log::debug!(
                    "opus46: cooldown skip {} {} pattern={} signal_date={}",
                    sig.ticker,
                    sig.position_type.as_str(),
                    sig.pattern,
                    sig.signal_date,
                );
                continue;
            }
            proposed.push((sig, spec));
        }

        if proposed.is_empty() {
            return Ok(Vec::new());
        }

        let mut candidate = self.cooldown.clone();
        for (sig, spec) in &proposed {
            candidate.record(spec, sig.signal_date);
        }
        if let Err(e) = candidate.save(&self.cooldown_path) {
            return Err(SignalError::fatal(format!(
                "opus46: cooldown save to {} failed: {e}; refusing to emit \
                 because a restart could re-emit within the 6h cooldown",
                self.cooldown_path.display(),
            )));
        }
        self.cooldown = candidate;
        Ok(proposed.into_iter().map(|(s, _)| s).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use claude_trader_live::error::{LiveError, Result as LiveResult};
    use std::sync::Mutex;

    /// Returns empty candle vecs for any request — exercises the
    /// "no-data, no-signals" path without hitting the network.
    struct EmptyClient;

    impl LiveMarketClient for EmptyClient {
        fn fetch_klines(
            &self,
            _: &str,
            _: &str,
            _: DateTime<Utc>,
            _: DateTime<Utc>,
        ) -> LiveResult<Vec<Candle>> {
            Ok(Vec::new())
        }
    }

    /// Records each fetch_klines and fetch_funding_rates request and
    /// returns one trivial candle / empty funding so the poll proceeds
    /// past data guards. The single candle is intentionally below any
    /// indicator warmup, so `generate_signals` simply skips the symbol —
    /// which is what we want for an interval-tracking test.
    struct RecordingClient {
        kline_calls: Mutex<Vec<(String, String)>>,
        funding_calls: Mutex<Vec<String>>,
    }

    impl RecordingClient {
        fn new() -> Self {
            Self {
                kline_calls: Mutex::new(Vec::new()),
                funding_calls: Mutex::new(Vec::new()),
            }
        }
        fn kline_calls(&self) -> Vec<(String, String)> {
            self.kline_calls.lock().unwrap().clone()
        }
        fn funding_calls(&self) -> Vec<String> {
            self.funding_calls.lock().unwrap().clone()
        }
    }

    impl LiveMarketClient for RecordingClient {
        fn fetch_klines(
            &self,
            symbol: &str,
            interval: &str,
            _: DateTime<Utc>,
            _: DateTime<Utc>,
        ) -> LiveResult<Vec<Candle>> {
            self.kline_calls
                .lock()
                .unwrap()
                .push((symbol.to_string(), interval.to_string()));
            let now = Utc::now();
            Ok(vec![Candle {
                open_time: now - Duration::hours(1),
                close_time: now - Duration::milliseconds(1),
                open: 100.0,
                high: 100.0,
                low: 100.0,
                close: 100.0,
                volume: 1.0,
                taker_buy_volume: 0.5,
            }])
        }

        fn fetch_funding_rates(
            &self,
            symbol: &str,
            _: DateTime<Utc>,
            _: DateTime<Utc>,
        ) -> LiveResult<Vec<FundingRate>> {
            self.funding_calls.lock().unwrap().push(symbol.to_string());
            Ok(Vec::new())
        }
    }

    /// A client that always errors — used to assert setup() returns Fatal
    /// when the probe fetch fails.
    struct FailingClient;

    impl LiveMarketClient for FailingClient {
        fn fetch_klines(
            &self,
            _: &str,
            _: &str,
            _: DateTime<Utc>,
            _: DateTime<Utc>,
        ) -> LiveResult<Vec<Candle>> {
            Err(LiveError::Http("simulated".into()))
        }
    }

    #[test]
    fn declares_21_symbols_starting_with_btc_and_excludes_delisted() {
        let s = Opus46Live::new();
        assert_eq!(s.symbols().len(), 21);
        assert_eq!(s.symbols()[0], "BTCUSDT");
        assert_eq!(s.strategy_id(), STRATEGY_ID);
        assert_eq!(s.analysis_interval(), "1h");
    }

    /// Guard against silently re-introducing a delisted symbol. MATICUSDT
    /// was removed from Binance USD-M after the Polygon→POL migration; if
    /// anyone needs POLUSDT they MUST re-backtest the strategy first.
    #[test]
    fn does_not_declare_delisted_maticusdt() {
        let s = Opus46Live::new();
        assert!(
            !s.symbols().iter().any(|sym| sym == "MATICUSDT"),
            "MATICUSDT was delisted from Binance USD-M; live universe must not include it"
        );
    }

    #[test]
    fn setup_failure_is_fatal() {
        let mut s = Opus46Live::new();
        let err = s.setup(Arc::new(FailingClient)).unwrap_err();
        assert!(err.is_fatal(), "expected fatal, got {err:?}");
    }

    #[test]
    fn setup_with_empty_probe_is_fatal() {
        let mut s = Opus46Live::new();
        let err = s.setup(Arc::new(EmptyClient)).unwrap_err();
        assert!(err.is_fatal());
    }

    #[test]
    fn poll_with_no_data_is_recoverable_not_fatal() {
        let mut s = Opus46Live::new();
        s.market_client = Some(Arc::new(EmptyClient));
        s.poll_time = Utc::now();
        let err = s.poll().unwrap_err();
        assert!(
            !err.is_fatal(),
            "poll() should be recoverable on empty data, got fatal: {err:?}"
        );
    }

    fn isolated_cooldown_path() -> PathBuf {
        let pid = std::process::id();
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("opus46_live_test_{pid}_{nonce}"));
        std::fs::create_dir_all(&dir).unwrap();
        dir.join("cd.json")
    }

    #[test]
    fn cooldown_persists_across_recreate() {
        use claude_trader_models::CooldownSpec;
        use claude_trader_models::PositionType;

        let path = isolated_cooldown_path();
        let signal_date = Utc::now();

        // First "process": record an emit.
        {
            let mut s = Opus46Live::new();
            s.set_cooldown_path(path.clone());
            // Mimic what poll() does after generate_signals: build the spec
            // off a stand-in Signal and record + save.
            let stand_in = test_signal("BTCUSDT", PositionType::Short, signal_date);
            let spec: CooldownSpec = s.strategy.cooldown_spec(&stand_in);
            s.cooldown.record(&spec, signal_date);
            s.cooldown.save(&path).unwrap();
        }

        // Second "process": fresh struct loads from disk, same key blocks.
        {
            let mut s = Opus46Live::new();
            s.set_cooldown_path(path.clone());
            let stand_in = test_signal("BTCUSDT", PositionType::Short, signal_date);
            let spec = s.strategy.cooldown_spec(&stand_in);
            assert!(s
                .cooldown
                .is_blocked(&spec, signal_date + Duration::hours(2)));
            // After 6h elapses, it unblocks.
            assert!(!s
                .cooldown
                .is_blocked(&spec, signal_date + Duration::hours(7)));
        }

        let _ = std::fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn cooldown_distinguishes_long_and_short_sides() {
        use claude_trader_models::PositionType;
        let path = isolated_cooldown_path();
        let mut s = Opus46Live::new();
        s.set_cooldown_path(path.clone());
        let now = Utc::now();
        let short_sig = test_signal("BTCUSDT", PositionType::Short, now);
        let long_sig = test_signal("BTCUSDT", PositionType::Long, now);
        let short_spec = s.strategy.cooldown_spec(&short_sig);
        let long_spec = s.strategy.cooldown_spec(&long_sig);
        s.cooldown.record(&short_spec, now);
        assert!(s.cooldown.is_blocked(&short_spec, now + Duration::hours(1)));
        assert!(!s.cooldown.is_blocked(&long_spec, now + Duration::hours(1)));
        let _ = std::fs::remove_dir_all(path.parent().unwrap());
    }

    fn test_signal(
        ticker: &str,
        side: claude_trader_models::PositionType,
        signal_date: DateTime<Utc>,
    ) -> Signal {
        Signal {
            signal_date,
            position_type: side,
            ticker: ticker.to_string(),
            pattern: "release_short_1h".to_string(),
            tp_pct: Some(3.0),
            sl_pct: Some(1.5),
            tp_price: None,
            sl_price: None,
            leverage: 1.0,
            market_type: claude_trader_models::MarketType::Futures,
            taker_fee_rate: 0.0005,
            entry_price: None,
            fill_timeout_seconds: 3600,
            entry_delay_seconds: None,
            max_holding_hours: 60,
            size_multiplier: 1.0,
            metadata: HashMap::new(),
        }
    }

    fn ts(rfc3339: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(rfc3339)
            .unwrap()
            .with_timezone(&Utc)
    }

    fn test_candle(open_time: DateTime<Utc>, close_time: DateTime<Utc>) -> Candle {
        Candle {
            open_time,
            close_time,
            open: 100.0,
            high: 101.0,
            low: 99.0,
            close: 100.5,
            volume: 1.0,
            taker_buy_volume: 0.5,
        }
    }

    #[test]
    fn interval_fetch_start_fetches_missing_latest_closed_bar_only() {
        let now = ts("2026-05-01T23:00:00Z");
        let cached = vec![test_candle(
            ts("2026-05-01T21:00:00Z"),
            ts("2026-05-01T21:59:59.999Z"),
        )];

        let start = Opus46Live::interval_fetch_start(
            Some(&cached),
            "1h",
            1,
            200,
            1,
            now,
            Some(ts("2026-05-01T22:00:05Z")),
        )
        .unwrap();

        assert_eq!(start, ts("2026-05-01T22:00:00Z"));
    }

    #[test]
    fn interval_fetch_start_refetches_cached_partial_after_it_closes() {
        let now = ts("2026-05-01T22:00:00Z");
        let cached = vec![test_candle(
            ts("2026-05-01T21:00:00Z"),
            ts("2026-05-01T21:59:59.999Z"),
        )];

        let start = Opus46Live::interval_fetch_start(
            Some(&cached),
            "1h",
            1,
            200,
            1,
            now,
            Some(ts("2026-05-01T21:06:07Z")),
        )
        .unwrap();

        assert_eq!(start, ts("2026-05-01T21:00:00Z"));
    }

    #[test]
    fn interval_fetch_start_skips_unchanged_4h_cache() {
        let now = ts("2026-05-01T23:00:00Z");
        let cached = vec![test_candle(
            ts("2026-05-01T16:00:00Z"),
            ts("2026-05-01T19:59:59.999Z"),
        )];

        let start = Opus46Live::interval_fetch_start(
            Some(&cached),
            "4h",
            4,
            100,
            1,
            now,
            Some(ts("2026-05-01T20:00:05Z")),
        );

        assert_eq!(start, None);
    }

    #[test]
    fn poll_requests_btc_daily_plus_1h_and_4h_per_symbol() {
        let client = Arc::new(RecordingClient::new());
        let mut s = Opus46Live::new();
        s.set_cooldown_path(isolated_cooldown_path());
        s.market_client = Some(client.clone());
        s.poll_time = Utc::now();
        let _ = s.poll();
        let calls = client.kline_calls();
        let intervals: std::collections::HashSet<&str> =
            calls.iter().map(|(_, i)| i.as_str()).collect();
        assert!(intervals.contains("1d"), "BTC daily fetch missing");
        assert!(intervals.contains("1h"), "1h fetch missing");
        assert!(intervals.contains("4h"), "4h fetch missing");
        let oneh_symbols: std::collections::HashSet<&str> = calls
            .iter()
            .filter(|(_, i)| i == "1h")
            .map(|(s, _)| s.as_str())
            .collect();
        assert_eq!(
            oneh_symbols.len(),
            21,
            "expected 1h fetch for every declared symbol"
        );
    }

    /// Returns 1d empty but 1h non-empty so we can verify the synthesis
    /// fallback executes when the 1d endpoint fails.
    struct OnlyHourlyClient {
        kline_calls: Mutex<Vec<(String, String)>>,
    }
    impl OnlyHourlyClient {
        fn new() -> Self {
            Self {
                kline_calls: Mutex::new(Vec::new()),
            }
        }
        fn calls(&self) -> Vec<(String, String)> {
            self.kline_calls.lock().unwrap().clone()
        }
    }
    impl LiveMarketClient for OnlyHourlyClient {
        fn fetch_klines(
            &self,
            symbol: &str,
            interval: &str,
            _: DateTime<Utc>,
            _: DateTime<Utc>,
        ) -> LiveResult<Vec<Candle>> {
            self.kline_calls
                .lock()
                .unwrap()
                .push((symbol.to_string(), interval.to_string()));
            if interval == "1d" {
                Ok(Vec::new())
            } else {
                let now = Utc::now();
                Ok(vec![Candle {
                    open_time: now - Duration::hours(1),
                    close_time: now - Duration::milliseconds(1),
                    open: 100.0,
                    high: 100.0,
                    low: 100.0,
                    close: 100.0,
                    volume: 1.0,
                    taker_buy_volume: 0.5,
                }])
            }
        }
    }

    #[test]
    fn empty_btc_1d_falls_back_to_1h_synthesis() {
        let client = Arc::new(OnlyHourlyClient::new());
        let mut s = Opus46Live::new();
        s.set_cooldown_path(isolated_cooldown_path());
        s.market_client = Some(client.clone());
        s.poll_time = Utc::now();
        let _ = s.poll();
        let calls = client.calls();
        let has_btc_1d = calls.iter().any(|(sym, i)| sym == "BTCUSDT" && i == "1d");
        let has_btc_1h = calls.iter().any(|(sym, i)| sym == "BTCUSDT" && i == "1h");
        assert!(has_btc_1d, "must attempt 1d before fallback");
        assert!(
            has_btc_1h,
            "must attempt 1h synthesis fallback when 1d empty"
        );
    }

    /// Returns klines but errors on fetch_funding_rates — emulates a
    /// transient funding-endpoint outage.
    struct FundingErrorClient;
    impl LiveMarketClient for FundingErrorClient {
        fn fetch_klines(
            &self,
            _: &str,
            _: &str,
            _: DateTime<Utc>,
            _: DateTime<Utc>,
        ) -> LiveResult<Vec<Candle>> {
            let now = Utc::now();
            Ok(vec![Candle {
                open_time: now - Duration::hours(1),
                close_time: now - Duration::milliseconds(1),
                open: 100.0,
                high: 100.0,
                low: 100.0,
                close: 100.0,
                volume: 1.0,
                taker_buy_volume: 0.5,
            }])
        }
        fn fetch_funding_rates(
            &self,
            _: &str,
            _: DateTime<Utc>,
            _: DateTime<Utc>,
        ) -> LiveResult<Vec<FundingRate>> {
            Err(LiveError::Http("funding endpoint outage".into()))
        }
    }

    #[test]
    fn missing_funding_returns_recoverable_not_fatal() {
        let mut s = Opus46Live::new();
        s.set_cooldown_path(isolated_cooldown_path());
        s.market_client = Some(Arc::new(FundingErrorClient));
        s.poll_time = Utc::now();
        let err = s.poll().unwrap_err();
        assert!(
            !err.is_fatal(),
            "missing funding must be recoverable, got: {err:?}"
        );
        assert!(
            err.message().contains("funding"),
            "error should mention funding, got: {}",
            err.message()
        );
    }

    /// Drives the production code path (`commit_signals`) end-to-end. The
    /// alternative — engineering a 22-symbol synthetic OHLCV dataset that
    /// produces a real opus46 entry — is too brittle. Instead we hand
    /// commit_signals a hand-crafted Signal that survives `validate()`
    /// and the cooldown gate, force the save to fail by pointing
    /// cooldown_path at a write-rejecting location, and assert the exact
    /// contract: Fatal returned, in-memory state unchanged.
    #[test]
    fn commit_signals_save_failure_is_fatal_and_does_not_mutate_state() {
        use claude_trader_models::PositionType;

        let mut s = Opus46Live::new();
        let bad_path = PathBuf::from("/proc/self/cooldown_unwritable.json");
        s.set_cooldown_path(bad_path);

        // Sanity: confirm the path actually rejects writes — otherwise
        // the test would silently pass even after a regression.
        assert!(
            s.cooldown.save(&s.cooldown_path).is_err(),
            "save to /proc/self/... must fail; otherwise this test is moot",
        );

        let signal_date = Utc::now();
        let sig = test_signal("BTCUSDT", PositionType::Short, signal_date);
        let spec = s.strategy.cooldown_spec(&sig);

        let result = s.commit_signals(vec![sig]);

        let err = result.unwrap_err();
        assert!(
            err.is_fatal(),
            "save failure must escalate to Fatal, got: {err:?}"
        );
        assert!(
            err.message().contains("cooldown save"),
            "Fatal message must mention cooldown save, got: {}",
            err.message()
        );
        assert!(
            !s.cooldown
                .is_blocked(&spec, signal_date + Duration::hours(1)),
            "in-memory cooldown must remain unchanged when save fails"
        );
    }

    /// Happy-path counterpart to the fail test: with a writable cooldown
    /// path, commit_signals returns the signal AND advances in-memory state.
    #[test]
    fn commit_signals_happy_path_emits_and_records() {
        use claude_trader_models::PositionType;

        let mut s = Opus46Live::new();
        s.set_cooldown_path(isolated_cooldown_path());

        let signal_date = Utc::now();
        let sig = test_signal("BTCUSDT", PositionType::Short, signal_date);
        let spec = s.strategy.cooldown_spec(&sig);

        let out = s.commit_signals(vec![sig.clone()]).unwrap();
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].ticker, "BTCUSDT");
        assert!(s
            .cooldown
            .is_blocked(&spec, signal_date + Duration::hours(1)));

        // Second call within the cooldown window drops the signal.
        let out2 = s.commit_signals(vec![sig]).unwrap();
        assert!(out2.is_empty(), "in-window re-emit must be dropped");
    }

    #[test]
    fn poll_requests_funding_rate_per_symbol() {
        let client = Arc::new(RecordingClient::new());
        let mut s = Opus46Live::new();
        s.set_cooldown_path(isolated_cooldown_path());
        s.market_client = Some(client.clone());
        s.poll_time = Utc::now();
        let _ = s.poll();
        let funding_calls = client.funding_calls();
        let unique: std::collections::HashSet<&str> =
            funding_calls.iter().map(|s| s.as_str()).collect();
        assert_eq!(
            unique.len(),
            21,
            "expected funding fetch for every declared symbol, got {} unique calls",
            unique.len()
        );
    }
}
