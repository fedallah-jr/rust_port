//! Key-level pierce/rejection detectors.
//!
//! Given a `KeyLevels` snapshot (as returned by the runtime's
//! `supplementary.key_levels`), detect when a bar's high pierced a named
//! resistance level (`pdh`, `prev_week_high`, `prev_month_high`, ...) AND
//! closed back below it — i.e. a rejection from structural resistance.
//! Mirror logic for supports.
//!
//! The detectors score candidate levels by persistence (monthly >
//! weekly > daily) so you always get the "most significant" level the
//! bar reacted to, not just any level in range.

use claude_trader_models::KeyLevels;

use crate::events::LevelPierce;

#[derive(Debug, Clone, Copy)]
pub struct LevelPierceParams {
    /// Minimum fractional overshoot past the level. E.g. `0.0005` = 0.05 %.
    pub min_pierce_pct: f64,
    /// Maximum fractional overshoot. A bar that ran far past the level
    /// broke it structurally — that's a different pattern (trend
    /// continuation, not rejection).
    pub max_pierce_pct: f64,
}

/// Tier / persistence of a level: monthly > weekly > daily.
/// Higher-tier levels win ties.
const fn tier_of(name: &str) -> u8 {
    match name.as_bytes() {
        b"prev_month_high" | b"prev_month_low" => 3,
        b"prev_week_high" | b"prev_week_low" => 2,
        b"pdh" | b"pdl" => 1,
        _ => 0,
    }
}

fn select_best(
    current: Option<LevelPierce>,
    candidate: LevelPierce,
) -> Option<LevelPierce> {
    match current {
        None => Some(candidate),
        Some(c) if tier_of(candidate.level_name) > tier_of(c.level_name) => Some(candidate),
        other => other,
    }
}

/// Scan resistance levels (pdh, prev_week_high, prev_month_high) and
/// return the highest-tier one where the bar's high pierced the level
/// within `[min_pierce_pct, max_pierce_pct]` AND close stayed below it.
#[inline]
pub fn pierced_resistance(
    close: f64,
    high: f64,
    levels: &KeyLevels,
    params: &LevelPierceParams,
) -> Option<LevelPierce> {
    if close.is_nan() || high.is_nan() {
        return None;
    }
    let candidates: [(&'static str, Option<f64>); 3] = [
        ("pdh", levels.pdh),
        ("prev_week_high", levels.prev_week_high),
        ("prev_month_high", levels.prev_month_high),
    ];
    let mut best: Option<LevelPierce> = None;
    for (name, maybe) in candidates {
        let lvl = match maybe {
            Some(v) if v > 0.0 => v,
            _ => continue,
        };
        let pierce_pct = (high - lvl) / lvl;
        if pierce_pct < params.min_pierce_pct || pierce_pct > params.max_pierce_pct {
            continue;
        }
        if close >= lvl {
            continue;
        }
        best = select_best(
            best,
            LevelPierce {
                level_name: name,
                level_value: lvl,
                pierce_pct,
            },
        );
    }
    best
}

/// Mirror of `pierced_resistance` for supports.
#[inline]
pub fn pierced_support(
    close: f64,
    low: f64,
    levels: &KeyLevels,
    params: &LevelPierceParams,
) -> Option<LevelPierce> {
    if close.is_nan() || low.is_nan() {
        return None;
    }
    let candidates: [(&'static str, Option<f64>); 3] = [
        ("pdl", levels.pdl),
        ("prev_week_low", levels.prev_week_low),
        ("prev_month_low", levels.prev_month_low),
    ];
    let mut best: Option<LevelPierce> = None;
    for (name, maybe) in candidates {
        let lvl = match maybe {
            Some(v) if v > 0.0 => v,
            _ => continue,
        };
        let pierce_pct = (lvl - low) / lvl;
        if pierce_pct < params.min_pierce_pct || pierce_pct > params.max_pierce_pct {
            continue;
        }
        if close <= lvl {
            continue;
        }
        best = select_best(
            best,
            LevelPierce {
                level_name: name,
                level_value: lvl,
                pierce_pct,
            },
        );
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;

    fn params() -> LevelPierceParams {
        LevelPierceParams {
            min_pierce_pct: 0.0005,
            max_pierce_pct: 0.010,
        }
    }

    fn levels() -> KeyLevels {
        KeyLevels {
            pdh: Some(100.0),
            prev_week_high: Some(100.0),
            prev_month_high: Some(102.0),
            pdl: Some(90.0),
            prev_week_low: Some(90.0),
            prev_month_low: Some(89.0),
            ..Default::default()
        }
    }

    #[test]
    fn resistance_picks_highest_tier_on_tie() {
        // High wicks above both pdh and prev_week_high (both 100) and
        // closes below.
        let p = pierced_resistance(99.5, 100.5, &levels(), &params()).unwrap();
        // Monthly isn't pierced here (102 > 100.5). Between PDH and
        // week, week wins by tier.
        assert_eq!(p.level_name, "prev_week_high");
        assert!((p.level_value - 100.0).abs() < 1e-12);
    }

    #[test]
    fn resistance_rejects_non_rejection_close() {
        // Close above the level — not a rejection, just a break-above.
        let p = pierced_resistance(100.5, 100.6, &levels(), &params());
        assert!(p.is_none());
    }

    #[test]
    fn resistance_rejects_overshoot_too_large() {
        // 5% above -> structural break, not rejection.
        let p = pierced_resistance(99.0, 105.0, &levels(), &params());
        assert!(p.is_none());
    }

    #[test]
    fn support_mirror_picks_highest_tier() {
        // Low wicks below monthly (89), closes above.
        let p = pierced_support(89.5, 88.95, &levels(), &params()).unwrap();
        assert_eq!(p.level_name, "prev_month_low");
    }

    #[test]
    fn support_rejects_undershoot_too_large() {
        let p = pierced_support(89.5, 60.0, &levels(), &params());
        assert!(p.is_none());
    }
}
