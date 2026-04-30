//! Shared research runtime — reusable evaluation pipeline for experiment crates.
//!
//! Each experiment crate implements `ResearchStrategy` and calls
//! `run_evaluation()`.  This is the only supported research/backtest
//! entrypoint.

pub mod context_builder;
pub mod key_levels;
pub mod metrics;
pub mod output;
pub mod validate;

use std::collections::{BTreeMap, HashMap};
use std::process;
use std::sync::Arc;
use std::time::Instant;

use chrono::{DateTime, Datelike, Duration, Utc};
use claude_trader_data::{BinanceClient, CandleStore, DataError, KlineFetch};
use claude_trader_evaluator::{
    build_evaluation_report, calendar, category_summaries, compute_overall_summary,
    cooldown::enforce_signal_cooldown, windows::group_into_periods,
};
use claude_trader_models::{
    Candle, CategorySummary, ContextKey, ContextMap, CooldownSpec, EvalWindow, EvaluationReport,
    FundingRate, HtfData, KeyLevels, MarketBias, MarketDataRequest, MarketType, PortfolioConfig,
    Signal, TradeResult,
};
use claude_trader_resolver::kernel::{self, MarketDataProvider};

use context_builder::{build_context, context_warmup};

use output::save_outputs;

use claude_trader_models::parse_interval_duration;

pub(crate) const COOLDOWN_WARMUP_DAYS: i64 = 14;
pub(crate) const DATA_BUFFER_HOURS: i64 = 168;
const DEFAULT_ENTRY_DELAY: i64 = 3;

// ---------------------------------------------------------------------------
// Request validation
// ---------------------------------------------------------------------------

/// Validate that a strategy's `market_data_request().ohlcv_interval` matches
/// its `analysis_interval()`. Returns `Err` on mismatch or invalid interval.
fn check_strategy_interval(strategy: &dyn ResearchStrategy) -> Result<(String, Duration), String> {
    let interval = strategy.analysis_interval().to_string();
    let duration = parse_interval_duration(&interval)
        .map_err(|e| format!("invalid analysis_interval {interval:?}: {e}"))?;
    let mdr = strategy.market_data_request();
    if mdr.ohlcv_interval != interval {
        return Err(format!(
            "analysis_interval() is {:?} but market_data_request().ohlcv_interval is {:?}. \
             These must match — either override market_data_request() to use the same interval, \
             or fix the mismatch.",
            interval, mdr.ohlcv_interval,
        ));
    }
    Ok((interval, duration))
}

/// Validate additional intervals: no 1m, no duplicates, no self-reference.
fn check_additional_intervals(strategy: &dyn ResearchStrategy) -> Result<(), String> {
    let primary = strategy.analysis_interval();
    let mut seen = std::collections::HashSet::new();
    for iv in strategy.additional_intervals() {
        if iv == "1m" {
            return Err(
                "1m cannot be used as an additional interval — 1m candles are disk-only \
                 and cannot be held in HtfData. Use analysis_interval(\"1m\") instead."
                    .to_string(),
            );
        }
        if iv == primary {
            return Err(format!(
                "additional_intervals() contains {iv:?} which is already the analysis_interval(). \
                 Remove it to avoid redundant work."
            ));
        }
        if !seen.insert(iv) {
            return Err(format!(
                "additional_intervals() contains duplicate interval {iv:?}"
            ));
        }
        parse_interval_duration(iv)
            .map_err(|e| format!("invalid additional interval {iv:?}: {e}"))?;
    }

    // Validate indicator_columns_per_interval keys are a subset of additional_intervals.
    let valid_intervals: std::collections::HashSet<&str> =
        strategy.additional_intervals().into_iter().collect();
    for iv in strategy.indicator_columns_per_interval().keys() {
        if !valid_intervals.contains(iv) {
            return Err(format!(
                "indicator_columns_per_interval() contains {iv:?} which is not in \
                 additional_intervals(). Fix the typo or add it to additional_intervals()."
            ));
        }
    }

    Ok(())
}

/// Validate strategy interval and additional intervals, exiting on failure.
pub(crate) fn validate_strategy_interval(strategy: &dyn ResearchStrategy) -> (String, Duration) {
    let result = check_strategy_interval(strategy).unwrap_or_else(|e| {
        eprintln!("FATAL: {e}");
        std::process::exit(1);
    });
    if let Err(e) = check_additional_intervals(strategy) {
        eprintln!("FATAL: {e}");
        std::process::exit(1);
    }
    result
}

// ---------------------------------------------------------------------------
// ResearchStrategy trait — the only thing experiment crates implement
// ---------------------------------------------------------------------------

/// Calibration configuration for strategies that use periodic recalibration.
pub struct CalibrationConfig {
    pub interval_hours: u32,
    pub lookback_hours: u32,
}

/// The trait every research experiment must implement.
pub trait ResearchStrategy: Send {
    /// Display name for output and results.tsv.
    fn name(&self) -> &str;

    /// Human-readable description of the strategy's hypothesis / what changed
    /// vs. the previous version. Written to the `strategy_description` column
    /// of `results.tsv` every run. The agent should update this method when
    /// the strategy changes so the row in `results.tsv` always reflects the
    /// current rationale. Default: the strategy name.
    fn description(&self) -> String {
        self.name().to_string()
    }

    /// Symbols to trade (API format, e.g. "ETHUSDT").
    fn symbols(&self) -> Vec<String>;

    /// Indicator columns to precompute (used for warmup calculation).
    fn indicator_columns(&self) -> &[&str];

    /// Cooldown policy for a given signal.
    ///
    /// The runtime enforces cooldown globally across a run using the returned
    /// `CooldownSpec`. Strategies MUST NOT track cooldown internally — emit every
    /// candidate and let the runtime filter.
    ///
    /// Use `CooldownSpec::symbol_side(signal, hours)` for the default
    /// per-(symbol, direction) behavior, or any other `CooldownKey` constructor
    /// (including `CooldownKey::custom`) to bucket by pattern, metadata, etc.
    /// Two signals that share a key must return the same `hours`.
    fn cooldown_spec(&self, signal: &Signal) -> CooldownSpec;

    /// Extra warmup bars beyond indicator requirements.
    fn extra_warmup_bars(&self) -> usize {
        100
    }

    /// Candle timeframe for analysis (e.g. "1h", "4h", "15m").
    /// Default: "1h". The runtime fetches and stores candles at this interval.
    fn analysis_interval(&self) -> &str {
        "1h"
    }

    /// Which raw datasets the strategy needs beyond OHLCV. Point-in-time
    /// context (funding, key levels, BTC structure) is declared via
    /// `required_context()`; the runtime auto-derives the corresponding
    /// `DataRequirement`s and `include_key_levels` setting. Override this
    /// method only to add OHLCV requirements (e.g. agg trades).
    fn market_data_request(&self) -> MarketDataRequest {
        MarketDataRequest::ohlcv_only(self.analysis_interval())
    }

    /// Point-in-time context dependencies.
    ///
    /// Declare everything the strategy reads via `ctx.context_at(...)` here.
    /// The runtime uses this list to drive fetching, warmup, and population
    /// of `ContextMap`. Strategies have no other path to BTC bias, key
    /// levels, or funding context.
    fn required_context(&self) -> Vec<ContextKey> {
        Vec::new()
    }

    /// Calibration config. Return Some to enable rolling calibration.
    fn calibration_config(&self) -> Option<CalibrationConfig> {
        None
    }

    /// Additional candle intervals available via `htf.additional_candles`.
    ///
    /// Candles are fetched globally and delivered through `HtfData`, clipped
    /// to the signal generation window. 1m is not supported (use
    /// `analysis_interval("1m")` instead). Must not include
    /// `analysis_interval()`.
    fn additional_intervals(&self) -> Vec<&str> {
        vec![]
    }

    /// Extra warmup bars per additional interval (keyed by interval string).
    fn extra_warmup_bars_per_interval(&self) -> HashMap<&str, usize> {
        HashMap::new()
    }

    /// Indicator columns to precompute on additional intervals.
    ///
    /// Keys must be a subset of `additional_intervals()`. Values use the
    /// same indicator name format as `indicator_columns()`. Intervals not
    /// listed get no precomputed indicators. Default: reuses
    /// `indicator_columns()` for every declared additional interval.
    fn indicator_columns_per_interval(&self) -> HashMap<&str, Vec<&str>> {
        let cols: Vec<&str> = self.indicator_columns().to_vec();
        self.additional_intervals()
            .into_iter()
            .map(|iv| (iv, cols.clone()))
            .collect()
    }

    /// Generate signals from pre-fetched candles and context.
    ///
    /// `candles` is keyed by API symbol. Each value includes warmup bars
    /// before `start`. Only emit signals with `signal_date` in `[start, end)`.
    /// `active_params` are calibrated parameters (empty for non-calibrated).
    /// `ctx` is the unified point-in-time context channel — use
    /// `ctx.context_at(key, t)` to read BTC bias, key levels, funding.
    /// `htf` contains additional-interval candles and indicators, if the
    /// strategy declared any.
    fn generate_signals(
        &self,
        candles: &BTreeMap<String, &[Candle]>,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        active_params: &HashMap<String, serde_json::Value>,
        ctx: &ContextMap,
        htf: &HtfData,
    ) -> Vec<Signal>;

    /// Run calibration grid search on lookback candles. `ctx` and `htf` are
    /// clipped to the lookback window end.
    fn calibrate(
        &self,
        _candles: &BTreeMap<String, &[Candle]>,
        _ctx: &ContextMap,
        _htf: &HtfData,
    ) -> Option<HashMap<String, serde_json::Value>> {
        None
    }
}

// ---------------------------------------------------------------------------
// CLI interface
// ---------------------------------------------------------------------------

/// Parsed command-line arguments for the research runner.
pub struct RunConfig {
    pub window_set: WindowSet,
    pub approximate: bool,
    pub command: RunCommand,
    /// If set, dump the EvaluationReport as JSON to this path for parity checks.
    pub dump_trades: Option<String>,
}

pub enum RunCommand {
    Eval,
    Validate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowSet {
    Development,
    Evaluation,
    All,
    Test,
    Complete,
}

impl WindowSet {
    pub fn parse(value: &str) -> Option<Self> {
        match value.to_ascii_lowercase().as_str() {
            "development" | "dev" => Some(Self::Development),
            "evaluation" | "eval" | "holdout" => Some(Self::Evaluation),
            "all" => Some(Self::All),
            "test" | "tests" => Some(Self::Test),
            "complete" => Some(Self::Complete),
            _ => None,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Development => "development",
            Self::Evaluation => "evaluation",
            Self::All => "all",
            Self::Test => "test",
            Self::Complete => "complete",
        }
    }

    pub fn windows(&self) -> Vec<EvalWindow> {
        match self {
            Self::Development => calendar::development_windows(),
            Self::Evaluation => calendar::evaluation_windows(),
            Self::All => calendar::all_windows(),
            Self::Test => calendar::test_windows(),
            Self::Complete => calendar::complete_windows(),
        }
    }
}

/// Parse CLI args for a research experiment binary.
pub fn parse_run_config<I>(args: I) -> Result<RunConfig, String>
where
    I: IntoIterator<Item = String>,
{
    let args: Vec<String> = args.into_iter().collect();
    let program = args.first().cloned().unwrap_or_else(|| "experiment".into());

    let usage = format!(
        "Usage:\n  {program} eval --windows <dev|eval|all|test|complete> [--approximate|--exact]\n  {program} validate --windows <dev|eval|all|test|complete> [--approximate|--exact]"
    );

    if args.len() < 2 {
        return Err(format!("Missing command.\n\n{usage}"));
    }

    if args[1] == "--help" || args[1] == "-h" {
        return Err(usage);
    }

    let command = match args[1].as_str() {
        "eval" => RunCommand::Eval,
        "validate" => RunCommand::Validate,
        other => return Err(format!("Unknown command: {other}\n\n{usage}")),
    };

    let mut window_set = WindowSet::Development;
    let mut approximate = true;
    let mut dump_trades = None;
    let mut idx = 2;

    while idx < args.len() {
        match args[idx].as_str() {
            "--windows" | "-w" => {
                let value = args
                    .get(idx + 1)
                    .ok_or_else(|| format!("Missing value for --windows\n\n{usage}"))?;
                window_set = WindowSet::parse(value)
                    .ok_or_else(|| format!("Unknown window set: {value}\n\n{usage}"))?;
                idx += 2;
            }
            "--exact" => {
                approximate = false;
                idx += 1;
            }
            "--approximate" => {
                approximate = true;
                idx += 1;
            }
            "--dump-trades" => {
                let value = args
                    .get(idx + 1)
                    .ok_or_else(|| format!("Missing value for --dump-trades\n\n{usage}"))?;
                dump_trades = Some(value.clone());
                idx += 2;
            }
            other => return Err(format!("Unknown argument: {other}\n\n{usage}")),
        }
    }

    Ok(RunConfig {
        window_set,
        approximate,
        command,
        dump_trades,
    })
}

// ---------------------------------------------------------------------------
// MemoryProvider — feeds resolver from CandleStore
// ---------------------------------------------------------------------------

/// Market data provider for trade resolution.
///
/// All candle data (1h and 1m) is served from the `CandleStore` — no
/// network calls.  Phases 1–3 of the pipeline guarantee that the store
/// contains every candle the resolver will ask for.  The `BinanceClient`
/// is only retained for `fetch_agg_trades` (exact-mode resolution).
struct MemoryProvider<'a> {
    store: &'a mut CandleStore,
    client: &'a BinanceClient,
}

impl<'a> MarketDataProvider for MemoryProvider<'a> {
    fn fetch_analysis_candles(
        &mut self,
        symbol: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Vec<Candle> {
        self.store.get_range(symbol, "1h", start, end)
    }

    fn fetch_minute_candles(
        &mut self,
        symbol: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Vec<Candle> {
        self.store.get_range(symbol, "1m", start, end)
    }

    fn fetch_agg_trades(
        &mut self,
        symbol: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Vec<claude_trader_models::AggTrade> {
        self.client
            .fetch_agg_trades(symbol, start, end)
            .unwrap_or_default()
    }
}

// ---------------------------------------------------------------------------
// Supplementary data fetching
// ---------------------------------------------------------------------------

/// Maximum number of outer-loop attempts before `ensure_candles` gives
/// up with an error. A single `KlineFetch::Interrupted` on a gap leaves
/// its uncovered suffix in place; the next attempt picks it up, so two
/// attempts give exactly one retry on an interrupted fetch.
const ENSURE_CANDLES_MAX_ATTEMPTS: u32 = 2;

/// Fill coverage gaps for `(symbol, interval)` between `[start, end)`
/// using the supplied fetcher closure.
///
/// Drives [`CandleStore::candle_coverage_gaps`] → for each reported gap:
///
/// - `KlineFetch::Complete(rows)` — persist rows, record coverage for
///   the full requested gap. Empty `Complete` still records coverage
///   (e.g. pre-listing probes), matching the funding path's empty-probe
///   semantic.
/// - `KlineFetch::Interrupted { rows, covered_up_to_ms }` — persist
///   partial rows, record coverage only for `[gap_start, covered_up_to_ms)`.
///   The uncovered suffix stays uncovered; the outer loop re-detects it
///   on the next attempt.
/// - `Err` — bail without recording coverage for the failed gap. Any
///   rows from earlier successful gaps in the same call remain.
///
/// The outer loop re-queries gaps after each pass. Progress made via
/// partial `Interrupted` rounds is preserved across iterations; if the
/// budget of [`ENSURE_CANDLES_MAX_ATTEMPTS`] attempts is exhausted with
/// gaps still outstanding, the function returns `Err`.
///
/// Pure over the fetcher — takes a closure rather than `&BinanceClient`
/// so tests can drive the full coverage-driven path without a network.
pub(crate) fn ensure_candles_with_fetcher<F>(
    store: &mut CandleStore,
    mut fetcher: F,
    symbol: &str,
    interval: &str,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
) -> Result<Vec<Candle>, String>
where
    F: FnMut(DateTime<Utc>, DateTime<Utc>) -> Result<KlineFetch, DataError>,
{
    let mut attempts: u32 = 0;
    loop {
        let gaps = store.candle_coverage_gaps(symbol, interval, start, end);
        if gaps.is_empty() {
            break;
        }
        if attempts >= ENSURE_CANDLES_MAX_ATTEMPTS {
            return Err(format!(
                "Incomplete {interval} coverage for {symbol} after {ENSURE_CANDLES_MAX_ATTEMPTS} attempts: {} gap(s) remain",
                gaps.len()
            ));
        }
        for (g_start, g_end) in &gaps {
            let outcome = fetcher(*g_start, *g_end)
                .map_err(|e| format!("{interval} fetch failed for {symbol}: {e}"))?;
            match outcome {
                KlineFetch::Complete(rows) => {
                    if !rows.is_empty() {
                        store.put(symbol, interval, &rows);
                    }
                    store.record_candle_coverage(symbol, interval, *g_start, *g_end);
                }
                KlineFetch::Interrupted {
                    rows,
                    covered_up_to_ms,
                } => {
                    if !rows.is_empty() {
                        store.put(symbol, interval, &rows);
                    }
                    // Only the probed prefix counts as covered. The
                    // uncovered suffix is surfaced as a fresh gap on
                    // the next outer iteration.
                    if let Some(covered_end) =
                        DateTime::<Utc>::from_timestamp_millis(covered_up_to_ms)
                    {
                        if covered_end > *g_start {
                            store.record_candle_coverage(
                                symbol,
                                interval,
                                *g_start,
                                covered_end,
                            );
                        }
                    }
                }
            }
        }
        attempts += 1;
    }
    Ok(store.get_range(symbol, interval, start, end))
}

/// Coverage-driven candle fetch. Defers to
/// [`ensure_candles_with_fetcher`] bound to `client.fetch_klines`.
///
/// Returns `Err` when coverage cannot be completed within
/// [`ENSURE_CANDLES_MAX_ATTEMPTS`] — callers treat this as a hard
/// failure (typically abort-worthy). Partial data is always persisted
/// before the error bubbles, so a subsequent run resumes from wherever
/// the failed run left off.
pub(crate) fn ensure_candles(
    store: &mut CandleStore,
    client: &BinanceClient,
    symbol: &str,
    interval: &str,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
) -> Result<Vec<Candle>, String> {
    ensure_candles_with_fetcher(
        store,
        |s, e| client.fetch_klines(symbol, interval, s, e),
        symbol,
        interval,
        start,
        end,
    )
}

// ---------------------------------------------------------------------------
// Additional interval helpers
// ---------------------------------------------------------------------------

/// Compute the global_start for an additional interval, accounting for
/// indicator warmup, per-interval extra warmup, and calibration lookback.
fn extra_interval_global_start(
    extra_iv: &str,
    iv_indicator_cols: &[&str],
    extra_warmup_map: &HashMap<&str, usize>,
    calib_config: &Option<CalibrationConfig>,
    earliest_period_start: DateTime<Utc>,
) -> DateTime<Utc> {
    let extra_iv_dur = parse_interval_duration(extra_iv).unwrap();
    let extra_iv_secs = extra_iv_dur.num_seconds();

    let indicator_warmup = claude_trader_indicators::required_warmup(iv_indicator_cols);
    let mut extra_warmup =
        indicator_warmup + extra_warmup_map.get(extra_iv).copied().unwrap_or(0);

    if let Some(cc) = calib_config {
        let calib_bars =
            ((cc.lookback_hours as f64) * 3600.0 / extra_iv_secs as f64).ceil() as usize;
        if calib_bars > extra_warmup {
            extra_warmup = calib_bars;
        }
    }

    let extra_warmup_dur = extra_iv_dur * extra_warmup as i32;
    earliest_period_start
        - Duration::days(COOLDOWN_WARMUP_DAYS)
        - extra_warmup_dur
}

/// Fetch additional interval candles into store.
///
/// Called from both `run_eval_pipeline()` and `run_validation()` to ensure
/// both paths load identical data. Uses per-interval warmup computation
/// via `extra_interval_global_start()`.
pub(crate) fn ensure_additional_interval_candles(
    store: &mut CandleStore,
    client: &BinanceClient,
    strategy: &dyn ResearchStrategy,
    api_symbols: &[String],
    iv_indicator_map: &HashMap<&str, Vec<&str>>,
    calib_config: &Option<CalibrationConfig>,
    earliest_period_start: DateTime<Utc>,
    global_end: DateTime<Utc>,
) {
    let extra_intervals = strategy.additional_intervals();
    if extra_intervals.is_empty() {
        return;
    }

    let extra_warmup_map = strategy.extra_warmup_bars_per_interval();

    for extra_iv in &extra_intervals {
        let iv_cols: Vec<&str> = iv_indicator_map
            .get(extra_iv)
            .cloned()
            .unwrap_or_default();
        let extra_start = extra_interval_global_start(
            extra_iv,
            &iv_cols,
            &extra_warmup_map,
            calib_config,
            earliest_period_start,
        );

        for sym in api_symbols {
            // `ensure_candles` is the single coverage authority — it errors
            // out if it can't complete coverage within its retry budget.
            if let Err(e) = ensure_candles(store, client, sym, extra_iv, extra_start, global_end) {
                eprintln!("FATAL: additional interval {extra_iv} for {sym}: {e}");
                std::process::exit(1);
            }
        }
    }
}

/// Populate `HtfData` from the store for all declared additional intervals.
///
/// Uses per-interval global_start to read back the full warmup range that
/// `ensure_additional_interval_candles` fetched. Computes indicators for
/// each interval according to `iv_indicator_map`.
pub(crate) fn populate_htf_data(
    htf: &mut HtfData,
    store: &mut CandleStore,
    strategy: &dyn ResearchStrategy,
    api_symbols: &[String],
    iv_indicator_map: &HashMap<&str, Vec<&str>>,
    calib_config: &Option<CalibrationConfig>,
    earliest_period_start: DateTime<Utc>,
    global_end: DateTime<Utc>,
) {
    let extra_intervals = strategy.additional_intervals();
    if extra_intervals.is_empty() {
        return;
    }

    let extra_warmup_map = strategy.extra_warmup_bars_per_interval();

    for extra_iv in &extra_intervals {
        let iv_cols: Vec<&str> = iv_indicator_map
            .get(extra_iv)
            .cloned()
            .unwrap_or_default();
        let extra_start = extra_interval_global_start(
            extra_iv,
            &iv_cols,
            &extra_warmup_map,
            calib_config,
            earliest_period_start,
        );

        let mut interval_map = HashMap::new();
        for sym in api_symbols {
            let candles = store.get_range(sym, extra_iv, extra_start, global_end);
            if !candles.is_empty() {
                interval_map.insert(sym.clone(), candles);
            }
        }

        if !iv_cols.is_empty() {
            let mut ind_map = HashMap::new();
            for (sym, candles) in &interval_map {
                let n = candles.len();
                let mut ohlcv = claude_trader_indicators::OhlcvFrame {
                    open: Vec::with_capacity(n),
                    high: Vec::with_capacity(n),
                    low: Vec::with_capacity(n),
                    close: Vec::with_capacity(n),
                    volume: Vec::with_capacity(n),
                    taker_buy_volume: Vec::with_capacity(n),
                };
                for c in candles {
                    ohlcv.open.push(c.open);
                    ohlcv.high.push(c.high);
                    ohlcv.low.push(c.low);
                    ohlcv.close.push(c.close);
                    ohlcv.volume.push(c.volume);
                    ohlcv.taker_buy_volume.push(c.taker_buy_volume);
                }
                if ohlcv.is_empty() {
                    continue;
                }
                match claude_trader_indicators::compute_indicators(&ohlcv, &iv_cols) {
                    Ok(result) => {
                        ind_map.insert(sym.clone(), result);
                    }
                    Err(e) => {
                        eprintln!(
                            "FATAL: compute_indicators failed for {sym} on {extra_iv}: {e}"
                        );
                        std::process::exit(1);
                    }
                }
            }
            htf.additional_indicators
                .insert(extra_iv.to_string(), ind_map);
        }

        htf.additional_candles
            .insert(extra_iv.to_string(), interval_map);
    }

    htf.debug_assert_sorted();
}

// ---------------------------------------------------------------------------
// Funding rate helpers
// ---------------------------------------------------------------------------

/// Fill coverage gaps for `symbol`'s funding rates between `[start, end)`
/// using the supplied fetcher closure. For every gap the fetcher is
/// invoked once; any rows returned are persisted via `put_funding`, and
/// the probed range is **unconditionally** recorded via
/// `record_funding_coverage` — including when the response is empty.
/// The latter is load-bearing: funding ticks every ~8h, so a trailing
/// probe is almost always empty; recording it anyway is what stops the
/// re-probe-every-run behavior that motivated the coverage design.
///
/// On fetch `Err`, the function bails immediately without recording
/// coverage for the failed gap. Rows persisted by prior successful gaps
/// in the same call remain in the store (progress preserved).
///
/// Pure over the fetcher — takes a closure rather than `&BinanceClient`
/// so tests can drive the full coverage path without a network.
pub(crate) fn ensure_funding_rates_with_fetcher<F>(
    store: &mut CandleStore,
    mut fetcher: F,
    symbol: &str,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
) -> Result<Vec<FundingRate>, String>
where
    F: FnMut(DateTime<Utc>, DateTime<Utc>) -> Result<Vec<FundingRate>, DataError>,
{
    let gaps = store.funding_coverage_gaps(symbol, start, end);
    for (g_start, g_end) in &gaps {
        match fetcher(*g_start, *g_end) {
            Ok(rows) => {
                if !rows.is_empty() {
                    store.put_funding(symbol, &rows);
                }
                store.record_funding_coverage(symbol, *g_start, *g_end);
            }
            Err(e) => {
                return Err(format!("funding_rates fetch failed for {symbol}: {e}"));
            }
        }
    }
    Ok(store.get_funding_range(symbol, start, end))
}

fn ensure_funding_rates(
    store: &mut CandleStore,
    client: &BinanceClient,
    symbol: &str,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
) -> Result<Vec<FundingRate>, String> {
    ensure_funding_rates_with_fetcher(
        store,
        |s, e| client.fetch_funding_rates(symbol, s, e),
        symbol,
        start,
        end,
    )
}

/// Raw per-symbol sources fetched once per run. `funding_rates` and
/// `key_levels` feed `build_context`; they are never exposed to strategies.
pub(crate) struct FetchedSources {
    pub funding_rates: HashMap<String, Vec<FundingRate>>,
    pub key_levels: HashMap<String, Vec<(DateTime<Utc>, KeyLevels)>>,
}

/// Fetch exactly the symbols declared in `required_ctx`. Funding is fetched
/// only for `ContextKey::Funding(sym)` entries; key levels only for
/// `ContextKey::KeyLevels(sym)` entries. Missing or empty responses produce
/// an error rather than a silent empty series downstream. `BtcStructure` is
/// not handled here — it's built separately via `build_btc_events`.
fn fetch_sources(
    store: &mut CandleStore,
    client: &BinanceClient,
    required_ctx: &[ContextKey],
    start: DateTime<Utc>,
    end: DateTime<Utc>,
) -> Result<FetchedSources, String> {
    use std::collections::BTreeSet;

    let mut funding_syms: BTreeSet<&str> = BTreeSet::new();
    let mut key_levels_syms: BTreeSet<&str> = BTreeSet::new();
    for key in required_ctx {
        match key {
            ContextKey::Funding(s) => {
                funding_syms.insert(s.as_str());
            }
            ContextKey::KeyLevels(s) => {
                key_levels_syms.insert(s.as_str());
            }
            ContextKey::BtcStructure => {}
        }
    }

    let mut funding_rates: HashMap<String, Vec<FundingRate>> = HashMap::new();
    let mut key_levels: HashMap<String, Vec<(DateTime<Utc>, KeyLevels)>> = HashMap::new();
    let mut errors: Vec<String> = Vec::new();

    if !funding_syms.is_empty() {
        let t = Instant::now();
        for sym in &funding_syms {
            match ensure_funding_rates(store, client, sym, start, end) {
                Ok(rates) if !rates.is_empty() => {
                    funding_rates.insert((*sym).to_string(), rates);
                }
                Ok(_) => {
                    errors.push(format!("funding_rates: empty response for {sym}"));
                }
                Err(e) => {
                    errors.push(format!("funding_rates: fetch failed for {sym}: {e}"));
                }
            }
        }
        let total: usize = funding_rates.values().map(|v| v.len()).sum();
        eprintln!(
            "Funding rates: {} entries for {} symbols in {:.0}ms",
            total,
            funding_rates.len(),
            t.elapsed().as_secs_f64() * 1000.0,
        );
    }

    if !key_levels_syms.is_empty() {
        let t = Instant::now();
        let kl_start = chrono::NaiveDate::from_ymd_opt((start.year() - 1).max(1), 1, 1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc();
        let hourly_start = start - Duration::days(7);

        for sym in &key_levels_syms {
            if let Err(e) = ensure_candles(store, client, sym, "1h", hourly_start, end) {
                errors.push(format!("key_levels: {e}"));
                continue;
            }
            let timestamps: Vec<DateTime<Utc>> = store
                .get_range_ref(sym, "1h", start, end)
                .iter()
                .filter(|c| c.open_time >= start)
                .map(|c| c.close_time)
                .collect();
            if timestamps.is_empty() {
                errors.push(format!("key_levels: no hourly data for {sym}"));
                continue;
            }
            match key_levels::fetch_symbol_key_levels(
                store,
                client,
                sym,
                kl_start,
                hourly_start,
                end,
                &timestamps,
            ) {
                Ok(kl) if !kl.is_empty() => {
                    key_levels.insert((*sym).to_string(), kl);
                }
                Ok(_) => {
                    errors.push(format!("key_levels: empty result for {sym}"));
                }
                Err(e) => {
                    errors.push(format!("key_levels: {e}"));
                }
            }
        }
        let total: usize = key_levels.values().map(|v| v.len()).sum();
        eprintln!(
            "Key levels: {} entries for {} symbols in {:.0}ms",
            total,
            key_levels.len(),
            t.elapsed().as_secs_f64() * 1000.0,
        );
    }

    if !errors.is_empty() {
        return Err(format!(
            "Failed to fetch {} requested dataset(s):\n  {}",
            errors.len(),
            errors.join("\n  "),
        ));
    }

    Ok(FetchedSources {
        funding_rates,
        key_levels,
    })
}

// ---------------------------------------------------------------------------
// BTC structure helpers
// ---------------------------------------------------------------------------

fn synthesize_daily_candles(hourly: &[Candle]) -> Vec<Candle> {
    use std::collections::BTreeMap;
    let mut days: BTreeMap<chrono::NaiveDate, Vec<&Candle>> = BTreeMap::new();
    for c in hourly {
        let date = c.close_time.date_naive();
        days.entry(date).or_default().push(c);
    }
    days.into_iter()
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
            let open_time = bars.first().unwrap().open_time;
            let close_time = bars.last().unwrap().close_time;
            Candle {
                open_time,
                close_time,
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

/// Build BTC daily structure events. Each entry is `(daily_close_time,
/// MarketBias)` — source-event timestamps, independent of any analysis
/// interval. `context_at`'s `partition_point` lookup handles expansion onto
/// arbitrary query times.
fn build_btc_events(
    store: &mut CandleStore,
    client: &BinanceClient,
    global_start: DateTime<Utc>,
    global_end: DateTime<Utc>,
) -> Vec<(DateTime<Utc>, MarketBias)> {
    let t_btc = Instant::now();
    let btc_sym = "BTCUSDT";
    let daily_start = global_start - Duration::days(60);

    let needs_refresh = {
        let existing_daily = store.get_range_ref(btc_sym, "1d", daily_start, global_end);
        existing_daily.is_empty()
            || existing_daily.last().unwrap().close_time < global_end - Duration::days(2)
    };
    if needs_refresh {
        match ensure_candles(store, client, btc_sym, "1d", daily_start, global_end) {
            Ok(ref candles)
                if !candles.is_empty()
                    && candles.last().unwrap().close_time >= global_end - Duration::days(2) =>
            { /* store has fresh enough data */ }
            _ => {
                eprintln!("  Synthesizing 1d candles from 1h for {btc_sym}...");
                let hourly = ensure_candles(store, client, btc_sym, "1h", daily_start, global_end)
                    .ok()
                    .filter(|h| !h.is_empty())
                    .unwrap_or_else(|| store.get_range(btc_sym, "1h", daily_start, global_end));
                let daily = synthesize_daily_candles(&hourly);
                if !daily.is_empty() {
                    store.put(btc_sym, "1d", &daily);
                }
            }
        }
    }

    let daily_candles = store.get_range_ref(btc_sym, "1d", daily_start, global_end);
    if daily_candles.is_empty() {
        eprintln!("WARNING: No daily BTC candles — BTC structure disabled.");
        return Vec::new();
    }

    let mut provider = claude_trader_btc_structure::DailyStructureProvider::new();
    provider.compute_from_candles(daily_candles);

    let daily_dates: Vec<DateTime<Utc>> = daily_candles.iter().map(|c| c.close_time).collect();
    let feature_matrix = provider.feature_matrix().cloned().unwrap_or_default();

    let mut events: Vec<(DateTime<Utc>, MarketBias)> =
        Vec::with_capacity(daily_dates.len().min(feature_matrix.len()));
    for (idx, ts) in daily_dates.iter().enumerate() {
        if idx >= feature_matrix.len() {
            break;
        }
        if let Some(fv) = feature_matrix[idx].get("market_bias_after_close") {
            let bias = match fv {
                claude_trader_btc_structure::engine::FeatureValue::Str(s) => {
                    MarketBias::from_lowercase_str(s.as_ref())
                }
                _ => MarketBias::Neutral,
            };
            events.push((*ts, bias));
        }
    }

    eprintln!(
        "BTC structure: {} daily events in {:.0}ms",
        events.len(),
        t_btc.elapsed().as_secs_f64() * 1000.0,
    );
    events
}

// ---------------------------------------------------------------------------
// Calibration orchestration
// ---------------------------------------------------------------------------

/// A time interval during which a fixed set of calibration params was active.
#[derive(Debug, Clone)]
pub struct CalibrationInterval {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub params: HashMap<String, serde_json::Value>,
}

fn calibration_boundaries(
    sig_start: DateTime<Utc>,
    sig_end: DateTime<Utc>,
    interval: Duration,
    calibration_anchor: DateTime<Utc>,
) -> Vec<DateTime<Utc>> {
    let elapsed_secs = (sig_start - calibration_anchor).num_seconds();
    let interval_secs = interval.num_seconds();
    let k = if elapsed_secs >= 0 {
        elapsed_secs / interval_secs
    } else {
        0
    };

    let mut boundaries: Vec<DateTime<Utc>> = Vec::new();
    let mut t = calibration_anchor + Duration::seconds(k * interval_secs);
    while t < sig_end {
        boundaries.push(t);
        t += interval;
    }
    if boundaries.is_empty() {
        boundaries.push(sig_start);
    }
    boundaries
}

fn generate_signals_with_calibration(
    strategy: &dyn ResearchStrategy,
    full_candles: &BTreeMap<String, Vec<Candle>>,
    store: &mut CandleStore,
    api_symbols: &[String],
    analysis_interval: &str,
    sig_start: DateTime<Utc>,
    sig_end: DateTime<Utc>,
    cc: &CalibrationConfig,
    full_ctx: &ContextMap,
    full_htf: &HtfData,
    intervals_out: &mut Vec<CalibrationInterval>,
) -> Vec<Signal> {
    let interval = Duration::hours(cc.interval_hours as i64);
    let lookback = Duration::hours(cc.lookback_hours as i64);
    let boundaries = calibration_boundaries(sig_start, sig_end, interval, sig_start);

    let mut all_signals: Vec<Signal> = Vec::new();

    for (i, &calib_time) in boundaries.iter().enumerate() {
        let lookback_start = calib_time - lookback;
        let lookback_end = calib_time;

        let mut lookback_candles: BTreeMap<String, Vec<Candle>> = BTreeMap::new();
        for sym in api_symbols {
            let candles = store.get_range(sym, analysis_interval, lookback_start, lookback_end);
            if !candles.is_empty() {
                lookback_candles.insert(sym.clone(), candles);
            }
        }

        let active_params = if !lookback_candles.is_empty() {
            let mut lookback_ctx = full_ctx.clone();
            lookback_ctx.clip_in_place(lookback_end);
            let lookback_htf = full_htf.truncated_at(lookback_end);
            let lookback_view: BTreeMap<String, &[Candle]> = lookback_candles
                .iter()
                .map(|(sym, v)| (sym.clone(), v.as_slice()))
                .collect();
            strategy
                .calibrate(&lookback_view, &lookback_ctx, &lookback_htf)
                .unwrap_or_default()
        } else {
            HashMap::new()
        };

        let gen_start = sig_start.max(calib_time);
        let gen_end = if i + 1 < boundaries.len() {
            boundaries[i + 1].min(sig_end)
        } else {
            sig_end
        };

        intervals_out.push(CalibrationInterval {
            start: gen_start,
            end: gen_end,
            params: active_params.clone(),
        });

        if gen_start < gen_end {
            let clipped: BTreeMap<String, &[Candle]> = full_candles
                .iter()
                .map(|(sym, candles)| {
                    let end_idx = candles.partition_point(|c| c.close_time <= gen_end);
                    (sym.clone(), &candles[..end_idx])
                })
                .collect();
            let mut clipped_ctx = full_ctx.clone();
            clipped_ctx.clip_in_place(gen_end);
            let clipped_htf = full_htf.truncated_at(gen_end);
            let sigs = strategy.generate_signals(
                &clipped,
                gen_start,
                gen_end,
                &active_params,
                &clipped_ctx,
                &clipped_htf,
            );
            all_signals.extend(sigs);
        }
    }

    all_signals.sort_by_key(|s| s.signal_date);
    all_signals
}

// ---------------------------------------------------------------------------
// PipelineState — captured during signal generation for validator replay
// ---------------------------------------------------------------------------

/// State captured during reference signal generation.
///
/// The validator uses this to replay each signal with the context and HTF
/// data active at the signal's time, rather than empty defaults.
pub(crate) struct PipelineState {
    pub full_ctx: ContextMap,
    pub full_htf: HtfData,
    pub calibration_intervals: Vec<CalibrationInterval>,
}

// ---------------------------------------------------------------------------
// Signal generation — shared between eval and validate
// ---------------------------------------------------------------------------

/// Result of the shared signal-generation pipeline.
pub(crate) struct GenerationResult {
    /// Per-window signals after cooldown filtering (used by eval).
    pub window_signals: Vec<(EvalWindow, Vec<Arc<Signal>>)>,
    /// All raw candidate signals across every period, globally sorted by
    /// `signal_date`, before cooldown is applied (used by validator).
    pub raw_signals: Vec<Arc<Signal>>,
    /// Pipeline state for validator replay.
    pub state: PipelineState,
}

/// Generate signals using the same pipeline as eval (calibration + BTC
/// structure + funding + key levels). Returns per-window signals
/// (post-cooldown), raw pre-cooldown signals, and the full pipeline state.
pub(crate) fn generate_all_signals(
    strategy: &dyn ResearchStrategy,
    store: &mut CandleStore,
    client: &BinanceClient,
    api_symbols: &[String],
    windows: &[EvalWindow],
    warmup_bars: usize,
    analysis_interval: &str,
    interval_duration: Duration,
    global_start: DateTime<Utc>,
    global_end: DateTime<Utc>,
) -> GenerationResult {
    let calib_config = strategy.calibration_config();
    let required = strategy.required_context();

    let needs_btc = required
        .iter()
        .any(|k| matches!(k, ContextKey::BtcStructure));

    // Extend the fetch start by the largest per-key warmup so on-demand
    // computations (e.g. funding z-score) see enough history at the first bar.
    let warmup = required
        .iter()
        .map(context_warmup)
        .max()
        .unwrap_or_else(Duration::zero);
    let fetch_start = global_start - warmup;

    let sources = match fetch_sources(store, client, &required, fetch_start, global_end) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("\nFATAL: {e}");
            eprintln!("Strategy requested these datasets but they could not be fetched.");
            eprintln!("Cannot produce valid research artifacts from incomplete data. Aborting.");
            std::process::exit(1);
        }
    };

    let btc_events = if needs_btc {
        build_btc_events(store, client, global_start, global_end)
    } else {
        Vec::new()
    };

    let full_ctx = build_context(
        &required,
        &sources.funding_rates,
        &sources.key_levels,
        &btc_events,
    );

    let mut full_htf = HtfData::default();
    let iv_indicator_map = strategy.indicator_columns_per_interval();
    let earliest_period_start = group_into_periods(windows, Duration::days(COOLDOWN_WARMUP_DAYS))
        .iter()
        .map(|p| p.start)
        .min()
        .unwrap_or(global_start);
    populate_htf_data(
        &mut full_htf,
        store,
        strategy,
        api_symbols,
        &iv_indicator_map,
        &calib_config,
        earliest_period_start,
        global_end,
    );

    let periods = group_into_periods(windows, Duration::days(COOLDOWN_WARMUP_DAYS));
    let mut all_raw_signals: Vec<Arc<Signal>> = Vec::new();
    let mut calibration_intervals: Vec<CalibrationInterval> = Vec::new();

    let warmup_dur = interval_duration * warmup_bars as i32;

    for period in &periods {
        let sig_start = period.start - Duration::days(COOLDOWN_WARMUP_DAYS);
        let sig_end = period.end;
        let fetch_start = sig_start - warmup_dur;

        let mut candles_map: BTreeMap<String, Vec<Candle>> = BTreeMap::new();
        for sym in api_symbols {
            let candles = store.get_range(sym, analysis_interval, fetch_start, sig_end);
            if !candles.is_empty() {
                candles_map.insert(sym.clone(), candles);
            }
        }

        let period_signals = if let Some(ref cc) = calib_config {
            generate_signals_with_calibration(
                strategy,
                &candles_map,
                store,
                api_symbols,
                analysis_interval,
                sig_start,
                sig_end,
                cc,
                &full_ctx,
                &full_htf,
                &mut calibration_intervals,
            )
        } else {
            let empty_params = HashMap::new();
            let mut clipped_ctx = full_ctx.clone();
            clipped_ctx.clip_in_place(sig_end);
            let clipped_htf = full_htf.truncated_at(sig_end);
            let candles_view: BTreeMap<String, &[Candle]> = candles_map
                .iter()
                .map(|(sym, v)| (sym.clone(), v.as_slice()))
                .collect();
            strategy.generate_signals(
                &candles_view,
                sig_start,
                sig_end,
                &empty_params,
                &clipped_ctx,
                &clipped_htf,
            )
        };

        all_raw_signals.extend(period_signals.into_iter().map(Arc::new));
    }

    // Global cooldown: sort every raw signal across all periods, apply the
    // per-signal spec filter once, then distribute the survivors into windows.
    // Periods overlap by COOLDOWN_WARMUP_DAYS, so concatenating per-period
    // slices does not yield a globally-sorted stream — sort explicitly here.
    all_raw_signals.sort_by_key(|s| s.signal_date);
    let filtered = enforce_signal_cooldown(&all_raw_signals, |s| strategy.cooldown_spec(s));

    let mut all_window_trades: Vec<(EvalWindow, Vec<Arc<Signal>>)> = Vec::new();
    for period in &periods {
        for window in &period.windows {
            let window_signals: Vec<Arc<Signal>> = filtered
                .iter()
                .filter(|s| s.signal_date >= window.start && s.signal_date < window.end)
                .cloned()
                .collect();
            all_window_trades.push((window.clone(), window_signals));
        }
    }

    let state = PipelineState {
        full_ctx,
        full_htf,
        calibration_intervals,
    };

    GenerationResult {
        window_signals: all_window_trades,
        raw_signals: all_raw_signals,
        state,
    }
}

// ---------------------------------------------------------------------------
// Report printing
// ---------------------------------------------------------------------------

fn rate_to_percent(r: f64) -> f64 {
    r * 100.0
}

fn normalize_signed_zero(v: f64) -> f64 {
    if v == 0.0 {
        0.0
    } else {
        v
    }
}

fn print_row(cs: &CategorySummary) {
    println!(
        "{:<40} | {:>5} | {:>+8.2}% | {:>5.1}% | {:>+7.2}% | {:>+7.2}% | {:>6} | {:>4}/{:<4} | {:>5.1}% | {:>5.2} | {:>7.2} | {:>5.1}% | {:>6.2} | {:>7.2}",
        cs.category,
        cs.positive_weeks,
        normalize_signed_zero(cs.total_pnl),
        rate_to_percent(cs.weekly_win_rate),
        normalize_signed_zero(cs.worst_week_pnl),
        normalize_signed_zero(cs.best_week_pnl),
        cs.total_trades,
        cs.short_trades,
        cs.long_trades,
        rate_to_percent(cs.trade_win_rate),
        normalize_signed_zero(cs.profit_factor),
        normalize_signed_zero(cs.sortino_ratio),
        normalize_signed_zero(cs.max_drawdown_pct),
        normalize_signed_zero(cs.weekly_omega_ratio),
        normalize_signed_zero(cs.preference_score),
    );
}

// ---------------------------------------------------------------------------
// EvaluationResult — returned from run_evaluation for output saving
// ---------------------------------------------------------------------------

/// Full result of an evaluation run, used for output saving.
pub struct EvaluationResult {
    pub strategy_name: String,
    pub strategy_description: String,
    pub window_set_label: String,
    pub approximate: bool,
    pub summaries: Vec<CategorySummary>,
    pub overall: CategorySummary,
    pub report: EvaluationReport,
    pub symbols: Vec<String>,
    /// Calibration intervals recorded during signal generation (empty if no calibration).
    pub calibration_intervals: Vec<CalibrationInterval>,
}

// ---------------------------------------------------------------------------
// run_evaluation — the main pipeline
// ---------------------------------------------------------------------------

/// Run the full evaluation pipeline for a research strategy.
///
/// This is the single entry point experiment crates call from their `main.rs`.
pub fn run_evaluation(strategy: &dyn ResearchStrategy, config: &RunConfig) {
    match config.command {
        RunCommand::Eval => run_eval_pipeline(strategy, config),
        RunCommand::Validate => {
            let exit_code = validate::run_validation(strategy, config);
            process::exit(exit_code);
        }
    }
}

fn run_eval_pipeline(strategy: &dyn ResearchStrategy, config: &RunConfig) {
    let t_total = Instant::now();

    let windows = config.window_set.windows();
    let api_symbols = strategy.symbols();
    let indicator_cols = strategy.indicator_columns();
    let mut warmup_bars =
        claude_trader_indicators::required_warmup(indicator_cols) + strategy.extra_warmup_bars();

    let calib_config = strategy.calibration_config();

    let (analysis_interval, interval_duration) = validate_strategy_interval(strategy);
    let interval_secs = interval_duration.num_seconds();

    if let Some(ref cc) = calib_config {
        let calib_bars =
            ((cc.lookback_hours as f64) * 3600.0 / interval_secs as f64).ceil() as usize;
        if calib_bars > warmup_bars {
            warmup_bars = calib_bars;
        }
    }

    let portfolio_config = PortfolioConfig {
        approximate: config.approximate,
        seed: None,
        ..Default::default()
    };

    println!("{} — Rust Evaluation", strategy.name());
    println!("Windows: {} {}", windows.len(), config.window_set.label());
    println!(
        "Mode: {}",
        if config.approximate {
            "approximate"
        } else {
            "exact"
        }
    );
    println!("Symbols: {}", api_symbols.len());
    if calib_config.is_some() {
        println!("Calibration: enabled");
    }
    println!();

    if !config.approximate {
        eprintln!("Exact mode: aggregate trades will be fetched on demand (slower).");
    }

    let periods = group_into_periods(&windows, Duration::days(COOLDOWN_WARMUP_DAYS));
    // +1 safety bar so mid-interval period.start values still receive at
    // least `warmup_bars` fully-closed bars.
    let warmup_duration = interval_duration * (warmup_bars + 1) as i32;
    let global_start = periods.iter().map(|p| p.start).min().unwrap()
        - Duration::days(COOLDOWN_WARMUP_DAYS)
        - warmup_duration;
    let global_end =
        periods.iter().map(|p| p.end).max().unwrap() + Duration::hours(DATA_BUFFER_HOURS);

    // === Phase 1: Load candle store ===
    let t_store = Instant::now();
    let mut store = CandleStore::new();
    let client = BinanceClient::new(MarketType::Futures);

    let mut failed_symbols: Vec<String> = Vec::new();
    for sym in &api_symbols {
        match ensure_candles(
            &mut store,
            &client,
            sym,
            &analysis_interval,
            global_start,
            global_end,
        ) {
            Ok(candles) if candles.is_empty() => {
                eprintln!("  ERROR: No {} data available for {sym}", analysis_interval);
                failed_symbols.push(sym.clone());
            }
            Err(e) => {
                eprintln!("  ERROR: {e}");
                failed_symbols.push(sym.clone());
            }
            Ok(_) => {}
        }
    }
    if !failed_symbols.is_empty() {
        eprintln!(
            "\nFATAL: Missing {} candle data for {} symbol(s): {}",
            analysis_interval,
            failed_symbols.len(),
            failed_symbols.join(", "),
        );
        eprintln!("Cannot produce valid research artifacts from incomplete data. Aborting.");
        process::exit(1);
    }
    println!(
        "{} candles loaded: {} candles in {:.0}ms",
        analysis_interval,
        store.mem_candle_count(),
        t_store.elapsed().as_secs_f64() * 1000.0,
    );

    // Additional interval candles (multi-timeframe)
    let iv_indicator_map = strategy.indicator_columns_per_interval();
    let earliest_period_start = periods.iter().map(|p| p.start).min().unwrap();
    ensure_additional_interval_candles(
        &mut store,
        &client,
        strategy,
        &api_symbols,
        &iv_indicator_map,
        &calib_config,
        earliest_period_start,
        global_end,
    );

    // === Phase 2: Generate signals ===
    let t_signals = Instant::now();
    let gen_result = generate_all_signals(
        strategy,
        &mut store,
        &client,
        &api_symbols,
        &windows,
        warmup_bars,
        &analysis_interval,
        interval_duration,
        global_start,
        global_end,
    );

    let all_window_trades = gen_result.window_signals;
    let calibration_intervals = gen_result.state.calibration_intervals;
    let total_signals: usize = all_window_trades.iter().map(|(_, s)| s.len()).sum();
    println!(
        "Signals generated: {} in {:.0}ms",
        total_signals,
        t_signals.elapsed().as_secs_f64() * 1000.0,
    );

    // === Phase 2.5: Pre-fetch 1h candles for resolution if analysis != 1h ===
    // The resolver's fallback tier always uses 1h candles (see MemoryProvider).
    // If the strategy uses a different interval, 1h data won't be in the store
    // from phase 1. Bulk-fetch now to avoid per-signal API calls.
    // When key_levels already ran, 1h data is already in the store and
    // ensure_candles will skip the fetch.
    //
    // `ensure_candles` is the authoritative coverage check; ignoring its
    // `Err` would let a coverage shortfall leak to the resolver, whose
    // `MemoryProvider` only serves what the store already contains (no
    // network fallback). Abort loudly instead of silently degrading.
    if analysis_interval != "1h" && total_signals > 0 {
        let t_1h = Instant::now();
        let mut failed_1h_syms: Vec<String> = Vec::new();
        for sym in &api_symbols {
            if let Err(e) = ensure_candles(&mut store, &client, sym, "1h", global_start, global_end) {
                eprintln!("  ERROR: 1h resolver prefetch failed for {sym}: {e}");
                failed_1h_syms.push(sym.clone());
            }
        }
        if !failed_1h_syms.is_empty() {
            eprintln!(
                "\nFATAL: 1h resolver prefetch failed for {} symbol(s): {}",
                failed_1h_syms.len(),
                failed_1h_syms.join(", "),
            );
            eprintln!("Cannot resolve trades without complete 1h fallback data. Aborting.");
            process::exit(1);
        }
        eprintln!(
            "1h resolver candles loaded in {:.0}ms",
            t_1h.elapsed().as_secs_f64() * 1000.0,
        );
    }

    // === Phase 3: Pre-fetch 1m candles ===
    let t_prefetch_1m = Instant::now();
    let mut minute_ranges: HashMap<String, (i64, i64)> = HashMap::new();

    for (_, signals) in &all_window_trades {
        for sig in signals {
            let sym = &sig.ticker;
            let sig_ms = sig.signal_date.timestamp_millis();
            let delay_ms = sig.entry_delay_seconds.unwrap_or(DEFAULT_ENTRY_DELAY) * 1000;
            let fill_ms = sig.fill_timeout_seconds * 1000;
            let hold_ms = sig.max_holding_hours * 3_600_000;
            let entry_start = sig_ms - 120_000;
            let exit_end = sig_ms + delay_ms + fill_ms + hold_ms + 60_000;

            let entry = minute_ranges
                .entry(sym.clone())
                .or_insert((entry_start, exit_end));
            entry.0 = entry.0.min(entry_start);
            entry.1 = entry.1.max(exit_end);
        }
    }

    // Parallel pre-load of 1m files into raw_cache (4 concurrent reads).
    let preload_syms: Vec<String> = minute_ranges.keys().cloned().collect();
    store.preload_1m(&preload_syms);

    let mut failed_1m_syms: Vec<String> = Vec::new();
    for (sym, (start_ms, end_ms)) in &minute_ranges {
        let need_start = claude_trader_resolver::ms_to_dt(*start_ms);
        let need_end = claude_trader_resolver::ms_to_dt(*end_ms);
        if let Err(e) = ensure_candles(&mut store, &client, sym, "1m", need_start, need_end) {
            eprintln!("  ERROR: 1m prefetch failed for {sym}: {e}");
            failed_1m_syms.push(sym.clone());
        }
    }
    if !failed_1m_syms.is_empty() {
        eprintln!(
            "\nFATAL: 1m prefetch failed for {} symbol(s): {}",
            failed_1m_syms.len(),
            failed_1m_syms.join(", "),
        );
        eprintln!("Cannot resolve trades without complete 1m data. Aborting.");
        process::exit(1);
    }
    println!(
        "1m candles loaded: {} symbols in {:.0}ms",
        minute_ranges.len(),
        t_prefetch_1m.elapsed().as_secs_f64() * 1000.0,
    );

    // === Phase 4: Resolve trades ===
    let t_resolve = Instant::now();
    let mut resolved_window_trades: Vec<(EvalWindow, Vec<TradeResult>)> = Vec::new();

    {
        let mut provider = MemoryProvider {
            store: &mut store,
            client: &client,
        };

        for (window, signals) in &all_window_trades {
            let mut trades = Vec::new();
            for signal in signals {
                let trade = kernel::backtest_signal(
                    signal,
                    &mut provider,
                    config.approximate,
                    None,
                    DEFAULT_ENTRY_DELAY,
                );
                trades.push(trade);
            }
            trades.sort_by_key(|t| t.entry_time);
            resolved_window_trades.push((window.clone(), trades));
        }
    }

    // All data fetched and resolved — release BinanceClient caches to free memory.
    client.clear_caches();

    let mut exact_count = 0usize;
    let mut entry_fb_count = 0usize;
    let mut exit_fb_count = 0usize;
    let mut random_count = 0usize;
    for (_, trades) in &resolved_window_trades {
        for t in trades {
            if t.exit_reason == claude_trader_models::ExitReason::Unfilled {
                continue;
            }
            if t.entry_fallback {
                entry_fb_count += 1;
            }
            if t.exit_fallback {
                exit_fb_count += 1;
            }
            if !t.entry_fallback && !t.exit_fallback {
                exact_count += 1;
            }
            if t.random_resolved {
                random_count += 1;
            }
        }
    }
    println!(
        "Resolved {} trades in {:.0}ms ({} exact | {} entry-fb | {} exit-fb | {} random)",
        total_signals,
        t_resolve.elapsed().as_secs_f64() * 1000.0,
        exact_count,
        entry_fb_count,
        exit_fb_count,
        random_count,
    );

    // === Phase 5: Build report and print ===
    let report =
        build_evaluation_report(resolved_window_trades, &portfolio_config, &api_symbols);
    let summaries = category_summaries(&report);
    let overall = compute_overall_summary(&report);

    println!();
    println!(
        "{:<40} | {:>5} | {:>9} | {:>6} | {:>8} | {:>8} | {:>6} | {:>9} | {:>6} | {:>5} | {:>7} | {:>6} | {:>6} | {:>7}",
        "Category", "Win", "PNL", "WR", "Worst", "Best", "Trades", "S/L", "Trd WR", "PF", "Sort", "DD", "Omega", "Pref",
    );
    println!("{}", "-".repeat(160));
    for cs in &summaries {
        print_row(cs);
    }
    print_row(&overall);

    let generalization = metrics::compute_generalization_score(&report.window_results);

    println!();
    println!(
        "Summary: PNL {:+.2}% | MDD {:.2}% | PF {:.2} | Sortino {:.2} | Pref {:.3} | Gen {:.3}",
        overall.total_pnl,
        overall.max_drawdown_pct,
        overall.profit_factor,
        overall.sortino_ratio,
        overall.preference_score,
        generalization.score,
    );
    println!(
        "Generalization: score {:.3} | CV {:.3} | mean/bucket {:+.2}% | std/bucket {:.2}% | {} buckets of {} windows",
        generalization.score,
        generalization.cv,
        generalization.mean_bucket_pnl,
        generalization.std_bucket_pnl,
        generalization.bucket_count,
        metrics::GENERALIZATION_BUCKET_WINDOWS,
    );
    println!("Eligible: {}", overall.preference_eligible);
    println!("Total time: {:.2}s", t_total.elapsed().as_secs_f64());

    // === Phase 6: Save outputs ===
    let result = EvaluationResult {
        strategy_name: strategy.name().to_string(),
        strategy_description: strategy.description(),
        window_set_label: config.window_set.label().to_string(),
        approximate: config.approximate,
        summaries,
        overall,
        report,
        symbols: api_symbols,
        calibration_intervals,
    };

    // Dump report for parity verification if requested
    if let Some(ref path) = config.dump_trades {
        let json = serde_json::to_string_pretty(&result.report).expect("serialize report");
        std::fs::write(path, &json).expect("write dump file");
        eprintln!("Parity dump written to {path}");
    }

    if let Err(e) = save_outputs(&result) {
        eprintln!("WARNING: Failed to save outputs: {e}");
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    /// Minimal strategy for testing interval validation.
    struct TestStrategy {
        interval: &'static str,
        custom_mdr: Option<MarketDataRequest>,
    }

    impl TestStrategy {
        fn new(interval: &'static str) -> Self {
            Self {
                interval,
                custom_mdr: None,
            }
        }

        fn with_mismatched_mdr(interval: &'static str, mdr_interval: &str) -> Self {
            Self {
                interval,
                custom_mdr: Some(MarketDataRequest::ohlcv_only(mdr_interval)),
            }
        }
    }

    impl ResearchStrategy for TestStrategy {
        fn name(&self) -> &str {
            "test"
        }
        fn symbols(&self) -> Vec<String> {
            vec!["BTCUSDT".into()]
        }
        fn indicator_columns(&self) -> &[&str] {
            &[]
        }
        fn cooldown_spec(&self, signal: &Signal) -> CooldownSpec {
            CooldownSpec::symbol_side(signal, 0.0)
        }

        fn analysis_interval(&self) -> &str {
            self.interval
        }

        fn market_data_request(&self) -> MarketDataRequest {
            match &self.custom_mdr {
                Some(mdr) => mdr.clone(),
                None => MarketDataRequest::ohlcv_only(self.analysis_interval()),
            }
        }

        fn generate_signals(
            &self,
            _candles: &BTreeMap<String, &[Candle]>,
            _start: DateTime<Utc>,
            _end: DateTime<Utc>,
            _active_params: &HashMap<String, serde_json::Value>,
            _ctx: &ContextMap,
            _htf: &HtfData,
        ) -> Vec<Signal> {
            vec![]
        }
    }

    #[test]
    fn test_validate_strategy_interval_1h() {
        let s = TestStrategy::new("1h");
        let (interval, duration) = validate_strategy_interval(&s);
        assert_eq!(interval, "1h");
        assert_eq!(duration, Duration::hours(1));
    }

    #[test]
    fn test_validate_strategy_interval_4h() {
        let s = TestStrategy::new("4h");
        let (interval, duration) = validate_strategy_interval(&s);
        assert_eq!(interval, "4h");
        assert_eq!(duration, Duration::hours(4));
    }

    #[test]
    fn test_validate_strategy_interval_15m() {
        let s = TestStrategy::new("15m");
        let (interval, duration) = validate_strategy_interval(&s);
        assert_eq!(interval, "15m");
        assert_eq!(duration, Duration::minutes(15));
    }

    #[test]
    fn test_calibration_bars_1h() {
        // 72 lookback hours / 1h interval = 72 bars
        let interval_secs = 3600i64;
        let lookback_hours = 72u32;
        let bars = ((lookback_hours as f64) * 3600.0 / interval_secs as f64).ceil() as usize;
        assert_eq!(bars, 72);
    }

    #[test]
    fn test_calibration_bars_4h() {
        // 72 lookback hours / 4h interval = 18 bars
        let interval_secs = 14400i64;
        let lookback_hours = 72u32;
        let bars = ((lookback_hours as f64) * 3600.0 / interval_secs as f64).ceil() as usize;
        assert_eq!(bars, 18);
    }

    #[test]
    fn test_calibration_bars_15m() {
        // 72 lookback hours / 0.25h interval = 288 bars
        let interval_secs = 900i64;
        let lookback_hours = 72u32;
        let bars = ((lookback_hours as f64) * 3600.0 / interval_secs as f64).ceil() as usize;
        assert_eq!(bars, 288);
    }

    #[test]
    fn test_calibration_bars_non_divisible() {
        // 100 lookback hours / 4h interval = 25 bars (100/4 = 25, exact)
        let interval_secs = 14400i64;
        let lookback_hours = 100u32;
        let bars = ((lookback_hours as f64) * 3600.0 / interval_secs as f64).ceil() as usize;
        assert_eq!(bars, 25);

        // 101 lookback hours / 4h interval = 26 bars (ceil(101/4) = 26)
        let lookback_hours = 101u32;
        let bars = ((lookback_hours as f64) * 3600.0 / interval_secs as f64).ceil() as usize;
        assert_eq!(bars, 26);
    }

    #[test]
    fn test_memory_provider_exact_coverage() {
        use claude_trader_data::CandleStore;

        // Use a unique test symbol to avoid collisions with real data on disk.
        let sym = "UT_MEMPROV_EXACT_COV";

        let mut store = CandleStore::new();
        let candles = vec![
            Candle {
                open_time: Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(),
                close_time: Utc.with_ymd_and_hms(2024, 1, 1, 4, 0, 0).unwrap(),
                open: 100.0,
                high: 105.0,
                low: 95.0,
                close: 102.0,
                volume: 1000.0,
                taker_buy_volume: 500.0,
            },
            Candle {
                open_time: Utc.with_ymd_and_hms(2024, 1, 1, 4, 0, 0).unwrap(),
                close_time: Utc.with_ymd_and_hms(2024, 1, 1, 8, 0, 0).unwrap(),
                open: 102.0,
                high: 108.0,
                low: 100.0,
                close: 106.0,
                volume: 1200.0,
                taker_buy_volume: 600.0,
            },
        ];
        store.put(sym, "4h", &candles);

        // Request exactly [00:00, 08:00) — should return cached
        let result = store.get_range(
            sym,
            "4h",
            Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2024, 1, 1, 8, 0, 0).unwrap(),
        );
        assert_eq!(result.len(), 2);

        // Request [00:00, 12:00) — only has up to 08:00, should return
        // partial (exact coverage check in MemoryProvider would trigger refetch,
        // but CandleStore.get_range still returns what it has)
        let result = store.get_range(
            sym,
            "4h",
            Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2024, 1, 1, 12, 0, 0).unwrap(),
        );
        assert_eq!(result.len(), 2); // store returns what it has
    }

    #[test]
    fn test_default_mdr_uses_analysis_interval() {
        let s = TestStrategy::new("4h");
        let mdr = s.market_data_request();
        assert_eq!(mdr.ohlcv_interval, "4h");
    }

    #[test]
    fn test_mismatch_analysis_interval_vs_mdr_fails() {
        let s = TestStrategy::with_mismatched_mdr("4h", "1h");
        let result = check_strategy_interval(&s);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("4h"), "error should mention 4h: {err}");
        assert!(err.contains("1h"), "error should mention 1h: {err}");
    }

    #[test]
    fn test_matching_custom_mdr_passes() {
        let s = TestStrategy::with_mismatched_mdr("4h", "4h");
        let result = check_strategy_interval(&s);
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_analysis_interval_fails() {
        let s = TestStrategy::new("xyz");
        let result = check_strategy_interval(&s);
        assert!(result.is_err());
    }

    // -----------------------------------------------------------------------
    // Smoke test helpers
    // -----------------------------------------------------------------------

    /// Generate synthetic candles at a given interval between start and end.
    fn make_candles(interval_str: &str, start: DateTime<Utc>, end: DateTime<Utc>) -> Vec<Candle> {
        let interval = parse_interval_duration(interval_str).unwrap();
        let mut candles = Vec::new();
        let mut t = start;
        let mut price = 100.0;
        while t < end {
            price += 0.1;
            candles.push(Candle {
                open_time: t,
                close_time: t + interval,
                open: price,
                high: price + 1.0,
                low: price - 1.0,
                close: price + 0.5,
                volume: 1000.0,
                taker_buy_volume: 500.0,
            });
            t = t + interval;
        }
        candles
    }

    /// Strategy that emits one signal per symbol at the first valid candle.
    struct SignalEmittingStrategy {
        interval: &'static str,
    }

    const SMOKE_TEST_SYMBOL: &str = "UT_SMOKE_TEST";

    impl ResearchStrategy for SignalEmittingStrategy {
        fn name(&self) -> &str {
            "smoke_test"
        }
        fn symbols(&self) -> Vec<String> {
            vec![SMOKE_TEST_SYMBOL.into()]
        }
        fn indicator_columns(&self) -> &[&str] {
            &[]
        }
        fn cooldown_spec(&self, signal: &Signal) -> CooldownSpec {
            CooldownSpec::symbol_side(signal, 0.0)
        }
        fn extra_warmup_bars(&self) -> usize {
            10
        }

        fn analysis_interval(&self) -> &str {
            self.interval
        }

        fn market_data_request(&self) -> MarketDataRequest {
            MarketDataRequest::ohlcv_only(self.analysis_interval())
        }

        fn generate_signals(
            &self,
            candles: &BTreeMap<String, &[Candle]>,
            start: DateTime<Utc>,
            end: DateTime<Utc>,
            _active_params: &HashMap<String, serde_json::Value>,
            _ctx: &ContextMap,
            _htf: &HtfData,
        ) -> Vec<Signal> {
            let mut signals = Vec::new();
            for (symbol, &bars) in candles {
                // Verify we got warmup bars before start.
                // The candle whose close_time == start is the first in-window
                // candle, not a warmup bar, so expect warmup_bars - 1.
                let pre_start = bars.iter().filter(|c| c.close_time < start).count();
                assert!(
                    pre_start >= 9,
                    "expected >= 9 warmup bars for {symbol}, got {pre_start} \
                     (interval={}, total bars={})",
                    self.interval,
                    bars.len(),
                );

                // Emit signals for every bar in [start, end) so that at
                // least one falls inside the eval window after the cooldown
                // warmup period is sliced away.
                for bar in bars {
                    if bar.close_time >= start && bar.close_time < end {
                        signals.push(Signal {
                            signal_date: bar.close_time,
                            position_type: claude_trader_models::PositionType::Long,
                            ticker: symbol.clone(),
                            pattern: "sanity_check".to_string(),
                            tp_pct: Some(2.0),
                            sl_pct: Some(1.0),
                            tp_price: None,
                            sl_price: None,
                            leverage: 1.0,
                            market_type: claude_trader_models::MarketType::Futures,
                            taker_fee_rate: 0.0005,
                            entry_price: None,
                            fill_timeout_seconds: 3600,
                            entry_delay_seconds: None,
                            max_holding_hours: 24,
                            size_multiplier: 1.0,
                            metadata: HashMap::new(),
                        });
                    }
                }
            }
            signals
        }
    }

    /// Run `generate_all_signals` with synthetic data at the given interval.
    /// Returns the number of signals generated.
    fn run_smoke_test(interval: &'static str) -> usize {
        let strategy = SignalEmittingStrategy { interval };
        let (analysis_interval, interval_duration) = check_strategy_interval(&strategy).unwrap();

        // Use a single small test window
        let window = EvalWindow {
            name: "SMOKE".into(),
            category: "test".into(),
            start: Utc.with_ymd_and_hms(2023, 1, 15, 0, 0, 0).unwrap(),
            end: Utc.with_ymd_and_hms(2023, 1, 22, 0, 0, 0).unwrap(),
        };
        let windows = vec![window];

        let warmup_bars = claude_trader_indicators::required_warmup(strategy.indicator_columns())
            + strategy.extra_warmup_bars();
        let warmup_dur = interval_duration * warmup_bars as i32;

        let periods = claude_trader_evaluator::windows::group_into_periods(
            &windows,
            Duration::days(COOLDOWN_WARMUP_DAYS),
        );
        let global_start = periods.iter().map(|p| p.start).min().unwrap()
            - Duration::days(COOLDOWN_WARMUP_DAYS)
            - warmup_dur;
        let global_end =
            periods.iter().map(|p| p.end).max().unwrap() + Duration::hours(DATA_BUFFER_HOURS);

        // Populate store with synthetic candles
        let mut store = CandleStore::new();
        let candles = make_candles(&analysis_interval, global_start, global_end);
        assert!(
            !candles.is_empty(),
            "make_candles produced 0 candles for interval {interval}"
        );
        store.put(SMOKE_TEST_SYMBOL, &analysis_interval, &candles);

        // BinanceClient won't be called — store is fully populated
        let client = BinanceClient::new(claude_trader_models::MarketType::Futures);

        let result = generate_all_signals(
            &strategy,
            &mut store,
            &client,
            &[SMOKE_TEST_SYMBOL.into()],
            &windows,
            warmup_bars,
            &analysis_interval,
            interval_duration,
            global_start,
            global_end,
        );

        let total: usize = result.window_signals.iter().map(|(_, s)| s.len()).sum();
        total
    }

    #[test]
    fn smoke_test_4h_signal_generation() {
        let n = run_smoke_test("4h");
        assert!(
            n > 0,
            "4h strategy should produce at least 1 signal, got {n}"
        );
    }

    #[test]
    fn smoke_test_15m_signal_generation() {
        let n = run_smoke_test("15m");
        assert!(
            n > 0,
            "15m strategy should produce at least 1 signal, got {n}"
        );
    }

    #[test]
    fn smoke_test_1h_signal_generation() {
        let n = run_smoke_test("1h");
        assert!(
            n > 0,
            "1h strategy should produce at least 1 signal, got {n}"
        );
    }

    // -----------------------------------------------------------------------
    // Global cooldown across calibration/period boundaries
    // -----------------------------------------------------------------------

    /// Variant of `SignalEmittingStrategy` with configurable cooldown hours —
    /// used to exercise the runtime's global cooldown filter. Signals are
    /// emitted only at bars that fall inside one of the strategy's windows
    /// so we can reason cleanly about post-distribution counts.
    struct CooldownStrategy {
        interval: &'static str,
        hours: f64,
        windows: Vec<(DateTime<Utc>, DateTime<Utc>)>,
    }

    impl ResearchStrategy for CooldownStrategy {
        fn name(&self) -> &str { "cooldown_test" }
        fn symbols(&self) -> Vec<String> { vec![SMOKE_TEST_SYMBOL.into()] }
        fn indicator_columns(&self) -> &[&str] { &[] }
        fn cooldown_spec(&self, signal: &Signal) -> CooldownSpec {
            CooldownSpec::symbol_side(signal, self.hours)
        }
        fn extra_warmup_bars(&self) -> usize { 10 }
        fn analysis_interval(&self) -> &str { self.interval }
        fn market_data_request(&self) -> MarketDataRequest {
            MarketDataRequest::ohlcv_only(self.analysis_interval())
        }
        fn generate_signals(
            &self,
            candles: &BTreeMap<String, &[Candle]>,
            _start: DateTime<Utc>,
            _end: DateTime<Utc>,
            _active_params: &HashMap<String, serde_json::Value>,
            _ctx: &ContextMap,
            _htf: &HtfData,
        ) -> Vec<Signal> {
            let mut signals = Vec::new();
            for (symbol, &bars) in candles {
                for bar in bars {
                    let in_any_window = self
                        .windows
                        .iter()
                        .any(|(s, e)| bar.close_time >= *s && bar.close_time < *e);
                    if !in_any_window {
                        continue;
                    }
                    signals.push(Signal {
                        signal_date: bar.close_time,
                        position_type: claude_trader_models::PositionType::Long,
                        ticker: symbol.clone(),
                        pattern: "cooldown_test".into(),
                        tp_pct: Some(2.0),
                        sl_pct: Some(1.0),
                        tp_price: None, sl_price: None,
                        leverage: 1.0,
                        market_type: MarketType::Futures,
                        taker_fee_rate: 0.0005,
                        entry_price: None,
                        fill_timeout_seconds: 3600,
                        entry_delay_seconds: None,
                        max_holding_hours: 24,
                        size_multiplier: 1.0,
                        metadata: HashMap::new(),
                    });
                }
            }
            signals
        }
    }

    fn run_cooldown_scenario(hours: f64) -> usize {
        let interval = "1h";

        // 30-day gap → two distinct periods (COOLDOWN_WARMUP_DAYS = 14).
        let window_a = EvalWindow {
            name: "W_A".into(), category: "test".into(),
            start: Utc.with_ymd_and_hms(2023, 1, 15, 0, 0, 0).unwrap(),
            end:   Utc.with_ymd_and_hms(2023, 1, 22, 0, 0, 0).unwrap(),
        };
        let window_b = EvalWindow {
            name: "W_B".into(), category: "test".into(),
            start: Utc.with_ymd_and_hms(2023, 2, 22, 0, 0, 0).unwrap(),
            end:   Utc.with_ymd_and_hms(2023, 3,  1, 0, 0, 0).unwrap(),
        };
        let strategy = CooldownStrategy {
            interval,
            hours,
            windows: vec![
                (window_a.start, window_a.end),
                (window_b.start, window_b.end),
            ],
        };

        let (analysis_interval, interval_duration) =
            check_strategy_interval(&strategy).unwrap();

        let windows = vec![window_a, window_b];
        let warmup_bars =
            claude_trader_indicators::required_warmup(strategy.indicator_columns())
                + strategy.extra_warmup_bars();
        let warmup_dur = interval_duration * warmup_bars as i32;

        let periods = claude_trader_evaluator::windows::group_into_periods(
            &windows, Duration::days(COOLDOWN_WARMUP_DAYS),
        );
        assert!(periods.len() >= 2, "windows must span multiple periods");

        let global_start = periods.iter().map(|p| p.start).min().unwrap()
            - Duration::days(COOLDOWN_WARMUP_DAYS)
            - warmup_dur;
        let global_end = periods.iter().map(|p| p.end).max().unwrap()
            + Duration::hours(DATA_BUFFER_HOURS);

        let mut store = CandleStore::new();
        let candles = make_candles(&analysis_interval, global_start, global_end);
        store.put(SMOKE_TEST_SYMBOL, &analysis_interval, &candles);
        let client = BinanceClient::new(MarketType::Futures);

        let result = generate_all_signals(
            &strategy, &mut store, &client,
            &[SMOKE_TEST_SYMBOL.into()], &windows,
            warmup_bars, &analysis_interval, interval_duration,
            global_start, global_end,
        );
        result.window_signals.iter().map(|(_, s)| s.len()).sum()
    }

    /// Sanity: with zero cooldown, every in-window bar emits a signal.
    #[test]
    fn no_cooldown_emits_all_in_window_bars() {
        let n = run_cooldown_scenario(0.0);
        // Two 7-day windows × 24 hourly bars = 336. Allow ≥300 to avoid
        // coupling to make_candles boundary behavior.
        assert!(n >= 300, "expected ≥300 signals with 0h cooldown, got {n}");
    }

    /// Two windows spaced wider than COOLDOWN_WARMUP_DAYS land in separate
    /// periods. With a cooldown wider than the entire run, exactly **one**
    /// signal may survive globally — per-period filtering would give one per
    /// period (two total). This pins the global-across-boundaries semantics.
    #[test]
    fn cooldown_applies_globally_across_period_boundaries() {
        let n = run_cooldown_scenario(24.0 * 365.0);
        assert_eq!(
            n, 1,
            "global cooldown must admit exactly one signal across all periods, got {n}"
        );
    }

    // -----------------------------------------------------------------------
    // check_additional_intervals tests
    // -----------------------------------------------------------------------

    /// Minimal strategy stub for testing interval validation.
    struct IntervalTestStrategy {
        analysis: &'static str,
        additional: Vec<&'static str>,
    }
    impl ResearchStrategy for IntervalTestStrategy {
        fn name(&self) -> &str { "test" }
        fn symbols(&self) -> Vec<String> { vec!["BTCUSDT".into()] }
        fn indicator_columns(&self) -> &[&str] { &[] }
        fn cooldown_spec(&self, signal: &Signal) -> CooldownSpec {
            CooldownSpec::symbol_side(signal, 24.0)
        }
        fn analysis_interval(&self) -> &str { self.analysis }
        fn additional_intervals(&self) -> Vec<&str> { self.additional.clone() }
        fn generate_signals(&self, _: &BTreeMap<String, &[Candle]>,
            _: DateTime<Utc>, _: DateTime<Utc>,
            _: &HashMap<String, serde_json::Value>,
            _: &ContextMap,
            _: &HtfData,
        ) -> Vec<Signal> { vec![] }
    }

    #[test]
    fn test_additional_intervals_rejects_1m() {
        let s = IntervalTestStrategy { analysis: "4h", additional: vec!["1m"] };
        let result = check_additional_intervals(&s);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("1m cannot be used"));
    }

    #[test]
    fn test_additional_intervals_rejects_self_reference() {
        let s = IntervalTestStrategy { analysis: "4h", additional: vec!["4h"] };
        let result = check_additional_intervals(&s);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("already the analysis_interval"));
    }

    #[test]
    fn test_additional_intervals_rejects_duplicates() {
        let s = IntervalTestStrategy { analysis: "4h", additional: vec!["1h", "1h"] };
        let result = check_additional_intervals(&s);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("duplicate"));
    }

    #[test]
    fn test_additional_intervals_rejects_invalid() {
        let s = IntervalTestStrategy { analysis: "4h", additional: vec!["3x"] };
        let result = check_additional_intervals(&s);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("invalid"));
    }

    #[test]
    fn test_additional_intervals_accepts_valid() {
        let s = IntervalTestStrategy { analysis: "4h", additional: vec!["1h", "15m"] };
        let result = check_additional_intervals(&s);
        assert!(result.is_ok());
    }

    #[test]
    fn test_additional_intervals_accepts_empty() {
        let s = IntervalTestStrategy { analysis: "4h", additional: vec![] };
        let result = check_additional_intervals(&s);
        assert!(result.is_ok());
    }

    /// Strategy stub that returns a bad indicator_columns_per_interval key.
    struct BadIndicatorIntervalStrategy;
    impl ResearchStrategy for BadIndicatorIntervalStrategy {
        fn name(&self) -> &str { "bad" }
        fn symbols(&self) -> Vec<String> { vec!["BTCUSDT".into()] }
        fn indicator_columns(&self) -> &[&str] { &["atr_14"] }
        fn cooldown_spec(&self, signal: &Signal) -> CooldownSpec {
            CooldownSpec::symbol_side(signal, 24.0)
        }
        fn analysis_interval(&self) -> &str { "1h" }
        fn additional_intervals(&self) -> Vec<&str> { vec!["4h"] }
        fn indicator_columns_per_interval(&self) -> HashMap<&str, Vec<&str>> {
            // "15m" is NOT in additional_intervals — should fail validation
            let mut m = HashMap::new();
            m.insert("4h", vec!["atr_14"]);
            m.insert("15m", vec!["atr_14"]);
            m
        }
        fn generate_signals(&self, _: &BTreeMap<String, &[Candle]>,
            _: DateTime<Utc>, _: DateTime<Utc>,
            _: &HashMap<String, serde_json::Value>,
            _: &ContextMap,
            _: &HtfData,
        ) -> Vec<Signal> { vec![] }
    }

    #[test]
    fn test_indicator_columns_per_interval_rejects_unknown_key() {
        let s = BadIndicatorIntervalStrategy;
        let result = check_additional_intervals(&s);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("15m"), "error should mention the bad key: {err}");
        assert!(
            err.contains("not in additional_intervals"),
            "error should explain the problem: {err}"
        );
    }

    // -----------------------------------------------------------------
    // ensure_funding_rates_with_fetcher — exercises coverage-driven
    // funding without a network client.
    // -----------------------------------------------------------------

    use std::cell::RefCell;
    use std::rc::Rc;

    fn rate_at(ts: DateTime<Utc>, v: f64) -> FundingRate {
        FundingRate {
            timestamp: ts,
            funding_rate: v,
            mark_price: None,
        }
    }

    /// Fetcher that records every `(start, end)` it was invoked with and
    /// returns a scripted response per call. Tests can inspect the log
    /// to prove the coverage layer suppressed or issued the expected
    /// calls.
    struct LoggingFetcher {
        calls: Rc<RefCell<Vec<(DateTime<Utc>, DateTime<Utc>)>>>,
        script: Rc<RefCell<std::vec::IntoIter<Result<Vec<FundingRate>, DataError>>>>,
    }

    impl LoggingFetcher {
        fn new(script: Vec<Result<Vec<FundingRate>, DataError>>) -> Self {
            Self {
                calls: Rc::new(RefCell::new(Vec::new())),
                script: Rc::new(RefCell::new(script.into_iter())),
            }
        }

        fn closure(
            &self,
        ) -> impl FnMut(DateTime<Utc>, DateTime<Utc>) -> Result<Vec<FundingRate>, DataError> + '_
        {
            let calls = self.calls.clone();
            let script = self.script.clone();
            move |s, e| {
                calls.borrow_mut().push((s, e));
                script
                    .borrow_mut()
                    .next()
                    .expect("fetcher called past end of script")
            }
        }
    }

    /// Pick a symbol unique per test to avoid cross-test collisions in
    /// the shared `~/.claude_trader` store directory.
    fn fresh_funding_symbol(tag: &str) -> String {
        format!("UT_ENSURE_FUND_{tag}")
    }

    /// Phase 3 headline: a probe that returned nothing must still record
    /// coverage, so the next run reports no gaps and issues no fetch.
    #[test]
    fn ensure_funding_records_empty_probe_and_suppresses_refetch() {
        let sym = fresh_funding_symbol("EMPTY_TAIL");
        let mut store = CandleStore::new();
        store.invalidate_funding(&sym);

        let start = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2024, 1, 1, 12, 0, 0).unwrap();

        let fetcher1 = LoggingFetcher::new(vec![Ok(vec![])]);
        let calls1 = fetcher1.calls.clone();
        let rates = ensure_funding_rates_with_fetcher(
            &mut store,
            fetcher1.closure(),
            &sym,
            start,
            end,
        )
        .expect("first call must succeed");
        assert!(rates.is_empty(), "store holds no rates after empty fetch");
        assert_eq!(calls1.borrow().len(), 1, "first call probed once");

        // Second run with same range must NOT call the fetcher —
        // coverage now covers [start, end) even though no rows exist.
        let fetcher2 = LoggingFetcher::new(vec![]);
        let calls2 = fetcher2.calls.clone();
        let rates = ensure_funding_rates_with_fetcher(
            &mut store,
            fetcher2.closure(),
            &sym,
            start,
            end,
        )
        .expect("second call must succeed without fetching");
        assert!(rates.is_empty());
        assert!(
            calls2.borrow().is_empty(),
            "coverage must suppress re-fetch; got calls {:?}",
            calls2.borrow()
        );

        store.invalidate_funding(&sym);
    }

    /// Successful fetch writes both data and coverage. A subsequent call
    /// over the same window returns the stored rows with zero fetches.
    #[test]
    fn ensure_funding_persists_rows_and_coverage() {
        let sym = fresh_funding_symbol("HAPPY_PATH");
        let mut store = CandleStore::new();
        store.invalidate_funding(&sym);

        let start = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2024, 1, 2, 0, 0, 0).unwrap();
        let rows = vec![
            rate_at(start, 0.0001),
            rate_at(start + Duration::hours(8), 0.0002),
            rate_at(start + Duration::hours(16), 0.0003),
        ];

        let fetcher1 = LoggingFetcher::new(vec![Ok(rows.clone())]);
        let got = ensure_funding_rates_with_fetcher(
            &mut store,
            fetcher1.closure(),
            &sym,
            start,
            end,
        )
        .expect("first call must succeed");
        assert_eq!(got.len(), 3);

        let fetcher2 = LoggingFetcher::new(vec![]);
        let calls2 = fetcher2.calls.clone();
        let got = ensure_funding_rates_with_fetcher(
            &mut store,
            fetcher2.closure(),
            &sym,
            start,
            end,
        )
        .expect("second call must succeed without fetching");
        assert_eq!(got.len(), 3);
        assert!(calls2.borrow().is_empty(), "coverage must suppress re-fetch");

        store.invalidate_funding(&sym);
    }

    /// A hard fetch error bails immediately without recording coverage
    /// for the failed gap. The next run must re-attempt it.
    #[test]
    fn ensure_funding_error_leaves_coverage_unchanged() {
        let sym = fresh_funding_symbol("HARD_ERROR");
        let mut store = CandleStore::new();
        store.invalidate_funding(&sym);

        let start = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2024, 1, 1, 12, 0, 0).unwrap();

        let fetcher1 =
            LoggingFetcher::new(vec![Err(DataError::Http("connection reset".into()))]);
        let result = ensure_funding_rates_with_fetcher(
            &mut store,
            fetcher1.closure(),
            &sym,
            start,
            end,
        );
        assert!(result.is_err(), "hard error must propagate");

        // Next run must retry the same range — no coverage was recorded
        // for the failed gap.
        let fetcher2 = LoggingFetcher::new(vec![Ok(vec![rate_at(start, 0.001)])]);
        let calls2 = fetcher2.calls.clone();
        let got = ensure_funding_rates_with_fetcher(
            &mut store,
            fetcher2.closure(),
            &sym,
            start,
            end,
        )
        .expect("retry must succeed");
        assert_eq!(got.len(), 1);
        assert_eq!(
            calls2.borrow().len(),
            1,
            "post-error retry must re-probe the same range"
        );

        store.invalidate_funding(&sym);
    }

    /// When ensure is called before any data exists and coverage
    /// synthesized from first-ever fetch includes only the fetched
    /// range, a later extended query produces a new trailing gap and
    /// only that gap is fetched.
    #[test]
    fn ensure_funding_only_fetches_uncovered_suffix_on_extension() {
        let sym = fresh_funding_symbol("EXTENSION");
        let mut store = CandleStore::new();
        store.invalidate_funding(&sym);

        let start = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        let mid = Utc.with_ymd_and_hms(2024, 1, 1, 12, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2024, 1, 2, 0, 0, 0).unwrap();

        // First fetch covers [start, mid).
        let fetcher1 = LoggingFetcher::new(vec![Ok(vec![
            rate_at(start, 0.0001),
            rate_at(start + Duration::hours(8), 0.0002),
        ])]);
        let _ = ensure_funding_rates_with_fetcher(
            &mut store,
            fetcher1.closure(),
            &sym,
            start,
            mid,
        )
        .expect("first fetch");

        // Extended fetch must probe only [mid, end).
        let fetcher2 = LoggingFetcher::new(vec![Ok(vec![rate_at(mid, 0.0003)])]);
        let calls2 = fetcher2.calls.clone();
        let _ = ensure_funding_rates_with_fetcher(
            &mut store,
            fetcher2.closure(),
            &sym,
            start,
            end,
        )
        .expect("second fetch");

        let calls = calls2.borrow();
        assert_eq!(
            calls.len(),
            1,
            "must probe only the uncovered suffix, not the full range"
        );
        assert_eq!(
            calls[0].0, mid,
            "probe must start at the boundary of prior coverage"
        );
        assert_eq!(calls[0].1, end);

        store.invalidate_funding(&sym);
    }

    // -----------------------------------------------------------------
    // ensure_candles_with_fetcher — exercises the coverage-driven
    // candle refactor (Phase 4) without a network client.
    // -----------------------------------------------------------------

    fn mock_candle(ts: DateTime<Utc>) -> Candle {
        Candle {
            open_time: ts,
            close_time: ts + Duration::hours(1),
            open: 1.0,
            high: 1.0,
            low: 1.0,
            close: 1.0,
            volume: 0.0,
            taker_buy_volume: 0.0,
        }
    }

    /// Fetcher that records every `(start, end)` it was invoked with and
    /// returns a scripted `Result<KlineFetch, DataError>` per call.
    struct LoggingKlineFetcher {
        calls: Rc<RefCell<Vec<(DateTime<Utc>, DateTime<Utc>)>>>,
        script: Rc<RefCell<std::vec::IntoIter<Result<KlineFetch, DataError>>>>,
    }

    impl LoggingKlineFetcher {
        fn new(script: Vec<Result<KlineFetch, DataError>>) -> Self {
            Self {
                calls: Rc::new(RefCell::new(Vec::new())),
                script: Rc::new(RefCell::new(script.into_iter())),
            }
        }

        fn closure(
            &self,
        ) -> impl FnMut(DateTime<Utc>, DateTime<Utc>) -> Result<KlineFetch, DataError> + '_ {
            let calls = self.calls.clone();
            let script = self.script.clone();
            move |s, e| {
                calls.borrow_mut().push((s, e));
                script
                    .borrow_mut()
                    .next()
                    .expect("fetcher called past end of script")
            }
        }
    }

    fn fresh_candle_symbol(tag: &str) -> String {
        format!("UT_ENSURE_CANDLES_{tag}")
    }

    /// An empty `Complete` response records coverage, so a follow-up
    /// ensure over the same range issues zero fetches. Parity with the
    /// funding path's empty-probe contract for pre-listing windows.
    #[test]
    fn ensure_candles_records_empty_complete_and_suppresses_refetch() {
        let sym = fresh_candle_symbol("EMPTY_COMPLETE");
        let mut store = CandleStore::new();
        store.invalidate_candle(&sym, "1h");

        let start = Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2020, 1, 2, 0, 0, 0).unwrap();

        let fetcher1 = LoggingKlineFetcher::new(vec![Ok(KlineFetch::Complete(vec![]))]);
        let calls1 = fetcher1.calls.clone();
        let got = ensure_candles_with_fetcher(
            &mut store,
            fetcher1.closure(),
            &sym,
            "1h",
            start,
            end,
        )
        .expect("first ensure must succeed");
        assert!(got.is_empty(), "no rows came back");
        assert_eq!(calls1.borrow().len(), 1, "first call probed once");

        let fetcher2 = LoggingKlineFetcher::new(vec![]);
        let calls2 = fetcher2.calls.clone();
        let got = ensure_candles_with_fetcher(
            &mut store,
            fetcher2.closure(),
            &sym,
            "1h",
            start,
            end,
        )
        .expect("second ensure must succeed without fetching");
        assert!(got.is_empty());
        assert!(
            calls2.borrow().is_empty(),
            "coverage must suppress re-fetch of empty Complete"
        );

        store.invalidate_candle(&sym, "1h");
    }

    /// Non-contiguous backfill: two separate `ensure_candles` runs cover
    /// `[A, B)` and `[C, D)` with a real gap between them. A third run
    /// over `[A, D)` must fetch **only** the middle gap — not the full
    /// range. This is the headline bug Phase 2+4 exist to fix.
    #[test]
    fn ensure_candles_non_contiguous_backfill_fills_only_middle() {
        let sym = fresh_candle_symbol("NONCONTIG");
        let mut store = CandleStore::new();
        store.invalidate_candle(&sym, "1h");

        let t0 = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        let t3 = t0 + Duration::hours(3);
        let t6 = t0 + Duration::hours(6);
        let t9 = t0 + Duration::hours(9);

        // First window: fetch [t0, t3).
        let first_rows: Vec<Candle> =
            (0..3).map(|i| mock_candle(t0 + Duration::hours(i))).collect();
        let f1 = LoggingKlineFetcher::new(vec![Ok(KlineFetch::Complete(first_rows.clone()))]);
        let _ = ensure_candles_with_fetcher(&mut store, f1.closure(), &sym, "1h", t0, t3)
            .expect("first fetch");

        // Second window: fetch [t6, t9). Leaves a hole at [t3, t6).
        let second_rows: Vec<Candle> =
            (0..3).map(|i| mock_candle(t6 + Duration::hours(i))).collect();
        let f2 = LoggingKlineFetcher::new(vec![Ok(KlineFetch::Complete(second_rows.clone()))]);
        let _ = ensure_candles_with_fetcher(&mut store, f2.closure(), &sym, "1h", t6, t9)
            .expect("second fetch");

        // Third run over the full range must hit ONLY [t3, t6).
        let middle_rows: Vec<Candle> =
            (0..3).map(|i| mock_candle(t3 + Duration::hours(i))).collect();
        let f3 = LoggingKlineFetcher::new(vec![Ok(KlineFetch::Complete(middle_rows))]);
        let calls3 = f3.calls.clone();
        let _ = ensure_candles_with_fetcher(&mut store, f3.closure(), &sym, "1h", t0, t9)
            .expect("third fetch");

        let calls = calls3.borrow();
        assert_eq!(
            calls.len(),
            1,
            "must fetch exactly one gap (the middle); got {calls:?}"
        );
        assert_eq!(calls[0].0, t3, "middle gap starts at t3");
        assert_eq!(calls[0].1, t6, "middle gap ends at t6");

        store.invalidate_candle(&sym, "1h");
    }

    /// `KlineFetch::Interrupted` leaves the uncovered suffix as a fresh
    /// gap. The outer loop retries it on the next attempt. Successful
    /// retry → Ok, and all rows from both attempts are persisted.
    #[test]
    fn ensure_candles_interrupted_then_complete_succeeds() {
        let sym = fresh_candle_symbol("INTERRUPTED_OK");
        let mut store = CandleStore::new();
        store.invalidate_candle(&sym, "1h");

        let t0 = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        let t2 = t0 + Duration::hours(2);
        let t4 = t0 + Duration::hours(4);

        let first_rows = vec![mock_candle(t0), mock_candle(t0 + Duration::hours(1))];
        let second_rows = vec![mock_candle(t2), mock_candle(t0 + Duration::hours(3))];

        let f = LoggingKlineFetcher::new(vec![
            Ok(KlineFetch::Interrupted {
                rows: first_rows.clone(),
                covered_up_to_ms: t2.timestamp_millis(),
            }),
            Ok(KlineFetch::Complete(second_rows.clone())),
        ]);
        let calls = f.calls.clone();
        let got = ensure_candles_with_fetcher(&mut store, f.closure(), &sym, "1h", t0, t4)
            .expect("Interrupted + retry Complete must succeed");

        assert_eq!(got.len(), 4, "all 4 candles persisted across attempts");
        let log = calls.borrow();
        assert_eq!(log.len(), 2);
        assert_eq!(log[0], (t0, t4), "first call covered the full gap");
        assert_eq!(
            log[1],
            (t2, t4),
            "retry narrowed to the uncovered suffix"
        );

        store.invalidate_candle(&sym, "1h");
    }

    /// Two consecutive `Interrupted` responses exhaust the retry
    /// budget; the third gap-scan sees uncovered range and errors out.
    #[test]
    fn ensure_candles_errors_after_exhausting_attempt_budget() {
        let sym = fresh_candle_symbol("INTERRUPTED_EXHAUST");
        let mut store = CandleStore::new();
        store.invalidate_candle(&sym, "1h");

        let t0 = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        let t2 = t0 + Duration::hours(2);
        let t3 = t0 + Duration::hours(3);
        let t4 = t0 + Duration::hours(4);

        let first_rows = vec![mock_candle(t0), mock_candle(t0 + Duration::hours(1))];
        let second_rows = vec![mock_candle(t2)];

        let f = LoggingKlineFetcher::new(vec![
            Ok(KlineFetch::Interrupted {
                rows: first_rows.clone(),
                covered_up_to_ms: t2.timestamp_millis(),
            }),
            Ok(KlineFetch::Interrupted {
                rows: second_rows.clone(),
                covered_up_to_ms: t3.timestamp_millis(),
            }),
        ]);
        let result = ensure_candles_with_fetcher(&mut store, f.closure(), &sym, "1h", t0, t4);
        assert!(result.is_err(), "two Interrupteds must exceed budget");
        let msg = result.unwrap_err();
        assert!(
            msg.contains("Incomplete"),
            "error must name the coverage shortfall: {msg}"
        );

        // Partial progress persists — the rows from both attempts are
        // in the store, and coverage reflects what was actually probed.
        let gaps = store.candle_coverage_gaps(&sym, "1h", t0, t4);
        assert_eq!(gaps, vec![(t3, t4)], "only the final uncovered suffix remains");

        store.invalidate_candle(&sym, "1h");
    }

    /// Hard fetch error propagates without recording coverage for the
    /// failed gap; a retry run with a clean response succeeds.
    #[test]
    fn ensure_candles_hard_error_leaves_coverage_unchanged() {
        let sym = fresh_candle_symbol("HARD_ERR");
        let mut store = CandleStore::new();
        store.invalidate_candle(&sym, "1h");

        let t0 = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        let t2 = t0 + Duration::hours(2);

        let f1 = LoggingKlineFetcher::new(vec![Err(DataError::Http("boom".into()))]);
        let result =
            ensure_candles_with_fetcher(&mut store, f1.closure(), &sym, "1h", t0, t2);
        assert!(result.is_err());

        // Coverage for [t0, t2) must still be uncovered.
        let gaps = store.candle_coverage_gaps(&sym, "1h", t0, t2);
        assert_eq!(
            gaps,
            vec![(t0, t2)],
            "hard error must not record coverage: {gaps:?}"
        );

        let rows = vec![mock_candle(t0), mock_candle(t0 + Duration::hours(1))];
        let f2 = LoggingKlineFetcher::new(vec![Ok(KlineFetch::Complete(rows.clone()))]);
        let got =
            ensure_candles_with_fetcher(&mut store, f2.closure(), &sym, "1h", t0, t2)
                .expect("retry succeeds");
        assert_eq!(got.len(), 2);

        store.invalidate_candle(&sym, "1h");
    }
}
