//! Feature matrix builder for BTC structure.
//!
//! Consumes the raw engine output, attributes each structure break to the
//! causally-valid confirmed level that was broken, and emits one feature-row
//! per bar that the strategy can read via `DailyStructureProvider::latest()`
//! or `::feature_matrix()`.

use std::collections::HashMap;

use crate::engine::{FeatureValue, StructureArtifacts};
use crate::ranking;

/// Result of the structure feature lab.
#[derive(Debug, Clone, Default)]
pub struct StructureLabArtifacts {
    pub feature_matrix: Vec<HashMap<&'static str, FeatureValue>>,
}

/// Run the full structure feature lab: rank breaks, build feature matrix.
pub fn run_structure_feature_lab(structure: &StructureArtifacts) -> StructureLabArtifacts {
    let ranked_breaks = ranking::rank_structure_breaks(
        &structure.structure_breaks,
        &structure.confirmed_highs,
        &structure.confirmed_lows,
    );

    let feature_matrix = build_feature_matrix(structure, &ranked_breaks);
    StructureLabArtifacts { feature_matrix }
}

/// Build the feature matrix from engine rows + ranked data.
fn build_feature_matrix(
    structure: &StructureArtifacts,
    ranked_breaks: &[ranking::RankedBreak],
) -> Vec<HashMap<&'static str, FeatureValue>> {
    let mut matrix = Vec::with_capacity(structure.feature_rows.len());

    // Track last break direction for regime features
    let mut last_major_break_bullish: Option<bool> = None;
    let mut last_global_break_bullish: Option<bool> = None;

    let break_by_bar: HashMap<usize, &ranking::RankedBreak> =
        ranked_breaks.iter().map(|b| (b.bar_index, b)).collect();

    for (i, engine_row) in structure.feature_rows.iter().enumerate() {
        let mut row = engine_row.clone();

        // Update break tracking
        if let Some(brk) = break_by_bar.get(&i) {
            let is_bullish = brk.event.contains("up");
            if brk.is_major_break {
                last_major_break_bullish = Some(is_bullish);
            }
            if brk.broken_level_scope == "global" {
                last_global_break_bullish = Some(is_bullish);
            }
        }

        // Regime features
        row.insert(
            "major_last_break_is_bullish",
            match last_major_break_bullish {
                Some(b) => FeatureValue::Bool(b),
                None => FeatureValue::Null,
            },
        );
        row.insert(
            "global_last_break_is_bullish",
            match last_global_break_bullish {
                Some(b) => FeatureValue::Bool(b),
                None => FeatureValue::Null,
            },
        );

        // Confluence flags
        let major_bullish = last_major_break_bullish.unwrap_or(false);
        let global_bullish = last_global_break_bullish.unwrap_or(false);
        row.insert(
            "major_global_bullish_confluence_flag",
            FeatureValue::Bool(major_bullish && global_bullish),
        );
        row.insert(
            "major_global_bearish_confluence_flag",
            FeatureValue::Bool(
                !major_bullish && !global_bullish && last_major_break_bullish.is_some(),
            ),
        );

        // Continuation flags (simplified — full version uses Fib positions)
        let bias = engine_row
            .get("market_bias_after_close")
            .and_then(|v| match v {
                FeatureValue::Str(s) => Some(s.as_ref()),
                _ => None,
            })
            .unwrap_or("neutral");
        row.insert(
            "global_continuation_long_flag",
            FeatureValue::Bool(bias == "bullish" && global_bullish),
        );
        row.insert(
            "global_continuation_short_flag",
            FeatureValue::Bool(
                bias == "bearish" && !global_bullish && last_global_break_bullish.is_some(),
            ),
        );

        matrix.push(row);
    }

    matrix
}
