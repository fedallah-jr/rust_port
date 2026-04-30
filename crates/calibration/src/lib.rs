//! Calibration runtime — grid search over parameter spaces.
//!
//! Rust port of `backtester/calibration.py`. Implements exhaustive grid
//! search with parallel scoring via rayon.

use std::collections::HashMap;

use claude_trader_models::CalibrationResult;
use rayon::prelude::*;

/// Maximum number of parameter combinations before erroring.
const MAX_COMBINATIONS: usize = 50_000;

/// Search for the best parameters via exhaustive grid search.
///
/// - `param_space`: maps parameter names to candidate value lists.
/// - `score_fn`: scores a parameter set (higher = better). Return `None` to
///   skip a candidate. **Called concurrently from multiple threads** — must
///   be `Sync`. Capture any per-search context (candles, HTF data, a
///   precomputed indicator table, etc.) in the closure.
///
/// Returns the best parameters and score, or `None` if the search fails or
/// every candidate returned `None`. On ties, the candidate with the smallest
/// original index wins (deterministic).
pub fn search_parameters<F>(
    param_space: &HashMap<String, Vec<serde_json::Value>>,
    score_fn: F,
) -> Option<CalibrationResult>
where
    F: Fn(&HashMap<String, serde_json::Value>) -> Option<f64> + Sync,
{
    if param_space.is_empty() {
        return None;
    }
    run_search(param_space, &score_fn)
}

/// Core parallel grid search. Entered via [`search_parameters`] after the
/// caller's preconditions are satisfied.
fn run_search<F>(
    param_space: &HashMap<String, Vec<serde_json::Value>>,
    score_fn: &F,
) -> Option<CalibrationResult>
where
    F: Fn(&HashMap<String, serde_json::Value>) -> Option<f64> + Sync,
{
    let keys = sorted_keys(param_space);
    let total = total_combinations(param_space, &keys);
    if total > MAX_COMBINATIONS {
        eprintln!(
            "WARNING: param_space produces {} combinations (limit is {}). Returning None.",
            total, MAX_COMBINATIONS
        );
        return None;
    }

    let candidates = generate_candidates(param_space, &keys);
    if candidates.is_empty() {
        return None;
    }

    // Parallel scoring + reduction in one pass. The `ScoreAcc` accumulator
    // preserves the sequential semantics exactly:
    //   * `evaluated` counts every candidate whose `score_fn` returned `Some`,
    //     including `NaN`.
    //   * `best_idx` / `best_score` track the highest non-NaN score, breaking
    //     ties in favor of the smaller original index.
    //   * `NaN` scores never displace a finite best (NaN > x is always false),
    //     matching the original `score > best_score` guard.
    let acc = candidates
        .par_iter()
        .enumerate()
        .map(|(idx, params)| ScoreAcc::from_candidate(score_fn(params), idx))
        .reduce(ScoreAcc::identity, ScoreAcc::merge);

    if acc.evaluated == 0 {
        return None;
    }

    let best_params = match acc.best_idx {
        Some(idx) => candidates[idx].clone(),
        None => HashMap::new(),
    };

    Some(CalibrationResult {
        best_params,
        best_score: acc.best_score,
        candidates_evaluated: acc.evaluated,
    })
}

/// Accumulator for the parallel calibration reduce. `merge` is associative so
/// rayon can fold results from arbitrary thread shards while preserving the
/// deterministic "lowest original index wins on ties" rule.
#[derive(Clone, Copy)]
struct ScoreAcc {
    best_score: f64,
    best_idx: Option<usize>,
    evaluated: usize,
}

impl ScoreAcc {
    fn identity() -> Self {
        Self {
            best_score: f64::NEG_INFINITY,
            best_idx: None,
            evaluated: 0,
        }
    }

    fn from_candidate(score: Option<f64>, idx: usize) -> Self {
        match score {
            Some(s) if !s.is_nan() => Self {
                best_score: s,
                best_idx: Some(idx),
                evaluated: 1,
            },
            Some(_) => Self {
                // NaN counts toward `evaluated` but never becomes best.
                best_score: f64::NEG_INFINITY,
                best_idx: None,
                evaluated: 1,
            },
            None => Self::identity(),
        }
    }

    fn merge(self, other: Self) -> Self {
        let evaluated = self.evaluated + other.evaluated;
        match (self.best_idx, other.best_idx) {
            (None, None) => Self {
                best_score: f64::NEG_INFINITY,
                best_idx: None,
                evaluated,
            },
            (Some(_), None) => Self { evaluated, ..self },
            (None, Some(_)) => Self { evaluated, ..other },
            (Some(ai), Some(bi)) => {
                let pick_self = if self.best_score > other.best_score {
                    true
                } else if other.best_score > self.best_score {
                    false
                } else {
                    // Equal scores — smaller original index wins.
                    ai < bi
                };
                if pick_self {
                    Self { evaluated, ..self }
                } else {
                    Self { evaluated, ..other }
                }
            }
        }
    }
}

fn sorted_keys(param_space: &HashMap<String, Vec<serde_json::Value>>) -> Vec<String> {
    let mut keys: Vec<String> = param_space.keys().cloned().collect();
    keys.sort();
    keys
}

fn total_combinations(
    param_space: &HashMap<String, Vec<serde_json::Value>>,
    keys: &[String],
) -> usize {
    keys.iter()
        .map(|k| param_space.get(k).map(|v| v.len()).unwrap_or(0))
        .product()
}

/// Generate all parameter combinations from a param_space.
fn generate_candidates(
    param_space: &HashMap<String, Vec<serde_json::Value>>,
    keys: &[String],
) -> Vec<HashMap<String, serde_json::Value>> {
    let values: Vec<&Vec<serde_json::Value>> = keys
        .iter()
        .map(|k| param_space.get(k).expect("missing param space entry"))
        .collect();

    let mut result = Vec::new();
    let mut indices = vec![0usize; keys.len()];

    if keys.is_empty() || values.iter().any(|v| v.is_empty()) {
        return result;
    }

    loop {
        // Build current combination
        let mut combo = HashMap::new();
        for (i, key) in keys.iter().enumerate() {
            combo.insert(key.clone(), values[i][indices[i]].clone());
        }
        result.push(combo);

        // Increment indices (odometer pattern)
        let mut carry = true;
        for i in (0..keys.len()).rev() {
            if carry {
                indices[i] += 1;
                if indices[i] >= values[i].len() {
                    indices[i] = 0;
                } else {
                    carry = false;
                }
            }
        }
        if carry {
            break; // All combinations exhausted
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_candidates() {
        let mut space = HashMap::new();
        space.insert(
            "a".to_string(),
            vec![serde_json::json!(1), serde_json::json!(2)],
        );
        space.insert(
            "b".to_string(),
            vec![
                serde_json::json!(10),
                serde_json::json!(20),
                serde_json::json!(30),
            ],
        );

        let keys = sorted_keys(&space);
        let candidates = generate_candidates(&space, &keys);
        assert_eq!(candidates.len(), 6); // 2 * 3
    }

    #[test]
    fn test_search_parameters() {
        let mut space = HashMap::new();
        space.insert(
            "threshold".to_string(),
            vec![
                serde_json::json!(10.0),
                serde_json::json!(20.0),
                serde_json::json!(30.0),
            ],
        );

        let result = search_parameters(&space, |params| {
            let t = params["threshold"].as_f64()?;
            Some(100.0 - (t - 25.0).abs()) // Best at 25, closest is 20
        })
        .unwrap();

        assert_eq!(result.best_params["threshold"].as_f64().unwrap(), 20.0);
        assert_eq!(result.candidates_evaluated, 3);
    }

    #[test]
    fn test_search_parameters_captures_context() {
        // Verify the closure captures caller-owned context for custom scoring.
        struct Ctx {
            target: f64,
        }
        let ctx = Ctx { target: 42.0 };

        let mut space = HashMap::new();
        space.insert(
            "x".to_string(),
            vec![
                serde_json::json!(10.0),
                serde_json::json!(42.0),
                serde_json::json!(99.0),
            ],
        );

        let result = search_parameters(&space, |params| {
            let x = params["x"].as_f64()?;
            Some(-(x - ctx.target).abs())
        })
        .unwrap();

        assert_eq!(result.best_params["x"].as_f64().unwrap(), 42.0);
    }

    #[test]
    fn test_tie_break_keeps_first_candidate() {
        let mut space = HashMap::new();
        space.insert(
            "a".to_string(),
            vec![serde_json::json!(1), serde_json::json!(2)],
        );

        let result = search_parameters(&space, |_params| Some(10.0)).unwrap();
        assert_eq!(result.best_params["a"], serde_json::json!(1));
    }

    #[test]
    fn test_nan_scores_match_python_behavior() {
        let mut space = HashMap::new();
        space.insert(
            "a".to_string(),
            vec![serde_json::json!(1), serde_json::json!(2)],
        );

        let result = search_parameters(&space, |_params| Some(f64::NAN)).unwrap();
        assert_eq!(result.best_params, HashMap::new());
        assert_eq!(result.best_score, f64::NEG_INFINITY);
        assert_eq!(result.candidates_evaluated, 2);
    }

    #[test]
    fn test_max_combinations_guard_returns_none() {
        let mut space = HashMap::new();
        space.insert(
            "a".to_string(),
            (0..251).map(serde_json::Value::from).collect(),
        );
        space.insert(
            "b".to_string(),
            (0..200).map(serde_json::Value::from).collect(),
        );

        let result = search_parameters(&space, |_params| Some(1.0));
        assert!(result.is_none(), "Expected None for oversized param space");
    }

    #[test]
    fn test_empty_param_space_returns_none() {
        let space: HashMap<String, Vec<serde_json::Value>> = HashMap::new();
        assert!(search_parameters(&space, |_params| Some(1.0)).is_none());
    }

}
