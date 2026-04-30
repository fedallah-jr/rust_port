//! Shared event / detection structs returned by the detectors in this crate.
//!
//! All types are `Copy` so they can flow through tight inner loops without
//! allocation, and through `TwoStageBook` slots without clone costs.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Long,
    Short,
}

/// Outcome of a CVD + RSI double-divergence scan at a given bar.
///
/// `pivot_idx` is the index of the prior peak (for bearish) or trough (for
/// bullish) that the current bar diverged against.
///
/// `price_diverg_pct` is the fractional new-high / new-low vs the pivot:
/// for bearish, `(highs[i] - highs[pivot]) / highs[pivot]`. Always
/// non-negative — zero or negative values mean "no divergence" and the
/// detector returns `None`.
///
/// `cvd_gap` is strictly positive and expressed in the sign that matches
/// the direction: bearish = `cvd[pivot] - cvd[i]`, bullish = `cvd[i] -
/// cvd[pivot]`.
///
/// `rsi_gap` is oriented the same way as `cvd_gap`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DivergenceDetection {
    pub pivot_idx: usize,
    pub price_diverg_pct: f64,
    pub cvd_gap: f64,
    pub rsi_gap: f64,
}

/// Outcome of a Donchian breakout / breakdown scan at a given bar.
///
/// `reference_level` is the HH / LL that was broken (the max high over
/// `[i-window, i)` for a long break, the min low for a short break).
/// `break_pct` is the fractional overshoot of close past that level,
/// always strictly positive; zero / negative overshoots return `None`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BreakoutDetection {
    pub direction: Direction,
    pub reference_level: f64,
    pub break_pct: f64,
}

/// Outcome of a key-level pierce/rejection scan.
///
/// The bar's extreme (high for resistance, low for support) pierced a
/// named horizontal level (PDH/PDL/prev-week/prev-month/...) and the
/// close retraced back through the level.
///
/// `pierce_pct` is the fractional overshoot past the level, always
/// strictly positive.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LevelPierce {
    pub level_name: &'static str,
    pub level_value: f64,
    pub pierce_pct: f64,
}
