//! Indicator specification registry — dependencies, warmup requirements,
//! and topological ordering.

use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

/// Raw OHLCV column names that don't need computation.
pub const RAW_INPUTS: &[&str] = &["open", "high", "low", "close", "volume", "taker_buy_volume"];

/// Specification for a single indicator.
#[derive(Debug, Clone)]
pub struct IndicatorSpec {
    pub name: &'static str,
    pub dependencies: &'static [&'static str],
    pub additional_bars: usize,
}

/// All known indicator specifications.
pub static INDICATOR_SPECS: &[IndicatorSpec] = &[
    // Internal: delta close
    IndicatorSpec {
        name: "_delta_close",
        dependencies: &["close"],
        additional_bars: 1,
    },
    IndicatorSpec {
        name: "_gain_ewm_14",
        dependencies: &["_delta_close"],
        additional_bars: 13,
    },
    IndicatorSpec {
        name: "_loss_ewm_14",
        dependencies: &["_delta_close"],
        additional_bars: 13,
    },
    IndicatorSpec {
        name: "rsi_14",
        dependencies: &["_gain_ewm_14", "_loss_ewm_14"],
        additional_bars: 0,
    },
    // True range + ATR
    IndicatorSpec {
        name: "true_range",
        dependencies: &["high", "low", "close"],
        additional_bars: 0,
    },
    IndicatorSpec {
        name: "atr_14",
        dependencies: &["true_range"],
        additional_bars: 13,
    },
    IndicatorSpec {
        name: "atr_72_avg",
        dependencies: &["atr_14"],
        additional_bars: 71,
    },
    IndicatorSpec {
        name: "atr_ratio",
        dependencies: &["atr_14", "atr_72_avg"],
        additional_bars: 0,
    },
    // Returns
    IndicatorSpec {
        name: "ret_24h",
        dependencies: &["close"],
        additional_bars: 24,
    },
    IndicatorSpec {
        name: "ret_48h",
        dependencies: &["close"],
        additional_bars: 48,
    },
    IndicatorSpec {
        name: "ret_72h",
        dependencies: &["close"],
        additional_bars: 72,
    },
    // Volume
    IndicatorSpec {
        name: "vol_sma_20",
        dependencies: &["volume"],
        additional_bars: 19,
    },
    IndicatorSpec {
        name: "vol_ratio",
        dependencies: &["volume", "vol_sma_20"],
        additional_bars: 0,
    },
    // Bollinger Bands
    IndicatorSpec {
        name: "_bb_ma_20",
        dependencies: &["close"],
        additional_bars: 19,
    },
    IndicatorSpec {
        name: "_bb_std_20",
        dependencies: &["close"],
        additional_bars: 19,
    },
    IndicatorSpec {
        name: "bb_upper",
        dependencies: &["_bb_ma_20", "_bb_std_20"],
        additional_bars: 0,
    },
    IndicatorSpec {
        name: "bb_lower",
        dependencies: &["_bb_ma_20", "_bb_std_20"],
        additional_bars: 0,
    },
    IndicatorSpec {
        name: "bb_pct_b",
        dependencies: &["close", "bb_upper", "bb_lower"],
        additional_bars: 0,
    },
    IndicatorSpec {
        name: "bb_width",
        dependencies: &["bb_upper", "bb_lower", "_bb_ma_20"],
        additional_bars: 0,
    },
    // EMA
    IndicatorSpec {
        name: "ema_20",
        dependencies: &["close"],
        additional_bars: 0,
    },
    // Keltner Channel
    IndicatorSpec {
        name: "kc_upper",
        dependencies: &["ema_20", "atr_14"],
        additional_bars: 0,
    },
    IndicatorSpec {
        name: "kc_lower",
        dependencies: &["ema_20", "atr_14"],
        additional_bars: 0,
    },
    // Squeeze
    IndicatorSpec {
        name: "squeeze_on",
        dependencies: &["bb_upper", "bb_lower", "kc_upper", "kc_lower"],
        additional_bars: 0,
    },
    IndicatorSpec {
        name: "squeeze_count",
        dependencies: &["squeeze_on"],
        additional_bars: 0,
    },
    // Momentum slope
    IndicatorSpec {
        name: "mom_slope",
        dependencies: &["close"],
        additional_bars: 19,
    },
    // Body
    IndicatorSpec {
        name: "body",
        dependencies: &["open", "close"],
        additional_bars: 0,
    },
    IndicatorSpec {
        name: "body_ratio",
        dependencies: &["body", "high", "low"],
        additional_bars: 0,
    },
    // ADX
    IndicatorSpec {
        name: "_plus_dm",
        dependencies: &["high", "low"],
        additional_bars: 1,
    },
    IndicatorSpec {
        name: "_minus_dm",
        dependencies: &["high", "low"],
        additional_bars: 1,
    },
    IndicatorSpec {
        name: "_smoothed_plus_dm",
        dependencies: &["_plus_dm"],
        additional_bars: 13,
    },
    IndicatorSpec {
        name: "_smoothed_minus_dm",
        dependencies: &["_minus_dm"],
        additional_bars: 13,
    },
    IndicatorSpec {
        name: "_smoothed_tr",
        dependencies: &["true_range"],
        additional_bars: 13,
    },
    IndicatorSpec {
        name: "_plus_di",
        dependencies: &["_smoothed_plus_dm", "_smoothed_tr"],
        additional_bars: 0,
    },
    IndicatorSpec {
        name: "_minus_di",
        dependencies: &["_smoothed_minus_dm", "_smoothed_tr"],
        additional_bars: 0,
    },
    IndicatorSpec {
        name: "_dx",
        dependencies: &["_plus_di", "_minus_di"],
        additional_bars: 0,
    },
    IndicatorSpec {
        name: "adx_14",
        dependencies: &["_dx"],
        additional_bars: 13,
    },
    // CVD
    IndicatorSpec {
        name: "volume_delta",
        dependencies: &["taker_buy_volume", "volume"],
        additional_bars: 0,
    },
    IndicatorSpec {
        name: "cvd",
        dependencies: &["volume_delta"],
        additional_bars: 0,
    },
    // Rolling VWAP (typical-price volume-weighted average over N bars).
    // vwap_N dependencies use HLC+volume. additional_bars = window - 1.
    IndicatorSpec {
        name: "_typ_price",
        dependencies: &["high", "low", "close"],
        additional_bars: 0,
    },
    IndicatorSpec {
        name: "vwap_20",
        dependencies: &["_typ_price", "volume"],
        additional_bars: 19,
    },
    IndicatorSpec {
        name: "vwap_48",
        dependencies: &["_typ_price", "volume"],
        additional_bars: 47,
    },
    IndicatorSpec {
        name: "vwap_dev_20",
        dependencies: &["close", "vwap_20"],
        additional_bars: 0,
    },
    IndicatorSpec {
        name: "vwap_dev_48",
        dependencies: &["close", "vwap_48"],
        additional_bars: 0,
    },

    // Point of Control: price bin with the highest traded volume over a
    // rolling window. Bars are attributed to the bin containing their
    // close. `poc_dev_N` is percent deviation of current close from POC.
    IndicatorSpec {
        name: "poc_48",
        dependencies: &["high", "low", "close", "volume"],
        additional_bars: 47,
    },
    IndicatorSpec {
        name: "poc_dev_48",
        dependencies: &["close", "poc_48"],
        additional_bars: 0,
    },

    // T3
    IndicatorSpec {
        name: "_t3_e1",
        dependencies: &["close"],
        additional_bars: 0,
    },
    IndicatorSpec {
        name: "_t3_e2",
        dependencies: &["_t3_e1"],
        additional_bars: 0,
    },
    IndicatorSpec {
        name: "_t3_e3",
        dependencies: &["_t3_e2"],
        additional_bars: 0,
    },
    IndicatorSpec {
        name: "_t3_e4",
        dependencies: &["_t3_e3"],
        additional_bars: 0,
    },
    IndicatorSpec {
        name: "_t3_e5",
        dependencies: &["_t3_e4"],
        additional_bars: 0,
    },
    IndicatorSpec {
        name: "_t3_e6",
        dependencies: &["_t3_e5"],
        additional_bars: 0,
    },
    IndicatorSpec {
        name: "t3",
        dependencies: &["_t3_e3", "_t3_e4", "_t3_e5", "_t3_e6"],
        additional_bars: 0,
    },
];

/// Look up an indicator spec by name.
pub fn get_spec(name: &str) -> Option<&'static IndicatorSpec> {
    spec_index().get(name).copied()
}

/// Lazily built name → spec index. One-time O(N) construction, O(1) lookup.
fn spec_index() -> &'static HashMap<&'static str, &'static IndicatorSpec> {
    static INDEX: OnceLock<HashMap<&'static str, &'static IndicatorSpec>> = OnceLock::new();
    INDEX.get_or_init(|| INDICATOR_SPECS.iter().map(|s| (s.name, s)).collect())
}

/// Resolve topological order for computing the requested indicators.
pub fn resolve_order(indicators: &[&str]) -> Result<Vec<&'static str>, String> {
    let raw: HashSet<&str> = RAW_INPUTS.iter().copied().collect();
    let mut order = Vec::new();
    let mut visited = HashSet::new();
    let mut visiting = HashSet::new();

    for &ind in indicators {
        visit(ind, &raw, &mut order, &mut visited, &mut visiting)?;
    }

    Ok(order)
}

fn visit<'a>(
    name: &'a str,
    raw: &HashSet<&str>,
    order: &mut Vec<&'static str>,
    visited: &mut HashSet<&'a str>,
    visiting: &mut HashSet<&'a str>,
) -> Result<(), String> {
    if raw.contains(name) || visited.contains(name) {
        return Ok(());
    }
    if visiting.contains(name) {
        return Err(format!("Cyclic dependency detected for indicator: {name}"));
    }
    visiting.insert(name);

    if let Some(spec) = get_spec(name) {
        for &dep in spec.dependencies {
            visit(dep, raw, order, visited, visiting)?;
        }
        visiting.remove(name);
        visited.insert(name);
        order.push(spec.name);
        Ok(())
    } else {
        Err(format!("Unknown indicator: '{name}'"))
    }
}

/// Compute the minimum warmup bars required for a set of indicators.
pub fn required_warmup(indicators: &[&str]) -> usize {
    let raw: HashSet<&str> = RAW_INPUTS.iter().copied().collect();
    let mut memo: HashMap<&'static str, usize> = HashMap::new();
    indicators
        .iter()
        .map(|&name| required_warmup_single(name, &raw, &mut memo))
        .max()
        .unwrap_or(0)
}

fn required_warmup_single(
    name: &str,
    raw: &HashSet<&str>,
    memo: &mut HashMap<&'static str, usize>,
) -> usize {
    if raw.contains(name) {
        return 1;
    }
    let spec = match get_spec(name) {
        Some(s) => s,
        None => return 0,
    };
    if let Some(&cached) = memo.get(spec.name) {
        return cached;
    }
    let dep_warmup = spec
        .dependencies
        .iter()
        .map(|&dep| required_warmup_single(dep, raw, memo))
        .max()
        .unwrap_or(0);
    let result = dep_warmup + spec.additional_bars;
    memo.insert(spec.name, result);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_warmup_rsi() {
        assert_eq!(required_warmup(&["rsi_14"]), 15);
    }

    #[test]
    fn test_warmup_atr_ratio() {
        assert_eq!(required_warmup(&["atr_ratio"]), 85);
    }

    #[test]
    fn test_warmup_ret_72h() {
        assert_eq!(required_warmup(&["ret_72h"]), 73);
    }

    #[test]
    fn test_warmup_bb() {
        assert_eq!(required_warmup(&["bb_upper"]), 20);
    }

    #[test]
    fn test_warmup_adx() {
        assert_eq!(required_warmup(&["adx_14"]), 28);
    }

    #[test]
    fn test_warmup_combo() {
        // atr_ratio=85 dominates
        assert_eq!(required_warmup(&["rsi_14", "atr_ratio", "ret_72h"]), 85);
    }

    #[test]
    fn test_resolve_order() {
        let order = resolve_order(&["rsi_14"]).unwrap();
        assert!(order.contains(&"_delta_close"));
        assert!(order.contains(&"_gain_ewm_14"));
        assert!(order.contains(&"_loss_ewm_14"));
        assert!(order.contains(&"rsi_14"));
        // rsi_14 must come after its dependencies
        let rsi_pos = order.iter().position(|&x| x == "rsi_14").unwrap();
        let gain_pos = order.iter().position(|&x| x == "_gain_ewm_14").unwrap();
        assert!(gain_pos < rsi_pos);
    }
}
