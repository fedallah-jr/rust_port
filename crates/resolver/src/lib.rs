//! Standalone backtest kernel — entry resolution, exit resolution, TP/SL
//! computation, PnL, timeout resolution, and stats aggregation.
//!
//! This is a pure Rust crate with no PyO3 dependency. The PyO3 glue crate
//! wraps these functions for Python interop.
//!
//! Ported from `resolver_rs/src/lib.rs` (exit resolution) and
//! `backtester/engine.py` + `backtester/resolver.py` (entry, timeout, stats).

use claude_trader_models::Signal;

// Re-export from models so downstream crates (research_runtime) that use
// `claude_trader_resolver::dt_to_ms` / `ms_to_dt` keep compiling.
pub use claude_trader_models::{dt_to_ms, ms_to_dt};

pub mod exit;
pub mod kernel;
pub mod stats;

// ---------------------------------------------------------------------------
// TP/SL price computation
// ---------------------------------------------------------------------------

/// Compute take-profit and stop-loss prices given an entry price and signal
/// parameters.
///
/// Mirrors Python `backtester.resolver.compute_tp_sl_prices()` and
/// `resolver_rs.compute_tp_sl_prices_rs()`.
///
/// Fee offset accounts for round-trip taker fees (entry + exit).
pub fn compute_tp_sl_prices(
    entry_price: f64,
    is_long: bool,
    tp_pct: Option<f64>,
    sl_pct: Option<f64>,
    taker_fee_rate: f64,
    tp_price_override: Option<f64>,
    sl_price_override: Option<f64>,
) -> Result<(f64, f64), String> {
    // Validate overrides are finite when provided
    if let Some(p) = tp_price_override {
        if !p.is_finite() || p <= 0.0 {
            return Err(format!(
                "tp_price_override must be finite and positive, got {p}"
            ));
        }
    }
    if let Some(p) = sl_price_override {
        if !p.is_finite() || p <= 0.0 {
            return Err(format!(
                "sl_price_override must be finite and positive, got {p}"
            ));
        }
    }

    let tp_price = match tp_price_override {
        Some(p) => p,
        None => {
            let pct = tp_pct.ok_or("tp_pct required when tp_price_override is not set")?;
            if !pct.is_finite() || pct <= 0.0 {
                return Err(format!("tp_pct must be finite and positive, got {pct}"));
            }
            let fee = taker_fee_rate * 2.0 * 100.0;
            let with_fees = pct + fee;
            if is_long {
                entry_price * (1.0 + with_fees / 100.0)
            } else {
                entry_price * (1.0 - with_fees / 100.0)
            }
        }
    };
    let sl_price = match sl_price_override {
        Some(p) => p,
        None => {
            let pct = sl_pct.ok_or("sl_pct required when sl_price_override is not set")?;
            if !pct.is_finite() || pct <= 0.0 {
                return Err(format!("sl_pct must be finite and positive, got {pct}"));
            }
            let fee = taker_fee_rate * 2.0 * 100.0;
            let net = pct - fee;
            if net <= 0.0 {
                return Err(format!(
                    "sl_pct ({pct:.4}%) must exceed round-trip fees ({fee:.4}%); requested net SL would be non-positive"
                ));
            }
            if is_long {
                entry_price * (1.0 - net / 100.0)
            } else {
                entry_price * (1.0 + net / 100.0)
            }
        }
    };
    // Validate TP/SL are on the correct side of entry and properly ordered
    if is_long {
        if tp_price <= entry_price {
            return Err(format!(
                "Long TP ({tp_price}) must be above entry ({entry_price})"
            ));
        }
        if sl_price >= entry_price {
            return Err(format!(
                "Long SL ({sl_price}) must be below entry ({entry_price})"
            ));
        }
    } else {
        if tp_price >= entry_price {
            return Err(format!(
                "Short TP ({tp_price}) must be below entry ({entry_price})"
            ));
        }
        if sl_price <= entry_price {
            return Err(format!(
                "Short SL ({sl_price}) must be above entry ({entry_price})"
            ));
        }
    }
    if is_long && tp_price <= sl_price {
        return Err(format!(
            "Long TP ({tp_price}) must be above SL ({sl_price})"
        ));
    }
    if !is_long && tp_price >= sl_price {
        return Err(format!(
            "Short TP ({tp_price}) must be below SL ({sl_price})"
        ));
    }
    Ok((tp_price, sl_price))
}

/// Convenience wrapper that extracts parameters from a Signal.
pub fn compute_tp_sl_prices_from_signal(
    entry_price: f64,
    signal: &Signal,
) -> Result<(f64, f64), String> {
    compute_tp_sl_prices(
        entry_price,
        signal.position_type.is_long(),
        signal.tp_pct,
        signal.sl_pct,
        signal.taker_fee_rate,
        signal.tp_price,
        signal.sl_price,
    )
}

// ---------------------------------------------------------------------------
// PnL computation
// ---------------------------------------------------------------------------

/// Compute net PnL, gross PnL, and fee drag percentages.
///
/// Mirrors Python `backtester.resolver.compute_pnl()` and
/// `resolver_rs.compute_pnl_rs()`.
///
/// Returns `(net_pnl_pct, gross_pnl_pct, fee_drag_pct)`.
pub fn compute_pnl(
    entry_price: f64,
    exit_price: f64,
    is_long: bool,
    leverage: f64,
    taker_fee_rate: f64,
) -> (f64, f64, f64) {
    if entry_price == 0.0 {
        let fee_drag = 2.0 * taker_fee_rate * leverage * 100.0;
        return (0.0 - fee_drag, 0.0, fee_drag);
    }
    let gross = if is_long {
        ((exit_price - entry_price) / entry_price) * 100.0 * leverage
    } else {
        ((entry_price - exit_price) / entry_price) * 100.0 * leverage
    };
    let fee_drag = 2.0 * taker_fee_rate * leverage * 100.0;
    (gross - fee_drag, gross, fee_drag)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // TP/SL tests

    #[test]
    fn test_tp_sl_long_basic() {
        let (tp, sl) =
            compute_tp_sl_prices(50000.0, true, Some(3.0), Some(1.5), 0.0005, None, None).unwrap();
        let fee = 0.0005 * 2.0 * 100.0; // 0.1
        assert!((tp - 50000.0 * (1.0 + (3.0 + fee) / 100.0)).abs() < 1e-10);
        assert!((sl - 50000.0 * (1.0 - (1.5 - fee) / 100.0)).abs() < 1e-10);
    }

    #[test]
    fn test_tp_sl_short_basic() {
        let (tp, sl) =
            compute_tp_sl_prices(50000.0, false, Some(3.0), Some(1.5), 0.0005, None, None).unwrap();
        let fee = 0.0005 * 2.0 * 100.0;
        assert!((tp - 50000.0 * (1.0 - (3.0 + fee) / 100.0)).abs() < 1e-10);
        assert!((sl - 50000.0 * (1.0 + (1.5 - fee) / 100.0)).abs() < 1e-10);
    }

    #[test]
    fn test_tp_sl_overrides() {
        let (tp, sl) = compute_tp_sl_prices(
            50000.0,
            true,
            None,
            None,
            0.0005,
            Some(51500.0),
            Some(49000.0),
        )
        .unwrap();
        assert!((tp - 51500.0).abs() < 1e-10);
        assert!((sl - 49000.0).abs() < 1e-10);
    }

    #[test]
    fn test_tp_sl_no_fee() {
        let (tp, sl) =
            compute_tp_sl_prices(100.0, true, Some(3.0), Some(1.5), 0.0, None, None).unwrap();
        assert!((tp - 103.0).abs() < 1e-10);
        assert!((sl - 98.5).abs() < 1e-10);
    }

    #[test]
    fn test_tp_sl_fee_exceeds_sl_errs() {
        let err = compute_tp_sl_prices(50000.0, true, Some(3.0), Some(0.05), 0.0005, None, None)
            .unwrap_err();
        assert!(err.contains("must exceed round-trip fees"), "got: {err}");
    }

    #[test]
    fn test_tp_sl_fee_equals_sl_errs() {
        let err = compute_tp_sl_prices(50000.0, true, Some(3.0), Some(0.1), 0.0005, None, None)
            .unwrap_err();
        assert!(err.contains("must exceed round-trip fees"), "got: {err}");
    }

    // PnL tests

    #[test]
    fn test_pnl_long_profit() {
        let (net, gross, fee) = compute_pnl(50000.0, 51500.0, true, 1.0, 0.0005);
        assert!((gross - 3.0).abs() < 1e-10);
        assert!((fee - 0.1).abs() < 1e-10);
        assert!((net - 2.9).abs() < 1e-10);
    }

    #[test]
    fn test_pnl_short_profit() {
        let (net, gross, fee) = compute_pnl(50000.0, 49000.0, false, 1.0, 0.0005);
        assert!((gross - 2.0).abs() < 1e-10);
        assert!((fee - 0.1).abs() < 1e-10);
        assert!((net - 1.9).abs() < 1e-10);
    }

    #[test]
    fn test_pnl_leverage() {
        let (net, gross, fee) = compute_pnl(50000.0, 51500.0, true, 10.0, 0.0005);
        assert!((gross - 30.0).abs() < 1e-10);
        assert!((fee - 1.0).abs() < 1e-10);
        assert!((net - 29.0).abs() < 1e-10);
    }

    #[test]
    fn test_pnl_no_fee() {
        let (net, gross, fee) = compute_pnl(100.0, 103.0, true, 1.0, 0.0);
        assert!((gross - 3.0).abs() < 1e-10);
        assert!(fee.abs() < 1e-10);
        assert!((net - gross).abs() < 1e-10);
    }
}
