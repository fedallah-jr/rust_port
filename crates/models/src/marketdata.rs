//! Market data model types.

use std::borrow::Cow;
use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// DataRequirement
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DataRequirement {
    #[serde(rename = "ohlcv")]
    Ohlcv,
    #[serde(rename = "agg_trades")]
    AggTrades,
}

impl DataRequirement {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Ohlcv => "ohlcv",
            Self::AggTrades => "agg_trades",
        }
    }
}

impl std::fmt::Display for DataRequirement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

// ---------------------------------------------------------------------------
// MarketDataRequest
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketDataRequest {
    #[serde(default = "default_datasets")]
    pub datasets: HashSet<DataRequirement>,
    #[serde(default = "default_ohlcv_interval")]
    pub ohlcv_interval: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub poll_ohlcv_interval: Option<String>,
}

fn default_datasets() -> HashSet<DataRequirement> {
    let mut s = HashSet::new();
    s.insert(DataRequirement::Ohlcv);
    s
}

fn default_ohlcv_interval() -> String {
    "1h".to_string()
}

impl Default for MarketDataRequest {
    fn default() -> Self {
        Self {
            datasets: default_datasets(),
            ohlcv_interval: "1h".to_string(),
            poll_ohlcv_interval: None,
        }
    }
}

impl MarketDataRequest {
    pub fn ohlcv_only(interval: &str) -> Self {
        Self {
            ohlcv_interval: interval.to_string(),
            ..Default::default()
        }
    }

    pub fn effective_poll_ohlcv_interval(&self) -> &str {
        self.poll_ohlcv_interval
            .as_deref()
            .unwrap_or(&self.ohlcv_interval)
    }
}

// ---------------------------------------------------------------------------
// MarketBias
// ---------------------------------------------------------------------------

/// BTC structural market bias — shared across btc_structure, research_runtime,
/// context, and strategies. `Copy` so per-boundary clones are free.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MarketBias {
    #[default]
    Neutral,
    Bullish,
    Bearish,
}

impl MarketBias {
    /// Returns the lowercase string form used in JSON serialization and by
    /// engine feature rows that embed the bias as a `FeatureValue::Str`.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Neutral => "neutral",
            Self::Bullish => "bullish",
            Self::Bearish => "bearish",
        }
    }

    /// Parse from the lowercase string form; unknown values map to `Neutral`.
    /// Used by consumers that read bias out of `FeatureValue::Str` entries.
    pub fn from_lowercase_str(s: &str) -> Self {
        match s {
            "bullish" => Self::Bullish,
            "bearish" => Self::Bearish,
            _ => Self::Neutral,
        }
    }
}

// ---------------------------------------------------------------------------
// KeyLevels
// ---------------------------------------------------------------------------

/// Price levels at various timeframes — mirrors `marketdata/key_levels.py`.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct KeyLevels {
    // 4-Hour
    pub h4_open: Option<f64>,
    pub prev_h4_high: Option<f64>,
    pub prev_h4_low: Option<f64>,
    pub h4_eq: Option<f64>,

    // Daily
    pub daily_open: Option<f64>,
    pub pdh: Option<f64>,
    pub pdl: Option<f64>,
    pub daily_eq: Option<f64>,

    // Weekly
    pub weekly_open: Option<f64>,
    pub prev_week_high: Option<f64>,
    pub prev_week_low: Option<f64>,
    pub weekly_eq: Option<f64>,

    // Monthly
    pub monthly_open: Option<f64>,
    pub prev_month_high: Option<f64>,
    pub prev_month_low: Option<f64>,
    pub monthly_eq: Option<f64>,

    // Quarterly
    pub quarterly_open: Option<f64>,
    pub prev_quarter_high: Option<f64>,
    pub prev_quarter_low: Option<f64>,
    pub quarterly_eq: Option<f64>,

    // Yearly
    pub yearly_open: Option<f64>,
    pub yearly_high: Option<f64>,
    pub yearly_low: Option<f64>,
    pub yearly_eq: Option<f64>,

    // Monday range
    pub monday_high: Option<f64>,
    pub monday_low: Option<f64>,
    pub monday_mid: Option<f64>,

    // Sessions
    pub asia_open: Option<f64>,
    pub asia_high: Option<f64>,
    pub asia_low: Option<f64>,

    pub london_open: Option<f64>,
    pub london_high: Option<f64>,
    pub london_low: Option<f64>,

    pub ny_open: Option<f64>,
    pub ny_high: Option<f64>,
    pub ny_low: Option<f64>,
}

// ---------------------------------------------------------------------------
// HtfData — strategy-visible higher-timeframe data
// ---------------------------------------------------------------------------

/// Higher-timeframe candles and their precomputed indicators, handed to
/// strategies alongside `ContextMap`. Point-in-time signals (funding, key
/// levels, BTC structure) live in `ContextMap`, not here.
#[derive(Debug, Clone, Default)]
pub struct HtfData {
    /// Additional timeframe candles: `interval → symbol → candles`. Sorted
    /// by `close_time`. Populated when the strategy declares
    /// `additional_intervals()`.
    pub additional_candles: HashMap<String, HashMap<String, Vec<crate::Candle>>>,
    /// Precomputed indicators on the additional timeframe candles:
    /// `interval → symbol → (indicator_name → values)`. Aligned 1:1 with
    /// `additional_candles[interval][symbol]`.
    pub additional_indicators: HashMap<String, HashMap<String, HashMap<String, Vec<f64>>>>,
}

impl HtfData {
    /// Return a view truncated to `close_time <= t`. Borrowed fast path when
    /// every series' last candle is already `<= t`.
    pub fn truncated_at(&self, t: DateTime<Utc>) -> Cow<'_, Self> {
        if self.is_bounded_by(t) {
            return Cow::Borrowed(self);
        }
        Cow::Owned(Self {
            additional_candles: self
                .additional_candles
                .iter()
                .map(|(interval, syms)| {
                    (
                        interval.clone(),
                        syms.iter()
                            .map(|(sym, klines)| {
                                let idx = klines.partition_point(|c| c.close_time <= t);
                                (sym.clone(), klines[..idx].to_vec())
                            })
                            .collect(),
                    )
                })
                .collect(),
            additional_indicators: self
                .additional_indicators
                .iter()
                .map(|(interval, syms)| {
                    (
                        interval.clone(),
                        syms.iter()
                            .map(|(sym, ind_result)| {
                                let candle_count = self
                                    .additional_candles
                                    .get(interval)
                                    .and_then(|s| s.get(sym))
                                    .map(|klines| klines.partition_point(|c| c.close_time <= t))
                                    .unwrap_or(0);
                                (
                                    sym.clone(),
                                    ind_result
                                        .iter()
                                        .map(|(name, vals)| {
                                            (name.clone(), vals[..candle_count].to_vec())
                                        })
                                        .collect(),
                                )
                            })
                            .collect(),
                    )
                })
                .collect(),
        })
    }

    fn is_bounded_by(&self, t: DateTime<Utc>) -> bool {
        self.additional_candles
            .values()
            .all(|syms| syms.values().all(|v| v.last().map_or(true, |c| c.close_time <= t)))
    }

    /// Debug-assert that every additional_candles series is sorted by
    /// close_time and that additional_indicators are 1:1 aligned.
    pub fn debug_assert_sorted(&self) {
        for (interval, syms) in &self.additional_candles {
            for (sym, klines) in syms {
                debug_assert!(
                    klines
                        .windows(2)
                        .all(|w| w[0].close_time <= w[1].close_time),
                    "additional_candles[{interval}][{sym}] not sorted by close_time"
                );
            }
        }
        for (interval, syms) in &self.additional_indicators {
            for (sym, ind_result) in syms {
                let candle_len = self
                    .additional_candles
                    .get(interval)
                    .and_then(|s| s.get(sym))
                    .map(|c| c.len())
                    .unwrap_or(0);
                for (name, vals) in ind_result {
                    debug_assert_eq!(
                        vals.len(),
                        candle_len,
                        "additional_indicators[{interval}][{sym}][{name}] length {} != candle length {candle_len}",
                        vals.len()
                    );
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Evaluator result types — mirrors `backtester/evaluator.py`
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalWindow {
    pub name: String,
    pub category: String,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowResult {
    pub window: EvalWindow,
    pub backtest: BacktestResult,
    pub signal_count: usize,
    pub short_count: usize,
    pub long_count: usize,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CategorySummary {
    pub category: String,
    pub windows: usize,
    pub total_pnl: f64,
    pub weekly_win_rate: f64,
    pub positive_weeks: usize,
    pub worst_week_pnl: f64,
    pub best_week_pnl: f64,
    pub total_trades: usize,
    pub resolved_trades: usize,
    pub short_trades: usize,
    pub long_trades: usize,
    pub active_weeks: usize,
    pub trade_win_rate: f64,
    pub profit_factor: f64,
    pub sortino_ratio: f64,
    pub max_drawdown_pct: f64,
    pub pnl_to_mdd: f64,
    pub weekly_omega_ratio: f64,
    pub coverage_penalty: f64,
    pub preference_eligible: bool,
    pub preference_score: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SymbolSummary {
    pub symbol: String,
    pub total_trades: usize,
    pub wins: usize,
    pub losses: usize,
    pub win_rate: f64,
    pub total_pnl: f64,
    pub avg_pnl: f64,
    pub profit_factor: f64,
    pub avg_hold_hours: f64,
    pub short_trades: usize,
    pub short_wins: usize,
    pub short_pnl: f64,
    pub long_trades: usize,
    pub long_wins: usize,
    pub long_pnl: f64,
    pub tp_exits: usize,
    pub sl_exits: usize,
    pub timeout_exits: usize,
    pub unfilled: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioConfig {
    #[serde(default = "default_true")]
    pub approximate: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seed: Option<u64>,
    #[serde(default = "default_risk_free_rate")]
    pub risk_free_rate_annual: f64,
    #[serde(default = "default_entry_delay")]
    pub entry_delay_seconds: i64,
    #[serde(default = "default_data_max_workers")]
    pub data_max_workers: usize,
    #[serde(default)]
    pub backtest_max_workers: usize,
}

fn default_true() -> bool {
    true
}
fn default_risk_free_rate() -> f64 {
    0.0373
}
fn default_entry_delay() -> i64 {
    3
}
fn default_data_max_workers() -> usize {
    8
}

impl Default for PortfolioConfig {
    fn default() -> Self {
        Self {
            approximate: true,
            seed: None,
            risk_free_rate_annual: 0.0373,
            entry_delay_seconds: 3,
            data_max_workers: 8,
            backtest_max_workers: 0,
        }
    }
}

use crate::backtester::BacktestResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationReport {
    pub window_results: Vec<WindowResult>,
    pub config: PortfolioConfig,
    pub symbols: Vec<String>,
}

// ---------------------------------------------------------------------------
// CalibrationResult — mirrors `backtester/calibration.py`
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalibrationResult {
    pub best_params: HashMap<String, serde_json::Value>,
    pub best_score: f64,
    pub candidates_evaluated: usize,
}
