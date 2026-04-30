//! CVD + RSI double-divergence detectors.
//!
//! A pivot bar is chosen as the extreme-high / extreme-low within
//! `[i - lookback, i - exclude_recent)` on the price series. The current
//! bar `i` is then checked for:
//!   * a new high (bearish) / new low (bullish) of at least
//!     `min_price_diverg_pct` against the pivot,
//!   * a CVD value that moved STRICTLY the opposite way of price
//!     (weak-hands distribution / hidden accumulation),
//!   * an RSI gap of at least `min_rsi_gap` in the same direction as CVD.
//!
//! Both detectors are pure functions of their input slices; no internal
//! state, no allocation in the hot path.

use crate::events::DivergenceDetection;

/// Parameters for a divergence scan. All thresholds are `f64` fractions
/// (not percents): e.g. `0.005` = 0.5 % price diverge.
#[derive(Debug, Clone, Copy)]
pub struct DivergenceParams {
    /// Total lookback window (bars).
    pub lookback: usize,
    /// How many of the most recent bars to exclude from the pivot search
    /// (so the pivot is a distinct prior swing, not the current extension).
    pub exclude_recent: usize,
    /// Minimum fractional price divergence (higher-high or lower-low) for
    /// the current bar against the pivot. Strictly `>=`.
    pub min_price_diverg_pct: f64,
    /// Minimum RSI gap in the oriented direction (bearish = pivot_rsi -
    /// current_rsi; bullish = current_rsi - pivot_rsi). Strictly `>=`.
    pub min_rsi_gap: f64,
}

impl DivergenceParams {
    /// Sanity check used internally. Exposed so strategy crates can
    /// fail fast on invalid param combos.
    pub const fn is_valid(&self) -> bool {
        self.lookback > self.exclude_recent + 3
            && self.lookback > 0
            && self.exclude_recent > 0
    }
}

/// Bearish double divergence: current HIGH exceeds the prior-peak HIGH by
/// at least `min_price_diverg_pct`, CVD is STRICTLY below the pivot's CVD,
/// and RSI gap meets `min_rsi_gap`.
///
/// All input slices must be the same length. Returns `None` when:
///   * `i < params.lookback`
///   * the exclude-recent window leaves fewer than 3 bars
///   * `highs[pivot]` is non-positive (can't form a ratio)
///   * price diverge below threshold
///   * CVD didn't diverge (gap `<= 0`)
///   * RSI gap below threshold
///   * any of the required inputs is NaN
#[inline]
pub fn bearish_double_divergence(
    highs: &[f64],
    cvd: &[f64],
    rsi: &[f64],
    i: usize,
    params: &DivergenceParams,
) -> Option<DivergenceDetection> {
    if i < params.lookback || highs.len() != cvd.len() || highs.len() != rsi.len() {
        return None;
    }
    let start = i - params.lookback;
    let end = i - params.exclude_recent;
    if end <= start + 3 {
        return None;
    }

    // Prior peak by HIGH within [start, end).
    let mut pivot = start;
    let mut pivot_val = highs[start];
    for j in (start + 1)..end {
        let h = highs[j];
        if h > pivot_val {
            pivot_val = h;
            pivot = j;
        }
    }

    if pivot_val <= 0.0 {
        return None;
    }

    let hi_i = highs[i];
    let cvd_i = cvd[i];
    let cvd_p = cvd[pivot];
    let rsi_i = rsi[i];
    let rsi_p = rsi[pivot];
    if hi_i.is_nan() || cvd_i.is_nan() || cvd_p.is_nan() || rsi_i.is_nan() || rsi_p.is_nan() {
        return None;
    }

    let price_diverg_pct = (hi_i - pivot_val) / pivot_val;
    if price_diverg_pct < params.min_price_diverg_pct {
        return None;
    }

    let cvd_gap = cvd_p - cvd_i;
    if cvd_gap <= 0.0 {
        return None;
    }

    let rsi_gap = rsi_p - rsi_i;
    if rsi_gap < params.min_rsi_gap {
        return None;
    }

    Some(DivergenceDetection {
        pivot_idx: pivot,
        price_diverg_pct,
        cvd_gap,
        rsi_gap,
    })
}

/// Bullish double divergence (mirror of `bearish_double_divergence`):
/// current LOW below the prior-trough LOW, CVD strictly higher, RSI
/// strictly higher.
///
/// Returns `None` if any input is NaN or the trough low is non-positive.
#[inline]
pub fn bullish_double_divergence(
    lows: &[f64],
    cvd: &[f64],
    rsi: &[f64],
    i: usize,
    params: &DivergenceParams,
) -> Option<DivergenceDetection> {
    if i < params.lookback || lows.len() != cvd.len() || lows.len() != rsi.len() {
        return None;
    }
    let start = i - params.lookback;
    let end = i - params.exclude_recent;
    if end <= start + 3 {
        return None;
    }

    let mut pivot = start;
    let mut pivot_val = lows[start];
    for j in (start + 1)..end {
        let l = lows[j];
        if l < pivot_val {
            pivot_val = l;
            pivot = j;
        }
    }

    if pivot_val <= 0.0 {
        return None;
    }

    let lo_i = lows[i];
    let cvd_i = cvd[i];
    let cvd_p = cvd[pivot];
    let rsi_i = rsi[i];
    let rsi_p = rsi[pivot];
    if lo_i.is_nan() || cvd_i.is_nan() || cvd_p.is_nan() || rsi_i.is_nan() || rsi_p.is_nan() {
        return None;
    }

    let price_diverg_pct = (pivot_val - lo_i) / pivot_val;
    if price_diverg_pct < params.min_price_diverg_pct {
        return None;
    }

    let cvd_gap = cvd_i - cvd_p;
    if cvd_gap <= 0.0 {
        return None;
    }

    let rsi_gap = rsi_i - rsi_p;
    if rsi_gap < params.min_rsi_gap {
        return None;
    }

    Some(DivergenceDetection {
        pivot_idx: pivot,
        price_diverg_pct,
        cvd_gap,
        rsi_gap,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_params() -> DivergenceParams {
        DivergenceParams {
            lookback: 24,
            exclude_recent: 4,
            min_price_diverg_pct: 0.003,
            min_rsi_gap: 0.0,
        }
    }

    #[test]
    fn params_valid_requires_gap() {
        assert!(default_params().is_valid());
        assert!(!DivergenceParams {
            lookback: 5,
            exclude_recent: 4,
            min_price_diverg_pct: 0.0,
            min_rsi_gap: 0.0
        }
        .is_valid());
    }

    #[test]
    fn bearish_rejects_below_lookback() {
        let n = 30;
        let highs = vec![100.0; n];
        let cvd = vec![0.0; n];
        let rsi = vec![50.0; n];
        assert!(bearish_double_divergence(&highs, &cvd, &rsi, 10, &default_params()).is_none());
    }

    #[test]
    fn bearish_detects_classic_divergence() {
        // Prior peak at idx 10 = high 110, cvd 100, rsi 75.
        // Current bar at idx 28 = higher high 112, lower cvd 90, lower rsi 70.
        let n = 30;
        let mut highs = vec![100.0; n];
        let mut cvd = vec![50.0; n];
        let mut rsi = vec![60.0; n];
        highs[10] = 110.0;
        cvd[10] = 100.0;
        rsi[10] = 75.0;
        highs[28] = 112.0;
        cvd[28] = 90.0;
        rsi[28] = 70.0;
        let d = bearish_double_divergence(&highs, &cvd, &rsi, 28, &default_params()).unwrap();
        assert_eq!(d.pivot_idx, 10);
        assert!((d.price_diverg_pct - (112.0 - 110.0) / 110.0).abs() < 1e-12);
        assert!((d.cvd_gap - 10.0).abs() < 1e-12);
        assert!((d.rsi_gap - 5.0).abs() < 1e-12);
    }

    #[test]
    fn bearish_rejects_when_cvd_not_diverging() {
        let n = 30;
        let mut highs = vec![100.0; n];
        let mut cvd = vec![50.0; n];
        let rsi = vec![70.0; n];
        highs[10] = 110.0;
        cvd[10] = 100.0;
        highs[28] = 112.0;
        cvd[28] = 105.0; // cvd HIGHER — no distribution
        assert!(bearish_double_divergence(&highs, &cvd, &rsi, 28, &default_params()).is_none());
    }

    #[test]
    fn bearish_rejects_sub_threshold_price_diverg() {
        let n = 30;
        let mut highs = vec![100.0; n];
        let mut cvd = vec![50.0; n];
        let rsi = vec![70.0; n];
        highs[10] = 110.0;
        cvd[10] = 100.0;
        highs[28] = 110.05; // <0.3% HH
        cvd[28] = 80.0;
        assert!(bearish_double_divergence(&highs, &cvd, &rsi, 28, &default_params()).is_none());
    }

    #[test]
    fn bearish_rejects_nan_inputs() {
        let n = 30;
        let mut highs = vec![100.0; n];
        let mut cvd = vec![50.0; n];
        let rsi = vec![70.0; n];
        highs[10] = 110.0;
        cvd[10] = 100.0;
        highs[28] = 112.0;
        cvd[28] = f64::NAN; // RSI finite but CVD NaN
        assert!(bearish_double_divergence(&highs, &cvd, &rsi, 28, &default_params()).is_none());
    }

    #[test]
    fn bullish_mirror_detects_divergence() {
        let n = 30;
        let mut lows = vec![100.0; n];
        let mut cvd = vec![50.0; n];
        let mut rsi = vec![40.0; n];
        lows[10] = 90.0;
        cvd[10] = 100.0;
        rsi[10] = 25.0;
        lows[28] = 88.0;
        cvd[28] = 110.0;
        rsi[28] = 30.0;
        let d = bullish_double_divergence(&lows, &cvd, &rsi, 28, &default_params()).unwrap();
        assert_eq!(d.pivot_idx, 10);
        assert!((d.price_diverg_pct - (90.0 - 88.0) / 90.0).abs() < 1e-12);
        assert!((d.cvd_gap - 10.0).abs() < 1e-12);
        assert!((d.rsi_gap - 5.0).abs() < 1e-12);
    }

    /// Reference implementation mirroring opus47_2 v25 exactly.
    /// Used to pin the crate's behaviour to the research baseline.
    fn reference_bearish(
        highs: &[f64],
        cvd: &[f64],
        rsi: &[f64],
        i: usize,
    ) -> Option<(usize, f64, f64, f64)> {
        const LOOKBACK: usize = 24;
        const EXCLUDE: usize = 4;
        const MIN_PCT: f64 = 0.003;
        if i < LOOKBACK {
            return None;
        }
        let start = i - LOOKBACK;
        let end = i - EXCLUDE;
        if end <= start + 3 {
            return None;
        }
        let mut peak = start;
        for j in start..end {
            if highs[j] > highs[peak] {
                peak = j;
            }
        }
        let hh_pct = (highs[i] - highs[peak]) / highs[peak];
        if hh_pct < MIN_PCT {
            return None;
        }
        let cvd_gap = cvd[peak] - cvd[i];
        if cvd_gap <= 0.0 {
            return None;
        }
        let rsi_gap = rsi[peak] - rsi[i];
        if rsi_gap < 0.0 {
            return None;
        }
        Some((peak, hh_pct, cvd_gap, rsi_gap))
    }

    #[test]
    fn bearish_matches_reference_on_synthetic_series() {
        // Deterministic pseudo-random series.
        let n = 500;
        let mut highs = Vec::with_capacity(n);
        let mut cvd = Vec::with_capacity(n);
        let mut rsi = Vec::with_capacity(n);
        let mut acc = 100.0f64;
        let mut c = 0.0f64;
        for k in 0..n {
            let theta = k as f64 * 0.17;
            acc += theta.sin() * 0.3 + (theta * 0.7).cos() * 0.5;
            c += (theta * 1.3).sin();
            highs.push(100.0 + acc);
            cvd.push(c);
            rsi.push(50.0 + (theta * 0.4).sin() * 20.0);
        }
        let params = default_params();
        for i in 24..n {
            let expected = reference_bearish(&highs, &cvd, &rsi, i);
            let got = bearish_double_divergence(&highs, &cvd, &rsi, i, &params);
            match (expected, got) {
                (None, None) => {}
                (Some((pi, pct, cg, rg)), Some(d)) => {
                    assert_eq!(pi, d.pivot_idx, "bar {i}");
                    assert!((pct - d.price_diverg_pct).abs() < 1e-12);
                    assert!((cg - d.cvd_gap).abs() < 1e-12);
                    assert!((rg - d.rsi_gap).abs() < 1e-12);
                }
                (a, b) => panic!("mismatch at bar {i}: ref={a:?} got={b:?}"),
            }
        }
    }
}
