//! Parity tests for the resolver crate against Python golden fixtures.

use claude_trader_models::{
    ExitReason, MarketType, PositionType, ResolutionLevel, Signal, TradeResult, EPSILON_PCT,
    EPSILON_PRICE,
};
use claude_trader_resolver::stats::compute_stats;
use claude_trader_resolver::{compute_pnl, compute_tp_sl_prices};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("fixtures")
}

fn assert_approx_eq(actual: f64, expected: f64, epsilon: f64, context: &str) {
    if expected.is_nan() {
        assert!(actual.is_nan(), "{context}: expected NaN, got {actual}");
        return;
    }
    if expected.is_infinite() {
        assert!(
            actual.is_infinite() && actual.signum() == expected.signum(),
            "{context}: expected {expected}, got {actual}"
        );
        return;
    }
    let diff = (actual - expected).abs();
    assert!(
        diff <= epsilon,
        "{context}: expected {expected}, got {actual} (diff={diff}, epsilon={epsilon})"
    );
}

// ---------------------------------------------------------------------------
// TP/SL parity
// ---------------------------------------------------------------------------

#[test]
fn test_tp_sl_parity() {
    let path = fixtures_dir().join("resolver/tp_sl_prices.json");
    let content = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {e}", path.display()));
    let cases: Vec<serde_json::Value> = serde_json::from_str(&content).unwrap();

    for (i, case) in cases.iter().enumerate() {
        let entry_price = case["entry_price"].as_f64().unwrap();
        let is_long = case["is_long"].as_bool().unwrap();
        let tp_pct = case["tp_pct"].as_f64();
        let sl_pct = case["sl_pct"].as_f64();
        let fee_rate = case["taker_fee_rate"].as_f64().unwrap();
        let tp_override = case["tp_price_override"].as_f64();
        let sl_override = case["sl_price_override"].as_f64();
        let expected_tp = case["expected_tp_price"].as_f64().unwrap();
        let expected_sl = case["expected_sl_price"].as_f64().unwrap();

        let result = compute_tp_sl_prices(
            entry_price,
            is_long,
            tp_pct,
            sl_pct,
            fee_rate,
            tp_override,
            sl_override,
        );

        let fee_pct = fee_rate * 2.0 * 100.0;
        if sl_override.is_none() && sl_pct.is_some_and(|sl| sl <= fee_pct) {
            assert!(
                result.is_err(),
                "Case {i}: expected fee-vs-SL validation error (sl_pct={:?}, fee_pct={fee_pct})",
                sl_pct
            );
            continue;
        }

        // Cases with overrides that violate direction are now correctly rejected
        let prices_valid = if is_long {
            expected_tp > entry_price && expected_sl < entry_price
        } else {
            expected_tp < entry_price && expected_sl > entry_price
        };

        if !prices_valid {
            assert!(
                result.is_err(),
                "Case {i}: expected validation error for invalid TP/SL vs entry"
            );
            continue;
        }

        let (actual_tp, actual_sl) =
            result.unwrap_or_else(|e| panic!("Case {i}: compute failed: {e}"));

        assert_approx_eq(
            actual_tp,
            expected_tp,
            EPSILON_PRICE,
            &format!("Case {i} TP (entry={entry_price}, long={is_long})"),
        );
        assert_approx_eq(
            actual_sl,
            expected_sl,
            EPSILON_PRICE,
            &format!("Case {i} SL (entry={entry_price}, long={is_long})"),
        );
    }

    println!("TP/SL parity: {}/{} cases passed", cases.len(), cases.len());
}

// ---------------------------------------------------------------------------
// PnL parity
// ---------------------------------------------------------------------------

#[test]
fn test_pnl_parity() {
    let path = fixtures_dir().join("resolver/pnl_values.json");
    let content = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {e}", path.display()));
    let cases: Vec<serde_json::Value> = serde_json::from_str(&content).unwrap();

    for (i, case) in cases.iter().enumerate() {
        let entry = case["entry_price"].as_f64().unwrap();
        let exit = case["exit_price"].as_f64().unwrap();
        let is_long = case["is_long"].as_bool().unwrap();
        let leverage = case["leverage"].as_f64().unwrap();
        let fee_rate = case["taker_fee_rate"].as_f64().unwrap();
        let expected_net = case["expected_net_pnl_pct"].as_f64().unwrap();
        let expected_gross = case["expected_gross_pnl_pct"].as_f64().unwrap();
        let expected_fee = case["expected_fee_drag_pct"].as_f64().unwrap();

        let (actual_net, actual_gross, actual_fee) =
            compute_pnl(entry, exit, is_long, leverage, fee_rate);

        let ctx = format!("Case {i} (entry={entry}, exit={exit}, long={is_long}, lev={leverage})");
        assert_approx_eq(actual_net, expected_net, EPSILON_PCT, &format!("{ctx} net"));
        assert_approx_eq(
            actual_gross,
            expected_gross,
            EPSILON_PCT,
            &format!("{ctx} gross"),
        );
        assert_approx_eq(actual_fee, expected_fee, EPSILON_PCT, &format!("{ctx} fee"));
    }

    println!("PnL parity: {}/{} cases passed", cases.len(), cases.len());
}

// ---------------------------------------------------------------------------
// Stats computation parity
// ---------------------------------------------------------------------------

#[test]
fn test_stats_parity() {
    let path = fixtures_dir().join("engine/compute_stats.json");
    let content = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {e}", path.display()));
    let fixture: serde_json::Value = serde_json::from_str(&content).unwrap();

    let trade_data = fixture["trades"].as_array().unwrap();
    let expected = &fixture["expected_stats"];

    // Reconstruct TradeResult list from fixture
    let trades: Vec<TradeResult> = trade_data
        .iter()
        .map(|t| {
            let sig = &t["signal"];
            let pos_type = match sig["position_type"].as_str().unwrap() {
                "LONG" => PositionType::Long,
                "SHORT" => PositionType::Short,
                other => panic!("Unknown position type: {other}"),
            };
            let exit_reason = match t["exit_reason"].as_str().unwrap() {
                "TP" => ExitReason::Tp,
                "SL" => ExitReason::Sl,
                "TIMEOUT" => ExitReason::Timeout,
                "UNFILLED" => ExitReason::Unfilled,
                other => panic!("Unknown exit reason: {other}"),
            };
            let res_level = match t["resolution_level"].as_str().unwrap() {
                "1h" => ResolutionLevel::Hour,
                "1m" => ResolutionLevel::Minute,
                "trade" => ResolutionLevel::Trade,
                other => panic!("Unknown resolution level: {other}"),
            };

            let signal_date =
                chrono::DateTime::parse_from_rfc3339(sig["signal_date"].as_str().unwrap())
                    .unwrap()
                    .with_timezone(&chrono::Utc);

            let entry_time =
                chrono::DateTime::parse_from_rfc3339(t["entry_time"].as_str().unwrap())
                    .unwrap()
                    .with_timezone(&chrono::Utc);

            let exit_time = chrono::DateTime::parse_from_rfc3339(t["exit_time"].as_str().unwrap())
                .unwrap()
                .with_timezone(&chrono::Utc);

            TradeResult {
                signal: Arc::new(Signal {
                    signal_date,
                    position_type: pos_type,
                    ticker: sig["ticker"].as_str().unwrap().to_string(),
                    pattern: sig
                        .get("pattern")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    tp_pct: sig["tp_pct"].as_f64(),
                    sl_pct: sig["sl_pct"].as_f64(),
                    tp_price: None,
                    sl_price: None,
                    leverage: sig["leverage"].as_f64().unwrap(),
                    market_type: MarketType::Futures,
                    taker_fee_rate: 0.0005,
                    entry_price: None,
                    fill_timeout_seconds: 3600,
                    entry_delay_seconds: None,
                    max_holding_hours: 72,
                    size_multiplier: sig["size_multiplier"].as_f64().unwrap(),
                    metadata: HashMap::new(),
                }),
                entry_price: t["entry_price"].as_f64().unwrap(),
                entry_time,
                exit_price: t["exit_price"].as_f64().unwrap(),
                exit_time,
                exit_reason,
                resolution_level: res_level,
                tp_price: t["tp_price"].as_f64().unwrap(),
                sl_price: t["sl_price"].as_f64().unwrap(),
                pnl_pct: t["pnl_pct"].as_f64().unwrap(),
                gross_pnl_pct: t["gross_pnl_pct"].as_f64().unwrap(),
                fee_drag_pct: t["fee_drag_pct"].as_f64().unwrap(),
                entry_fallback: false,
                exit_fallback: false,
                random_resolved: false,
            }
        })
        .collect();

    let result = compute_stats(&trades);

    // Compare against expected
    assert_eq!(
        result.total_trades,
        expected["total_trades"].as_u64().unwrap() as usize
    );
    assert_eq!(result.wins, expected["wins"].as_u64().unwrap() as usize);
    assert_eq!(result.losses, expected["losses"].as_u64().unwrap() as usize);
    assert_eq!(
        result.open_trades,
        expected["open_trades"].as_u64().unwrap() as usize
    );
    assert_eq!(
        result.unfilled,
        expected["unfilled"].as_u64().unwrap() as usize
    );

    assert_approx_eq(
        result.win_rate,
        expected["win_rate"].as_f64().unwrap(),
        EPSILON_PCT,
        "win_rate",
    );
    assert_approx_eq(
        result.total_pnl_pct,
        expected["total_pnl_pct"].as_f64().unwrap(),
        EPSILON_PCT,
        "total_pnl_pct",
    );
    assert_approx_eq(
        result.avg_pnl_pct,
        expected["avg_pnl_pct"].as_f64().unwrap(),
        EPSILON_PCT,
        "avg_pnl_pct",
    );
    assert_approx_eq(
        result.profit_factor,
        expected["profit_factor"].as_f64().unwrap(),
        EPSILON_PCT,
        "profit_factor",
    );
    assert_approx_eq(
        result.max_drawdown_pct,
        expected["max_drawdown_pct"].as_f64().unwrap(),
        EPSILON_PCT,
        "max_drawdown_pct",
    );

    // Equity curve parity
    let expected_curve = expected["equity_curve"].as_array().unwrap();
    assert_eq!(
        result.equity_curve.len(),
        expected_curve.len(),
        "equity curve length mismatch"
    );
    for (i, (actual, exp)) in result
        .equity_curve
        .iter()
        .zip(expected_curve.iter())
        .enumerate()
    {
        assert_approx_eq(
            *actual,
            exp.as_f64().unwrap(),
            EPSILON_PCT,
            &format!("equity_curve[{i}]"),
        );
    }

    println!(
        "Stats parity: {} trades, win_rate={:.1}%, pnl={:.2}%, mdd={:.2}%",
        result.total_trades, result.win_rate, result.total_pnl_pct, result.max_drawdown_pct,
    );
}
