//! Core BTC structure simulation engine.
//!
//! Implements the dual-track state machine for swing detection, confirmation,
//! and structure break detection (BOS/CHoCH).

use std::borrow::Cow;
use std::collections::{HashMap, VecDeque};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::config::BtcStructureConfig;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// Main output of the structure simulation.
#[derive(Debug, Clone, Default)]
pub struct StructureArtifacts {
    /// Per-bar feature rows (date → feature map).
    pub feature_rows: Vec<HashMap<&'static str, FeatureValue>>,
    /// Confirmed swing highs.
    pub confirmed_highs: Vec<ConfirmedLevel>,
    /// Confirmed swing lows.
    pub confirmed_lows: Vec<ConfirmedLevel>,
    /// Structure break events (BOS/CHoCH).
    pub structure_breaks: Vec<StructureBreak>,
    /// Summary statistics.
    pub summary: HashMap<String, serde_json::Value>,
}

/// A confirmed swing level.
///
/// `bar_index` is the **swing** bar (the extremum; Python's `swing_date`).
/// `date` and `confirmation_bar_index` identify the bar at which the swing
/// was confirmed and became causally observable (Python's `available_on`).
/// Downstream consumers that need to reason causally about "when did this
/// level exist" must use `confirmation_bar_index`, not `bar_index`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfirmedLevel {
    pub bar_index: usize,
    pub date: DateTime<Utc>,
    pub confirmation_bar_index: usize,
    pub value: f64,
    pub label: String,
    pub confluence_count: usize,
    pub confluence_windows: Vec<usize>,
    pub bars_to_confirmation: usize,
    pub breaks_structure: bool,
}

/// A structure break event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructureBreak {
    pub bar_index: usize,
    pub date: DateTime<Utc>,
    pub event: String, // "bos_up", "bos_down", "choch_up", "choch_down"
    pub close: f64,
    pub broken_level: f64,
    pub excursion: f64,
}

/// Feature values can be float, int, bool, or string.
///
/// `Str` holds a `Cow<'static, str>` so static strings (e.g. market-bias
/// labels) can be inserted without a heap allocation; dynamic strings go via
/// `Cow::Owned`. Deserialization always produces `Cow::Owned(String)`, so the
/// on-disk JSON shape is unchanged.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FeatureValue {
    Float(f64),
    Int(i64),
    Bool(bool),
    Str(Cow<'static, str>),
    Null,
}

// Market bias direction — the canonical type lives in the shared models crate.
// Strategies and the runtime import it directly from `claude_trader_models`;
// this in-scope alias exists only so `engine.rs` can say `MarketBias` unqualified.
use claude_trader_models::MarketBias;

/// Checkpoint for resumable computation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructureCheckpoint {
    pub resume_from: usize,
    pub state: EngineState,
    pub feature_rows_count: usize,
}

/// Mutable engine state tracked across bars.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EngineState {
    pub candidate_high: Option<Candidate>,
    pub candidate_low: Option<Candidate>,
    pub latest_confirmed_high: Option<ConfirmedLevel>,
    pub latest_confirmed_low: Option<ConfirmedLevel>,
    pub active_side: Option<String>, // "high" or "low"
    pub last_confirmed_side: Option<String>,
    pub market_bias: MarketBias,
}

/// A swing candidate being tracked.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Candidate {
    pub kind: String, // "high" or "low"
    pub bar_index: usize,
    pub value: f64,
    pub bars_active: usize,
    pub confluence_count: usize,
    pub confluence_windows: Vec<usize>,
    pub breaks_structure: bool,
}

// ---------------------------------------------------------------------------
// Bar data for the hot loop
// ---------------------------------------------------------------------------

struct BarData {
    index: usize,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    atr: f64,
}

// ---------------------------------------------------------------------------
// Main simulation
// ---------------------------------------------------------------------------

/// Run the structure simulation on OHLCV arrays.
///
/// `dates` - bar timestamps
/// `open`, `high`, `low`, `close` - OHLCV arrays (same length)
///
/// Returns artifacts and a checkpoint for resumption.
pub fn simulate_btc_structure(
    dates: &[DateTime<Utc>],
    open: &[f64],
    high: &[f64],
    low: &[f64],
    close: &[f64],
    config: &BtcStructureConfig,
    checkpoint: Option<StructureCheckpoint>,
    prior_artifacts: Option<StructureArtifacts>,
) -> (StructureArtifacts, StructureCheckpoint) {
    let n = close.len();

    if let Err(e) = config.validate() {
        panic!("BtcStructureConfig invalid: {e}");
    }

    // Precompute ATR
    let atr = compute_atr(high, low, close, config.atr_window);

    // Precompute rolling highs/lows for each window
    let rolling_highs = precompute_rolling(high, &config.level_windows, true);
    let rolling_lows = precompute_rolling(low, &config.level_windows, false);

    // Initialize or restore state
    let (start_idx, mut state, mut artifacts) = match checkpoint {
        Some(cp) => {
            let arts = prior_artifacts.unwrap_or_default();
            (cp.resume_from, cp.state, arts)
        }
        None => (
            0,
            EngineState::default_neutral(),
            StructureArtifacts::default(),
        ),
    };

    // Main bar loop
    for i in start_idx..n {
        let bar = BarData {
            index: i,
            open: open[i],
            high: high[i],
            low: low[i],
            close: close[i],
            atr: if i < atr.len() { atr[i] } else { 0.0 },
        };
        let date = dates[i];

        // Save bias before processing (Copy — zero cost)
        let bias_before = state.market_bias;

        // Step 1: Detect structure breaks against confirmed levels
        let break_event = detect_structure_break(&bar, &state, config);
        if let Some(ref brk) = break_event {
            match brk.event.as_str() {
                "bos_up" | "choch_up" => state.market_bias = MarketBias::Bullish,
                "bos_down" | "choch_down" => state.market_bias = MarketBias::Bearish,
                _ => {}
            };
            artifacts.structure_breaks.push(StructureBreak {
                bar_index: i,
                date,
                event: brk.event.clone(),
                close: bar.close,
                broken_level: brk.broken_level,
                excursion: brk.excursion,
            });
        }

        // Step 2: Update candidates and check confirmations
        let (confirmed_high, confirmed_low) =
            update_candidates(&bar, date, &mut state, config, &rolling_highs, &rolling_lows);

        if let Some(ref ch) = confirmed_high {
            artifacts.confirmed_highs.push(ConfirmedLevel {
                bar_index: ch.bar_index,
                date,
                confirmation_bar_index: ch.confirmation_bar_index,
                value: ch.value,
                label: ch.label.clone(),
                confluence_count: ch.confluence_count,
                confluence_windows: ch.confluence_windows.clone(),
                bars_to_confirmation: ch.bars_to_confirmation,
                breaks_structure: ch.breaks_structure,
            });
        }
        if let Some(ref cl) = confirmed_low {
            artifacts.confirmed_lows.push(ConfirmedLevel {
                bar_index: cl.bar_index,
                date,
                confirmation_bar_index: cl.confirmation_bar_index,
                value: cl.value,
                label: cl.label.clone(),
                confluence_count: cl.confluence_count,
                confluence_windows: cl.confluence_windows.clone(),
                bars_to_confirmation: cl.bars_to_confirmation,
                breaks_structure: cl.breaks_structure,
            });
        }

        // Step 3: Build feature row
        let mut row: HashMap<&'static str, FeatureValue> = HashMap::new();
        row.insert("bar_index", FeatureValue::Int(i as i64));
        row.insert("open", FeatureValue::Float(bar.open));
        row.insert("high", FeatureValue::Float(bar.high));
        row.insert("low", FeatureValue::Float(bar.low));
        row.insert("close", FeatureValue::Float(bar.close));
        row.insert("atr", FeatureValue::Float(bar.atr));
        row.insert(
            "market_bias_asof",
            FeatureValue::Str(Cow::Borrowed(bias_before.as_str())),
        );
        row.insert(
            "market_bias_after_close",
            FeatureValue::Str(Cow::Borrowed(state.market_bias.as_str())),
        );

        // Break flags
        let brk_event = break_event.as_ref().map(|b| b.event.as_str()).unwrap_or("");
        row.insert(
            "bos_up_on_close_flag",
            FeatureValue::Bool(brk_event == "bos_up"),
        );
        row.insert(
            "bos_down_on_close_flag",
            FeatureValue::Bool(brk_event == "bos_down"),
        );
        row.insert(
            "choch_up_on_close_flag",
            FeatureValue::Bool(brk_event == "choch_up"),
        );
        row.insert(
            "choch_down_on_close_flag",
            FeatureValue::Bool(brk_event == "choch_down"),
        );

        // Confirmation flags
        row.insert(
            "confirmed_high_on_close_flag",
            FeatureValue::Bool(confirmed_high.is_some()),
        );
        row.insert(
            "confirmed_low_on_close_flag",
            FeatureValue::Bool(confirmed_low.is_some()),
        );

        artifacts.feature_rows.push(row);
    }

    // Summary
    let mut summary = HashMap::new();
    summary.insert("total_bars".to_string(), serde_json::Value::from(n));
    summary.insert(
        "confirmed_highs".to_string(),
        serde_json::Value::from(artifacts.confirmed_highs.len()),
    );
    summary.insert(
        "confirmed_lows".to_string(),
        serde_json::Value::from(artifacts.confirmed_lows.len()),
    );
    summary.insert(
        "structure_breaks".to_string(),
        serde_json::Value::from(artifacts.structure_breaks.len()),
    );
    artifacts.summary = summary;

    let checkpoint = StructureCheckpoint {
        resume_from: n,
        state,
        feature_rows_count: artifacts.feature_rows.len(),
    };

    (artifacts, checkpoint)
}

// ---------------------------------------------------------------------------
// ATR computation
// ---------------------------------------------------------------------------

fn compute_atr(high: &[f64], low: &[f64], close: &[f64], window: usize) -> Vec<f64> {
    let n = high.len();
    let mut tr = vec![0.0f64; n];
    if n > 0 {
        tr[0] = high[0] - low[0];
    }
    for i in 1..n {
        let hl = high[i] - low[i];
        let hc = (high[i] - close[i - 1]).abs();
        let lc = (low[i] - close[i - 1]).abs();
        tr[i] = hl.max(hc).max(lc);
    }

    // Rolling mean of true range with min_periods=1 (matches Python causal_atr)
    let mut atr = vec![0.0f64; n];
    if n > 0 {
        let mut sum = 0.0f64;
        for i in 0..n {
            sum += tr[i];
            if i < window {
                atr[i] = sum / (i + 1) as f64;
            } else {
                sum -= tr[i - window];
                atr[i] = sum / window as f64;
            }
        }
    }
    atr
}

// ---------------------------------------------------------------------------
// Rolling highs/lows precomputation
// ---------------------------------------------------------------------------

fn precompute_rolling(data: &[f64], windows: &[usize], is_high: bool) -> HashMap<usize, Vec<f64>> {
    let n = data.len();
    let mut result = HashMap::new();

    for &w in windows {
        let mut rolling = vec![f64::NAN; n];
        // Monotonic deque: O(n) sliding window max/min.
        // Each element enters and leaves the deque at most once.
        let mut deque: VecDeque<usize> = VecDeque::new();

        for i in 1..n {
            // data[i-1] is the newest element entering window [start, i)
            if is_high {
                while let Some(&back) = deque.back() {
                    if data[back] <= data[i - 1] {
                        deque.pop_back();
                    } else {
                        break;
                    }
                }
            } else {
                while let Some(&back) = deque.back() {
                    if data[back] >= data[i - 1] {
                        deque.pop_back();
                    } else {
                        break;
                    }
                }
            }
            deque.push_back(i - 1);

            // Evict elements outside the window
            let start = if i > w { i - w } else { 0 };
            while let Some(&front) = deque.front() {
                if front < start {
                    deque.pop_front();
                } else {
                    break;
                }
            }

            if let Some(&front) = deque.front() {
                rolling[i] = data[front];
            }
        }

        result.insert(w, rolling);
    }

    result
}

// ---------------------------------------------------------------------------
// Structure break detection
// ---------------------------------------------------------------------------

struct BreakResult {
    event: String,
    broken_level: f64,
    excursion: f64,
}

fn detect_structure_break(
    bar: &BarData,
    state: &EngineState,
    config: &BtcStructureConfig,
) -> Option<BreakResult> {
    let (ch, cl) = match (&state.latest_confirmed_high, &state.latest_confirmed_low) {
        (Some(h), Some(l)) => (h, l),
        _ => return None,
    };

    let up_threshold =
        (bar.atr * config.bos_choch_atr_multiplier).max(ch.value.abs() * config.bos_choch_pct);
    let up_break = bar.close > ch.value + up_threshold;
    let up_excursion = if ch.value != 0.0 {
        (bar.close - ch.value) / ch.value
    } else {
        0.0
    };

    let down_threshold =
        (bar.atr * config.bos_choch_atr_multiplier).max(cl.value.abs() * config.bos_choch_pct);
    let down_break = bar.close < cl.value - down_threshold;
    let down_excursion = if cl.value != 0.0 {
        (cl.value - bar.close) / cl.value
    } else {
        0.0
    };

    // Collision resolution: pick larger excursion
    let (is_up, excursion) = match (up_break, down_break) {
        (true, true) => {
            if up_excursion.abs() >= down_excursion.abs() {
                (true, up_excursion)
            } else {
                (false, down_excursion)
            }
        }
        (true, false) => (true, up_excursion),
        (false, true) => (false, down_excursion),
        (false, false) => return None,
    };

    let event = if is_up {
        if state.market_bias == MarketBias::Bearish {
            "choch_up"
        } else {
            "bos_up"
        }
    } else if state.market_bias == MarketBias::Bullish {
        "choch_down"
    } else {
        "bos_down"
    };

    Some(BreakResult {
        event: event.to_string(),
        broken_level: if is_up { ch.value } else { cl.value },
        excursion,
    })
}

// ---------------------------------------------------------------------------
// Candidate update and confirmation
// ---------------------------------------------------------------------------

fn update_candidates(
    bar: &BarData,
    bar_date: DateTime<Utc>,
    state: &mut EngineState,
    config: &BtcStructureConfig,
    rolling_highs: &HashMap<usize, Vec<f64>>,
    rolling_lows: &HashMap<usize, Vec<f64>>,
) -> (Option<ConfirmedLevel>, Option<ConfirmedLevel>) {
    let mut confirmed_high = None;
    let mut confirmed_low = None;

    // Update high candidate
    if let Some(ref mut cand) = state.candidate_high {
        cand.bars_active += 1;
        if bar.high > cand.value + bar.atr * config.candidate_replace_min_atr_step {
            cand.value = bar.high;
            cand.bar_index = bar.index;
            cand.bars_active = 0;
        }
        let windows = compute_confluence(
            cand.value,
            bar.index,
            &config.level_windows,
            rolling_highs,
            bar.atr * config.level_tolerance_atr_multiplier,
        );
        cand.confluence_count = windows.len();
        cand.confluence_windows = windows;
    }
    // Check confirmation (separate borrow scope)
    if let Some(ref cand) = state.candidate_high {
        if let Some(level) = try_confirm_high(bar, cand, state, config, bar_date) {
            state.latest_confirmed_high = Some(level.clone());
            state.last_confirmed_side = Some("high".to_string());
            confirmed_high = Some(level);
            state.candidate_high = None;
            state.active_side = Some("low".to_string());
        }
    }
    // Drop stale high candidate that exceeded max_candidate_bars without confirming
    if let Some(ref cand) = state.candidate_high {
        if cand.bars_active >= config.max_candidate_bars {
            state.candidate_high = None;
        }
    }
    if state.candidate_high.is_none()
        && confirmed_high.is_none()
        && state.active_side.as_deref() != Some("low")
    {
        // Initialize high candidate
        let windows = compute_confluence(
            bar.high,
            bar.index,
            &config.level_windows,
            rolling_highs,
            bar.atr * config.level_tolerance_atr_multiplier,
        );
        state.candidate_high = Some(Candidate {
            kind: "high".to_string(),
            bar_index: bar.index,
            value: bar.high,
            bars_active: 0,
            confluence_count: windows.len(),
            confluence_windows: windows,
            breaks_structure: state
                .latest_confirmed_high
                .as_ref()
                .map(|h| bar.high > h.value)
                .unwrap_or(false),
        });
    }

    // Update low candidate
    if let Some(ref mut cand) = state.candidate_low {
        cand.bars_active += 1;
        if bar.low < cand.value - bar.atr * config.candidate_replace_min_atr_step {
            cand.value = bar.low;
            cand.bar_index = bar.index;
            cand.bars_active = 0;
        }
        let windows = compute_confluence(
            cand.value,
            bar.index,
            &config.level_windows,
            rolling_lows,
            bar.atr * config.level_tolerance_atr_multiplier,
        );
        cand.confluence_count = windows.len();
        cand.confluence_windows = windows;
    }
    if let Some(ref cand) = state.candidate_low {
        if let Some(level) = try_confirm_low(bar, cand, state, config, bar_date) {
            state.latest_confirmed_low = Some(level.clone());
            state.last_confirmed_side = Some("low".to_string());
            confirmed_low = Some(level);
            state.candidate_low = None;
            state.active_side = Some("high".to_string());
        }
    }
    // Drop stale low candidate that exceeded max_candidate_bars without confirming
    if let Some(ref cand) = state.candidate_low {
        if cand.bars_active >= config.max_candidate_bars {
            state.candidate_low = None;
        }
    }
    if state.candidate_low.is_none()
        && confirmed_low.is_none()
        && state.active_side.as_deref() != Some("high")
    {
        let windows = compute_confluence(
            bar.low,
            bar.index,
            &config.level_windows,
            rolling_lows,
            bar.atr * config.level_tolerance_atr_multiplier,
        );
        state.candidate_low = Some(Candidate {
            kind: "low".to_string(),
            bar_index: bar.index,
            value: bar.low,
            bars_active: 0,
            confluence_count: windows.len(),
            confluence_windows: windows,
            breaks_structure: state
                .latest_confirmed_low
                .as_ref()
                .map(|l| bar.low < l.value)
                .unwrap_or(false),
        });
    }

    (confirmed_high, confirmed_low)
}

fn compute_confluence(
    value: f64,
    bar_index: usize,
    windows: &[usize],
    rolling: &HashMap<usize, Vec<f64>>,
    tolerance: f64,
) -> Vec<usize> {
    let mut matched = Vec::with_capacity(windows.len());
    for &w in windows {
        if let Some(arr) = rolling.get(&w) {
            if bar_index < arr.len() {
                let level = arr[bar_index];
                if !level.is_nan() && (value - level).abs() <= tolerance {
                    matched.push(w);
                }
            }
        }
    }
    matched
}

fn try_confirm_high(
    bar: &BarData,
    cand: &Candidate,
    state: &EngineState,
    config: &BtcStructureConfig,
    bar_date: DateTime<Utc>,
) -> Option<ConfirmedLevel> {
    if cand.bars_active < config.min_bars_confirmation {
        return None;
    }

    let threshold = (bar.atr * config.atr_multiplier).max(cand.value.abs() * config.pct_threshold);
    let excursion = cand.value - bar.close;

    let price_ok = excursion >= threshold;
    let confluence_ok =
        cand.confluence_count >= config.level_confluence_required || cand.breaks_structure;
    let forced = cand.bars_active >= config.force_confirmation_after_bars;

    if price_ok && (confluence_ok || forced) {
        let label = classify_high(cand.value, bar.atr, state, config);
        Some(ConfirmedLevel {
            bar_index: cand.bar_index,
            date: bar_date,
            confirmation_bar_index: bar.index,
            value: cand.value,
            label,
            confluence_count: cand.confluence_count,
            confluence_windows: cand.confluence_windows.clone(),
            bars_to_confirmation: cand.bars_active,
            breaks_structure: cand.breaks_structure,
        })
    } else {
        None
    }
}

fn try_confirm_low(
    bar: &BarData,
    cand: &Candidate,
    state: &EngineState,
    config: &BtcStructureConfig,
    bar_date: DateTime<Utc>,
) -> Option<ConfirmedLevel> {
    if cand.bars_active < config.min_bars_confirmation {
        return None;
    }

    let threshold = (bar.atr * config.atr_multiplier).max(cand.value.abs() * config.pct_threshold);
    let excursion = bar.close - cand.value;

    let price_ok = excursion >= threshold;
    let confluence_ok =
        cand.confluence_count >= config.level_confluence_required || cand.breaks_structure;
    let forced = cand.bars_active >= config.force_confirmation_after_bars;

    if price_ok && (confluence_ok || forced) {
        let label = classify_low(cand.value, bar.atr, state, config);
        Some(ConfirmedLevel {
            bar_index: cand.bar_index,
            date: bar_date,
            confirmation_bar_index: bar.index,
            value: cand.value,
            label,
            confluence_count: cand.confluence_count,
            confluence_windows: cand.confluence_windows.clone(),
            bars_to_confirmation: cand.bars_active,
            breaks_structure: cand.breaks_structure,
        })
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// Classification (HH, LH, LL, HL, EQH, EQL)
// ---------------------------------------------------------------------------

fn label_tolerance(reference_value: f64, atr: f64, config: &BtcStructureConfig) -> f64 {
    let atr_part = if atr.is_nan() { 0.0 } else { atr * config.hhll_tolerance_atr_multiplier };
    let pct_part = reference_value.abs() * config.hhll_tolerance_pct;
    atr_part.max(pct_part)
}

fn classify_high(value: f64, atr: f64, state: &EngineState, config: &BtcStructureConfig) -> String {
    match &state.latest_confirmed_high {
        None => "INITIAL_HIGH".to_string(),
        Some(prev) => {
            let tol = label_tolerance(prev.value, atr, config);
            let diff = value - prev.value;
            if diff > tol {
                "HH".to_string()
            } else if diff < -tol {
                "LH".to_string()
            } else {
                "EQH".to_string()
            }
        }
    }
}

fn classify_low(value: f64, atr: f64, state: &EngineState, config: &BtcStructureConfig) -> String {
    match &state.latest_confirmed_low {
        None => "INITIAL_LOW".to_string(),
        Some(prev) => {
            let tol = label_tolerance(prev.value, atr, config);
            let diff = value - prev.value;
            if diff < -tol {
                "LL".to_string()
            } else if diff > tol {
                "HL".to_string()
            } else {
                "EQL".to_string()
            }
        }
    }
}

impl EngineState {
    fn default_neutral() -> Self {
        // MarketBias defaults to Neutral, so Default::default() is sufficient.
        Self::default()
    }
}
