//! Generalization score — time-based consistency metric.
//!
//! The generalization score is a **pure consistency measure**
//! computed from the coefficient of variation of 2-week PnL buckets
//! over the entire window set. It asks: "is this strategy's PnL
//! stable across time, or is it lumpy?".
//!
//! The design is intentionally agnostic to absolute performance
//! level — a strategy that earns a steady +0.1% per 2-week bucket is
//! more "generalising" than one that earns +5% in one bucket and 0 %
//! elsewhere, even though the second has higher total PnL. Absolute
//! performance is measured by `preference_score`; this metric
//! answers a different question.
//!
//! Algorithm:
//!   1. Sort all `WindowResult`s chronologically.
//!   2. Group consecutive pairs into disjoint 2-week buckets. (Every
//!      window is 7 days, so each bucket is 14 days of strategy PnL
//!      regardless of calendar gaps between windows.) If the number
//!      of windows is odd, the last window forms a single-entry
//!      bucket.
//!   3. Sum the `total_pnl_pct` of each bucket's windows.
//!   4. Compute `CV = std(bucket_pnls) / max(|mean(bucket_pnls)|, ε)`
//!      where `ε = 0.1` (percentage points) prevents division blow-ups
//!      when the mean is near zero.
//!   5. `score = 1 / (1 + CV)`, bounded in `(0, 1]`.
//!
//! Score interpretation:
//!   * `score = 1.0` → every 2-week bucket returned exactly the same
//!     PnL (perfect consistency).
//!   * `score → 0`   → PnL is highly lumpy relative to its mean.

use claude_trader_models::WindowResult;

/// Floor applied to `|mean|` in the CV denominator. Keeps the metric
/// well-defined when the mean bucket PnL is near zero and avoids the
/// classical CV blow-up. Chosen as 0.1 percentage points — small
/// enough that a steady 0.1 %/bucket strategy still scores near 1,
/// large enough to prevent noise-dominated CVs when the strategy
/// barely trades.
pub const CV_MEAN_FLOOR_PP: f64 = 0.1;

/// Number of windows in each generalization bucket. Every window is
/// 7 days, so a 2-window bucket is 14 days of strategy PnL. Kept as
/// a named constant so the metric is easy to retune.
pub const GENERALIZATION_BUCKET_WINDOWS: usize = 2;

/// Output of the generalization-score computation.
#[derive(Debug, Clone, Copy)]
pub struct GeneralizationResult {
    /// `1 / (1 + CV)`. Higher is better; `1.0` is perfect consistency.
    pub score: f64,
    /// Coefficient of variation of bucket PnLs. `std / max(|mean|, ε)`.
    /// Lower is better; 0 is perfect.
    pub cv: f64,
    /// Number of 2-week buckets the metric was computed over.
    pub bucket_count: usize,
    /// Mean of bucket PnL sums (percentage points).
    pub mean_bucket_pnl: f64,
    /// Standard deviation of bucket PnL sums (percentage points).
    pub std_bucket_pnl: f64,
}

impl GeneralizationResult {
    const EMPTY: Self = Self {
        score: 0.0,
        cv: f64::INFINITY,
        bucket_count: 0,
        mean_bucket_pnl: 0.0,
        std_bucket_pnl: 0.0,
    };
}

/// Compute the generalization score from the raw window results.
///
/// # Inputs
///
/// Pass **all** window results across every category. The score
/// aggregates purely on time — category labels are irrelevant here.
///
/// # Behaviour
///
/// - Windows are sorted chronologically by `start`.
/// - Disjoint 2-window (14-day) buckets are formed. If the total
///   window count is odd, the final bucket has a single window.
/// - Returns `GeneralizationResult::EMPTY` (score 0, CV ∞) when there
///   are fewer than two buckets — the variance is undefined.
pub fn compute_generalization_score(window_results: &[WindowResult]) -> GeneralizationResult {
    if window_results.len() < GENERALIZATION_BUCKET_WINDOWS {
        return GeneralizationResult::EMPTY;
    }

    // Chronological order first. Window_results aren't guaranteed
    // sorted by caller; the CV metric is time-based so we must sort.
    let mut sorted: Vec<&WindowResult> = window_results.iter().collect();
    sorted.sort_by_key(|wr| wr.window.start);

    let bucket_pnls: Vec<f64> = sorted
        .chunks(GENERALIZATION_BUCKET_WINDOWS)
        .map(|chunk| chunk.iter().map(|wr| wr.backtest.total_pnl_pct).sum())
        .collect();

    if bucket_pnls.len() < 2 {
        return GeneralizationResult::EMPTY;
    }

    let n = bucket_pnls.len() as f64;
    let mean: f64 = bucket_pnls.iter().sum::<f64>() / n;

    // Population variance — we're describing the observed sample, not
    // inferring a population, so the n (not n−1) denominator is the
    // right choice here.
    let variance: f64 = bucket_pnls.iter().map(|p| (p - mean).powi(2)).sum::<f64>() / n;
    let std = variance.sqrt();

    let cv = std / mean.abs().max(CV_MEAN_FLOOR_PP);
    let score = 1.0 / (1.0 + cv);

    GeneralizationResult {
        score,
        cv,
        bucket_count: bucket_pnls.len(),
        mean_bucket_pnl: mean,
        std_bucket_pnl: std,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, TimeZone, Utc};
    use claude_trader_models::{BacktestResult, EvalWindow, WindowResult};

    fn window(start_days: i64, pnl: f64) -> WindowResult {
        let start = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap()
            + Duration::days(start_days);
        WindowResult {
            window: EvalWindow {
                name: format!("W{start_days}"),
                category: "test".to_string(),
                start,
                end: start + Duration::days(7),
            },
            backtest: BacktestResult {
                trades: Vec::new(),
                total_trades: 0,
                wins: 0,
                losses: 0,
                open_trades: 0,
                unfilled: 0,
                win_rate: 0.0,
                total_pnl_pct: pnl,
                avg_pnl_pct: 0.0,
                profit_factor: 0.0,
                max_drawdown_pct: 0.0,
                equity_curve: vec![100.0],
            },
            signal_count: 0,
            short_count: 0,
            long_count: 0,
        }
    }

    fn approx_eq(a: f64, b: f64, tol: f64) -> bool {
        (a - b).abs() < tol
    }

    #[test]
    fn empty_input_returns_empty_result() {
        let r = compute_generalization_score(&[]);
        assert_eq!(r.score, 0.0);
        assert_eq!(r.bucket_count, 0);
    }

    #[test]
    fn single_window_returns_empty_result() {
        let data = vec![window(0, 1.0)];
        let r = compute_generalization_score(&data);
        assert_eq!(r.score, 0.0);
    }

    #[test]
    fn perfectly_consistent_gives_score_one() {
        // 10 windows, each +1% → every 2-week bucket is +2%. std = 0.
        let data: Vec<WindowResult> = (0..10).map(|i| window(i * 7, 1.0)).collect();
        let r = compute_generalization_score(&data);
        assert_eq!(r.bucket_count, 5);
        assert!(approx_eq(r.std_bucket_pnl, 0.0, 1e-12));
        assert!(approx_eq(r.cv, 0.0, 1e-12));
        assert!(approx_eq(r.score, 1.0, 1e-12));
    }

    /// The user's intended property: a steady 0.1 %/week strategy
    /// should score near the top of the scale, not be drowned by the
    /// CV mean floor.
    #[test]
    fn tiny_steady_return_scores_near_one() {
        let data: Vec<WindowResult> = (0..20).map(|i| window(i * 7, 0.1)).collect();
        let r = compute_generalization_score(&data);
        assert!(approx_eq(r.std_bucket_pnl, 0.0, 1e-12));
        assert!(approx_eq(r.score, 1.0, 1e-12));
    }

    #[test]
    fn lumpy_pnl_scores_lower_than_steady() {
        // Steady: every window +2% → every 2-week bucket +4%, std 0.
        let steady: Vec<WindowResult> = (0..10).map(|i| window(i * 7, 2.0)).collect();
        // Lumpy: one +20% window, rest flat. Total PnL identical to
        // steady, but the bucket containing the fat week stands out
        // so bucket std ≫ 0.
        let lumpy: Vec<WindowResult> = (0..10)
            .map(|i| window(i * 7, if i == 0 { 20.0 } else { 0.0 }))
            .collect();

        let rs = compute_generalization_score(&steady);
        let rl = compute_generalization_score(&lumpy);
        assert!(rs.score > rl.score);
        assert!(approx_eq(rs.score, 1.0, 1e-12));
        assert!(rl.score < 1.0);
    }

    #[test]
    fn zero_mean_is_floor_protected() {
        // Half +1, half -1 → mean ≈ 0 across buckets. Without the
        // floor this would blow up; with the floor the score is
        // simply small (large CV ≈ std / 0.1) but finite.
        let data: Vec<WindowResult> = (0..20)
            .map(|i| window(i * 7, if i % 2 == 0 { 1.0 } else { -1.0 }))
            .collect();
        let r = compute_generalization_score(&data);
        assert!(r.score.is_finite());
        assert!(r.cv.is_finite());
        // Bucket PnLs alternate (1-1)=0 and (-1+1)=0 — actually both
        // zero with this pairing, so std = 0, CV = 0, score = 1.
        // Change the pattern to expose non-zero variance:
        let data2: Vec<WindowResult> = (0..20)
            .map(|i| window(i * 7, if i < 10 { 1.0 } else { -1.0 }))
            .collect();
        let r2 = compute_generalization_score(&data2);
        assert!(r2.score.is_finite());
        assert!(r2.score < 1.0);
    }

    #[test]
    fn sorting_is_stable_regardless_of_input_order() {
        let mut ordered: Vec<WindowResult> = (0..10).map(|i| window(i * 7, (i as f64) * 0.5)).collect();
        let r_a = compute_generalization_score(&ordered);
        ordered.reverse();
        let r_b = compute_generalization_score(&ordered);
        assert!(approx_eq(r_a.score, r_b.score, 1e-12));
        assert!(approx_eq(r_a.cv, r_b.cv, 1e-12));
    }

    #[test]
    fn odd_window_count_forms_single_entry_final_bucket() {
        // 5 windows → 2 full buckets + 1 single-entry bucket = 3 buckets.
        let data: Vec<WindowResult> = (0..5).map(|i| window(i * 7, 1.0)).collect();
        let r = compute_generalization_score(&data);
        assert_eq!(r.bucket_count, 3);
    }
}
