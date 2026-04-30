//! Donchian-style breakout / breakdown detectors.
//!
//! A long breakout fires when the current close exceeds the maximum HIGH
//! over the prior `window` bars `[i-window, i)` by at least
//! `min_move_pct`. A short breakdown is the symmetric case against the
//! minimum LOW.
//!
//! Both functions do a single bounded scan per call and allocate nothing.

use crate::events::{BreakoutDetection, Direction};

/// Scan `highs[i-window..i]` for the rolling max and compare it to
/// `close_i`. Returns `None` when `i < window`, when the prior max is
/// NaN, or when the close didn't overshoot by at least `min_move_pct`.
#[inline]
pub fn donchian_high_break(
    highs: &[f64],
    close_i: f64,
    i: usize,
    window: usize,
    min_move_pct: f64,
) -> Option<BreakoutDetection> {
    if i < window || window == 0 || close_i.is_nan() {
        return None;
    }
    let start = i - window;
    let mut hh = highs[start];
    for j in (start + 1)..i {
        let h = highs[j];
        if h > hh {
            hh = h;
        }
    }
    if !hh.is_finite() || hh <= 0.0 {
        return None;
    }
    let break_pct = (close_i - hh) / hh;
    if break_pct < min_move_pct {
        return None;
    }
    Some(BreakoutDetection {
        direction: Direction::Long,
        reference_level: hh,
        break_pct,
    })
}

/// Scan `lows[i-window..i]` for the rolling min and compare it to
/// `close_i`. Returns `None` when `i < window`, when the prior min is
/// NaN / non-positive, or when the close didn't undershoot by at least
/// `min_move_pct`.
#[inline]
pub fn donchian_low_break(
    lows: &[f64],
    close_i: f64,
    i: usize,
    window: usize,
    min_move_pct: f64,
) -> Option<BreakoutDetection> {
    if i < window || window == 0 || close_i.is_nan() {
        return None;
    }
    let start = i - window;
    let mut ll = lows[start];
    for j in (start + 1)..i {
        let l = lows[j];
        if l < ll {
            ll = l;
        }
    }
    if !ll.is_finite() || ll <= 0.0 {
        return None;
    }
    let break_pct = (ll - close_i) / ll;
    if break_pct < min_move_pct {
        return None;
    }
    Some(BreakoutDetection {
        direction: Direction::Short,
        reference_level: ll,
        break_pct,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn long_break_detects_exceed() {
        let highs = vec![100.0, 101.0, 99.0, 102.0, 98.0];
        let d = donchian_high_break(&highs, 103.0, 5, 4, 0.005).unwrap();
        // Scans highs[1..5] = [101, 99, 102, 98] -> max 102.
        assert_eq!(d.direction, Direction::Long);
        assert!((d.reference_level - 102.0).abs() < 1e-12);
        assert!((d.break_pct - (103.0 - 102.0) / 102.0).abs() < 1e-12);
    }

    #[test]
    fn long_break_rejects_below_threshold() {
        let highs = vec![100.0, 101.0, 99.0, 102.0, 98.0];
        // 102.1 is only 0.098% above max; min_move_pct 0.005 (0.5%) -> reject.
        assert!(donchian_high_break(&highs, 102.1, 5, 4, 0.005).is_none());
    }

    #[test]
    fn long_break_rejects_when_i_less_than_window() {
        let highs = vec![100.0, 101.0, 99.0];
        assert!(donchian_high_break(&highs, 103.0, 2, 4, 0.005).is_none());
    }

    #[test]
    fn short_break_detects_undershoot() {
        let lows = vec![100.0, 99.0, 101.0, 98.0, 102.0];
        let d = donchian_low_break(&lows, 97.0, 5, 4, 0.005).unwrap();
        // Scans lows[1..5] = [99, 101, 98, 102] -> min 98.
        assert_eq!(d.direction, Direction::Short);
        assert!((d.reference_level - 98.0).abs() < 1e-12);
        assert!((d.break_pct - (98.0 - 97.0) / 98.0).abs() < 1e-12);
    }

    #[test]
    fn short_break_rejects_above_threshold() {
        let lows = vec![100.0, 99.0, 101.0, 98.0, 102.0];
        assert!(donchian_low_break(&lows, 97.9, 5, 4, 0.005).is_none());
    }

    #[test]
    fn nan_close_returns_none() {
        let highs = vec![100.0; 10];
        let lows = vec![99.0; 10];
        assert!(donchian_high_break(&highs, f64::NAN, 5, 4, 0.001).is_none());
        assert!(donchian_low_break(&lows, f64::NAN, 5, 4, 0.001).is_none());
    }

    /// Reference: opus47_3 v9 inline scan. Used to pin behaviour.
    fn reference_long(highs: &[f64], close: f64, i: usize, window: usize, min: f64) -> Option<f64> {
        if i < window {
            return None;
        }
        let mut hh = highs[i - window];
        for j in (i - window + 1)..i {
            if highs[j] > hh {
                hh = highs[j];
            }
        }
        let pct = (close - hh) / hh;
        if pct >= min { Some(hh) } else { None }
    }

    #[test]
    fn long_matches_reference_on_synthetic_series() {
        let n = 500;
        let mut highs = Vec::with_capacity(n);
        let mut closes = Vec::with_capacity(n);
        for k in 0..n {
            let t = k as f64 * 0.11;
            let drift = (t * 0.05).sin() * 5.0 + (t * 0.3).cos() * 2.0;
            highs.push(100.0 + drift + 0.5);
            closes.push(100.0 + drift);
        }
        for i in 48..n {
            let ref_hh = reference_long(&highs, closes[i], i, 48, 0.008);
            let got = donchian_high_break(&highs, closes[i], i, 48, 0.008);
            match (ref_hh, got) {
                (None, None) => {}
                (Some(hh), Some(d)) => assert!((hh - d.reference_level).abs() < 1e-12),
                (a, b) => panic!("bar {i}: ref={a:?} got={b:?}"),
            }
        }
    }
}
