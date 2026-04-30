//! Live adapter for the `opus46_max_16apr26_1` research strategy.
//!
//! The research strategy implements `ResearchStrategy::generate_signals` —
//! a batch operation over a prepared `BTreeMap<symbol, &[Candle]>` plus a
//! `ContextMap` and `HtfData`. This crate wraps that batch entry point in a
//! `LiveSignalGenerator` that:
//!
//!   1. Fetches fresh 1h candles for every declared symbol on each poll,
//!   2. Fetches fresh 4h candles + computes the strategy's 4h indicator set,
//!   3. Fetches BTC daily candles and runs `DailyStructureProvider` to
//!      derive `BtcStructure` market bias,
//!   4. Calls `generate_signals` with `start = latest_closed_bar.close_time`
//!      and `end = start + 1ms` so only the freshest closed bar can emit,
//!   5. Validates each signal and dedupes by `(ticker, signal_date)` so a
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

use std::path::PathBuf;

use chrono::{DateTime, Duration, Utc};
use claude_trader_btc_structure::{engine::FeatureValue, DailyStructureProvider};
use claude_trader_indicators::{compute_indicators, required_warmup, OhlcvFrame};
use claude_trader_live::market_client::LiveMarketClient;
use claude_trader_live::signal_generator::{LiveSignalGenerator, SignalError};
use claude_trader_models::{
    Candle, ContextKey, ContextMap, ContextValue, FundingRate, HtfData, MarketBias, SeriesInput,
    Signal,
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
const BTC_DAILY_LOOKBACK_DAYS: i64 = 200;
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
    cooldown: CooldownStore,
    cooldown_path: PathBuf,
}

impl Opus46Live {
    pub fn new() -> Self {
        let path = CooldownStore::default_path(STRATEGY_ID);
        Self {
            symbols: SYMBOLS.iter().map(|s| s.to_string()).collect(),
            strategy: Opus46Max16apr261,
            market_client: None,
            poll_time: DateTime::<Utc>::MIN_UTC,
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
        let start = now - Duration::days(BTC_DAILY_LOOKBACK_DAYS);
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

    /// Fetch funding rates for every declared symbol over the lookback
    /// window. Required-context: opus46's `required_context()` includes
    /// `Funding(sym)` for every traded symbol, and the research runtime
    /// would refuse to start without it. If any symbol returns sparse
    /// (<3 rates) or fails, return Recoverable so the engine skips this
    /// poll instead of trading with mismatched sizing.
    fn try_fetch_funding(
        &self,
        client: &Arc<dyn LiveMarketClient>,
        now: DateTime<Utc>,
    ) -> Result<HashMap<String, Vec<FundingRate>>, SignalError> {
        let start = now - Duration::days(FUNDING_LOOKBACK_DAYS);
        let mut out: HashMap<String, Vec<FundingRate>> = HashMap::new();
        let mut missing: Vec<String> = Vec::new();
        for sym in &self.symbols {
            match client.fetch_funding_rates(sym, start, now) {
                // funding_context_at requires ≥3 rates; below that the
                // FundingContext lookup can't produce a meaningful result.
                Ok(rates) if rates.len() >= 3 => {
                    out.insert(sym.clone(), rates);
                }
                Ok(rates) => {
                    log::warn!(
                        "opus46: {sym} funding has {} rates (<3 required)",
                        rates.len()
                    );
                    missing.push(sym.clone());
                }
                Err(e) => {
                    log::warn!("opus46: {sym} funding fetch failed: {}", e);
                    missing.push(sym.clone());
                }
            }
        }
        if !missing.is_empty() {
            let preview: Vec<String> = missing.iter().take(3).cloned().collect();
            return Err(SignalError::recoverable(format!(
                "opus46: required funding context missing for {}/{} symbols (e.g. {})",
                missing.len(),
                self.symbols.len(),
                preview.join(","),
            )));
        }
        Ok(out)
    }

    fn fetch_with_warmup(
        client: &Arc<dyn LiveMarketClient>,
        symbol: &str,
        interval: &str,
        bars: i64,
        unit_hours: i64,
        now: DateTime<Utc>,
    ) -> Result<Vec<Candle>, SignalError> {
        let start = now - Duration::hours(bars * unit_hours);
        client
            .fetch_klines(symbol, interval, start, now)
            .map_err(|e| SignalError::recoverable(format!("{symbol} {interval} fetch failed: {e}")))
    }

    fn build_htf_4h(&self, client: &Arc<dyn LiveMarketClient>, now: DateTime<Utc>) -> HtfData {
        let warmup = required_warmup(INDICATOR_COLUMNS_4H) as i64;
        let bars = warmup + EXTRA_4H_BARS;
        let mut candles_4h: HashMap<String, Vec<Candle>> = HashMap::new();
        let mut indicators_4h: HashMap<String, HashMap<String, Vec<f64>>> = HashMap::new();
        for sym in &self.symbols {
            let cs = match Self::fetch_with_warmup(client, sym, "4h", bars, 4, now) {
                Ok(c) => c,
                Err(e) => {
                    log::warn!("opus46: {sym} 4h fetch failed: {}", e.message());
                    continue;
                }
            };
            if cs.is_empty() {
                continue;
            }
            let ohlcv = OhlcvFrame {
                open: cs.iter().map(|c| c.open).collect(),
                high: cs.iter().map(|c| c.high).collect(),
                low: cs.iter().map(|c| c.low).collect(),
                close: cs.iter().map(|c| c.close).collect(),
                volume: cs.iter().map(|c| c.volume).collect(),
                taker_buy_volume: cs.iter().map(|c| c.taker_buy_volume).collect(),
            };
            match compute_indicators(&ohlcv, INDICATOR_COLUMNS_4H) {
                Ok(ind) => {
                    candles_4h.insert(sym.clone(), cs);
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
}

impl Default for Opus46Live {
    fn default() -> Self {
        Self::new()
    }
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
        // Smoke test: fetch BTC 1h to verify network + the symbol is tradable
        // before declaring the strategy ready. A failure here is fatal —
        // refusing to trade if we can't even reach the market data API.
        let probe_start = now - Duration::hours(2);
        let probe = client
            .fetch_klines("BTCUSDT", "1h", probe_start, now)
            .map_err(|e| {
                SignalError::fatal(format!("opus46 setup probe BTCUSDT 1h failed: {e}"))
            })?;
        if probe.is_empty() {
            return Err(SignalError::fatal("opus46 setup probe returned no candles"));
        }
        log::info!(
            "opus46 setup OK: BTCUSDT probe returned {} candle(s); strategy_id={}",
            probe.len(),
            STRATEGY_ID
        );
        self.market_client = Some(client);
        Ok(())
    }

    fn set_poll_time(&mut self, now: DateTime<Utc>) {
        self.poll_time = now;
    }

    fn poll(&mut self) -> Result<Vec<Signal>, SignalError> {
        let client = self.client()?.clone();
        let now = self.poll_time;

        // Phase 1 — eager fetch. Every required-context fetch issues now,
        // even if an earlier one already failed. Validation runs after all
        // fetches so transient errors don't mask each other and so
        // observability tooling sees a complete picture per poll.
        let btc_events_res = Self::try_build_btc_events(&client, now);
        let funding_res = self.try_fetch_funding(&client, now);

        let warmup_1h = required_warmup(INDICATOR_COLUMNS_1H) as i64;
        let bars_1h = warmup_1h + EXTRA_1H_BARS;
        let mut candles_1h: HashMap<String, Vec<Candle>> = HashMap::new();
        for sym in &self.symbols {
            match Self::fetch_with_warmup(&client, sym, "1h", bars_1h, 1, now) {
                Ok(cs) if !cs.is_empty() => {
                    candles_1h.insert(sym.clone(), cs);
                }
                Ok(_) => log::warn!("opus46: {sym} 1h fetch returned 0 candles"),
                Err(e) => log::warn!("opus46: {sym} 1h fetch failed: {}", e.message()),
            }
        }
        let htf = self.build_htf_4h(&client, now);

        // Phase 2 — validate required-context. Both BTC structure and
        // funding are declared in opus46's required_context(); a research
        // run that couldn't fetch them would refuse to start. The live
        // counterpart returns Recoverable so the engine logs and retries
        // on the next 1h boundary.
        let btc_events = btc_events_res?;
        let funding = funding_res?;
        if candles_1h.is_empty() {
            return Err(SignalError::recoverable(
                "opus46: no 1h candles for any symbol this poll",
            ));
        }

        // Phase 3 — assemble ContextMap.
        let mut entries: Vec<(ContextKey, SeriesInput)> = Vec::with_capacity(1 + funding.len());
        entries.push((ContextKey::BtcStructure, SeriesInput::Point(btc_events)));
        for (sym, rates) in funding {
            entries.push((ContextKey::Funding(sym), SeriesInput::FundingRaw(rates)));
        }
        let ctx = ContextMap::from_series(entries);

        // Phase 4 — pick the latest fully-closed 1h bar across symbols and
        // run the research strategy's generate_signals() against it.
        let latest_close = match candles_1h
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

        let mut tree: BTreeMap<String, &[Candle]> = BTreeMap::new();
        for (sym, cs) in &candles_1h {
            tree.insert(sym.clone(), cs.as_slice());
        }
        let active_params: HashMap<String, serde_json::Value> = HashMap::new();
        let raw = self
            .strategy
            .generate_signals(&tree, start, end, &active_params, &ctx, &htf);

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
