//! Structure-break ranking by scope and significance.

use crate::engine::{ConfirmedLevel, StructureBreak};

/// A structure break with ranking metadata.
///
/// `broken_level_score` and `is_strategy_break` are populated by
/// `rank_structure_breaks` and pinned by unit tests, but not yet read by
/// `build_feature_matrix`. They're kept on the struct as the ranker's
/// observable classification so downstream consumers (and the tests that
/// gate ranker correctness) can assert on them.
#[derive(Debug, Clone)]
pub struct RankedBreak {
    pub bar_index: usize,
    pub event: String,
    pub broken_level_scope: String,
    #[allow(dead_code)]
    pub broken_level_score: f64,
    pub is_major_break: bool,
    #[allow(dead_code)]
    pub is_strategy_break: bool,
}

/// Rank structure breaks by the broken level's scope and significance.
///
/// A break is attributed to the confirmed level whose `value` matches
/// `brk.broken_level` and whose `confirmation_bar_index` is strictly less
/// than `brk.bar_index`. This is the causal invariant: a break at bar `k`
/// can only be attributed to a level that was already *confirmed* (and
/// therefore observable) strictly before bar `k`.
///
/// The previous implementation used the swing bar index, which admitted
/// levels whose swing was before the break but whose confirmation happened
/// afterwards — a lookahead. If no level matches, the scope is `"unknown"`
/// and the break is marked unranked.
pub fn rank_structure_breaks(
    breaks: &[StructureBreak],
    confirmed_highs: &[ConfirmedLevel],
    confirmed_lows: &[ConfirmedLevel],
) -> Vec<RankedBreak> {
    breaks
        .iter()
        .map(|brk| {
            let is_up = brk.event.contains("up");
            let levels = if is_up {
                confirmed_highs
            } else {
                confirmed_lows
            };

            // Python `ranking.py` uses `np.isclose(source["value"], value, atol=1e-9)`.
            // We use the same absolute tolerance plus a relative guard for
            // large prices; in practice engine values are assigned bit-exact
            // from the source bars so this rarely matters.
            let value_tol = 1e-9_f64.max(brk.broken_level.abs() * 1e-9);

            let broken_level_info = levels
                .iter()
                .filter(|l| {
                    l.confirmation_bar_index < brk.bar_index
                        && (l.value - brk.broken_level).abs() <= value_tol
                })
                .max_by_key(|l| l.confirmation_bar_index);

            let (scope, score) = match broken_level_info {
                Some(level) => {
                    let max_w = level.confluence_windows.iter().copied().max().unwrap_or(0);
                    scope_from_max_window(max_w)
                }
                None => ("unknown", f64::NAN),
            };

            let is_major = scope == "major" || scope == "global";
            // A choch against any *known* non-local level qualifies; unknown
            // scopes stay out of the strategy set.
            let is_strategy = is_major
                || (brk.event.starts_with("choch") && scope != "local" && scope != "unknown");

            RankedBreak {
                bar_index: brk.bar_index,
                event: brk.event.clone(),
                broken_level_scope: scope.to_string(),
                broken_level_score: score,
                is_major_break: is_major,
                is_strategy_break: is_strategy,
            }
        })
        .collect()
}

/// Determine scope and base score from the maximum confluence window.
fn scope_from_max_window(max_window: usize) -> (&'static str, f64) {
    if max_window >= 180 {
        ("global", 3.0)
    } else if max_window >= 90 {
        ("major", 2.0)
    } else if max_window >= 30 {
        ("structural", 1.0)
    } else {
        ("local", 0.5)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn level(
        swing_bar: usize,
        confirmation_bar: usize,
        value: f64,
        windows: Vec<usize>,
    ) -> ConfirmedLevel {
        ConfirmedLevel {
            bar_index: swing_bar,
            date: chrono::Utc
                .timestamp_opt(1_700_000_000 + confirmation_bar as i64 * 86_400, 0)
                .unwrap(),
            confirmation_bar_index: confirmation_bar,
            value,
            label: "HH".to_string(),
            confluence_count: windows.len(),
            confluence_windows: windows,
            bars_to_confirmation: confirmation_bar.saturating_sub(swing_bar),
            breaks_structure: false,
        }
    }

    fn break_evt(bar: usize, event: &str, broken_value: f64) -> StructureBreak {
        StructureBreak {
            bar_index: bar,
            date: chrono::Utc
                .timestamp_opt(1_700_000_000 + bar as i64 * 86_400, 0)
                .unwrap(),
            event: event.to_string(),
            close: broken_value + 1.0,
            broken_level: broken_value,
            excursion: 0.01,
        }
    }

    // Dormant-bug regression: the A/B/22 scenario from the fix plan.
    //
    // Level A: swing at bar 10, confirmed at bar 13, value 100.0, global scope.
    // Level B: swing at bar 20, confirmed at bar 25, value 105.0, structural scope.
    // Break  : at bar 22 against A (close > 100).
    //
    // The pre-fix ranker iterated levels in reverse insertion order and used
    // `l.bar_index < brk.bar_index`, which picked B (swing 20 < 22) even
    // though B wasn't confirmed until bar 25. The fixed ranker must match
    // by value AND confirmation-bar < break-bar, attributing the break to A.
    #[test]
    fn rank_break_uses_confirmation_bar_not_swing_bar() {
        let a = level(10, 13, 100.0, vec![30, 90, 180]); // global
        let b = level(20, 25, 105.0, vec![30, 90]);      // major
        let brk = break_evt(22, "bos_up", 100.0);

        let ranked = rank_structure_breaks(&[brk], &[a, b], &[]);
        assert_eq!(ranked.len(), 1);
        let r = &ranked[0];
        assert_eq!(r.broken_level_scope, "global",
            "break at bar 22 was against level A (value 100, global); \
             fixed ranker must not look ahead to B (confirmed at bar 25).");
        assert!(r.is_major_break);
        assert!(r.is_strategy_break);
    }

    // Value-match: even if two levels are confirmed before the break, only
    // the one whose value equals `brk.broken_level` should be picked.
    #[test]
    fn rank_break_matches_by_value() {
        let a = level(10, 13, 100.0, vec![30, 90, 180]); // global, value 100
        let c = level(15, 18, 99.0,  vec![30, 90]);      // major,  value 99
        let brk = break_evt(22, "bos_up", 100.0);         // broken value = 100

        let ranked = rank_structure_breaks(&[brk], &[a, c], &[]);
        assert_eq!(ranked[0].broken_level_scope, "global");
    }

    // When no level matches (e.g. engine ran on a trimmed window), the
    // ranker must report `unknown` rather than invent a plausible match.
    #[test]
    fn rank_break_unknown_when_no_value_match() {
        let unrelated = level(10, 13, 200.0, vec![30, 90, 180]);
        let brk = break_evt(22, "bos_up", 100.0);

        let ranked = rank_structure_breaks(&[brk], &[unrelated], &[]);
        assert_eq!(ranked[0].broken_level_scope, "unknown");
        assert!(ranked[0].broken_level_score.is_nan());
        assert!(!ranked[0].is_major_break);
        assert!(!ranked[0].is_strategy_break);
    }

    // Levels confirmed after the break must not be considered at all.
    #[test]
    fn rank_break_ignores_future_confirmations() {
        // Same value as the break, but confirmed AT or AFTER the break bar.
        let future_a = level(10, 22, 100.0, vec![30, 90, 180]);
        let future_b = level(10, 30, 100.0, vec![30, 90, 180]);
        let brk = break_evt(22, "bos_up", 100.0);

        let ranked = rank_structure_breaks(&[brk], &[future_a, future_b], &[]);
        assert_eq!(ranked[0].broken_level_scope, "unknown",
            "levels confirmed at or after the break bar are not observable.");
    }
}
