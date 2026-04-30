//! Strategy evaluation pipeline — Rust port of `backtester/evaluator.py`.
//!
//! Aggregates resolved trades across multiple time windows and computes
//! category summaries with Sortino, preference scores, and drawdown.
//! Cooldown is applied upstream by the runtime (`cooldown` submodule); the
//! aggregation here does not filter trades.

pub mod calendar;
pub mod cooldown;
pub mod scoring;
pub mod windows;

use std::collections::HashMap;

use claude_trader_models::{
    CategorySummary, EvalWindow, EvaluationReport, PortfolioConfig, TradeResult, WindowResult,
};
use claude_trader_resolver::stats::compute_stats;

use scoring::{build_category_summary, overall_summary};

/// Aggregate already-resolved trades into the final `EvaluationReport`.
///
/// Pure aggregation: no cooldown is applied here. Cooldown is the runtime's
/// responsibility and has already filtered the signal stream that produced
/// these trades (see `crates/evaluator/src/cooldown.rs`).
pub fn build_evaluation_report(
    window_trades: Vec<(EvalWindow, Vec<TradeResult>)>,
    config: &PortfolioConfig,
    symbols: &[String],
) -> EvaluationReport {
    // Build window results
    let mut window_results = Vec::new();

    for (window, trades) in window_trades {
        let signal_count = trades.len();
        let short_count = trades
            .iter()
            .filter(|t| !t.signal.position_type.is_long())
            .count();
        let long_count = signal_count - short_count;

        let backtest = compute_stats(&trades);

        window_results.push(WindowResult {
            window,
            backtest,
            signal_count,
            short_count,
            long_count,
        });
    }

    EvaluationReport {
        window_results,
        config: config.clone(),
        symbols: symbols.to_vec(),
    }
}

/// Group windows by category and compute category summaries.
pub fn category_summaries(report: &EvaluationReport) -> Vec<CategorySummary> {
    let by_cat = group_by_category(&report.window_results);
    let mut summaries: Vec<CategorySummary> = by_cat
        .into_iter()
        .map(|(cat, wrs)| build_category_summary(&cat, &wrs, &report.config))
        .collect();
    summaries.sort_by(|a, b| {
        let (ka, kb) = (a.preference_score, b.preference_score);
        // NaN sorts last regardless of ordering direction — a NaN score is
        // an invalid category that shouldn't displace a real one. For finite
        // values, use total_cmp (descending — higher score first).
        let cmp = match (ka.is_nan(), kb.is_nan()) {
            (true, true) => std::cmp::Ordering::Equal,
            (true, false) => std::cmp::Ordering::Greater,
            (false, true) => std::cmp::Ordering::Less,
            (false, false) => kb.total_cmp(&ka),
        };
        cmp.then_with(|| a.category.cmp(&b.category))
    });
    summaries
}

/// Compute a single overall summary across all windows.
pub fn compute_overall_summary(report: &EvaluationReport) -> CategorySummary {
    overall_summary(&report.window_results, &report.config)
}

/// Group window results by category.
fn group_by_category(results: &[WindowResult]) -> HashMap<String, Vec<&WindowResult>> {
    let mut groups: HashMap<String, Vec<&WindowResult>> = HashMap::new();
    for wr in results {
        groups
            .entry(wr.window.category.clone())
            .or_default()
            .push(wr);
    }
    groups
}

#[cfg(test)]
mod nan_sort_tests {
    use super::*;

    fn cat(category: &str, preference_score: f64) -> CategorySummary {
        CategorySummary {
            category: category.to_string(),
            preference_score,
            ..Default::default()
        }
    }

    /// NaN preference_score must sort last regardless of alphabetical
    /// name ordering — otherwise a NaN category named "aaa" would
    /// displace a real category named "zzz" through the alphabetical
    /// tiebreak.
    #[test]
    fn nan_preference_score_sorts_last() {
        // Direct test of the sort predicate used in `category_summaries`.
        let mut summaries = vec![
            cat("aaa_nan", f64::NAN),
            cat("zzz_real", 1.0),
            cat("mmm_real", 5.0),
            cat("bbb_nan", f64::NAN),
        ];

        summaries.sort_by(|a, b| {
            let (ka, kb) = (a.preference_score, b.preference_score);
            let cmp = match (ka.is_nan(), kb.is_nan()) {
                (true, true) => std::cmp::Ordering::Equal,
                (true, false) => std::cmp::Ordering::Greater,
                (false, true) => std::cmp::Ordering::Less,
                (false, false) => kb.total_cmp(&ka),
            };
            cmp.then_with(|| a.category.cmp(&b.category))
        });

        let order: Vec<&str> = summaries.iter().map(|s| s.category.as_str()).collect();
        assert_eq!(
            order,
            vec!["mmm_real", "zzz_real", "aaa_nan", "bbb_nan"],
            "NaN categories must sort after all real ones, real values descending"
        );
    }
}
