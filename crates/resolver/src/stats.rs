//! Stats aggregation — computes BacktestResult from a list of TradeResults.
//!
//! Mirrors Python `backtester/engine.py::_compute_stats()`.

use claude_trader_models::{compute_equity_and_drawdown, BacktestResult, ExitReason, TradeResult};

/// Compute aggregated statistics from a list of trade results.
///
/// Mirrors Python `_compute_stats()` exactly:
/// - Resolved trades exclude UNFILLED
/// - Equity curve starts at 100.0, uses multiplicative compounding
/// - Max drawdown is computed from the equity curve
/// - Profit factor = gross_wins / gross_losses
pub fn compute_stats(trades: &[TradeResult]) -> BacktestResult {
    let total_all = trades.len();

    // Partition
    let unfilled = trades
        .iter()
        .filter(|t| t.exit_reason == ExitReason::Unfilled)
        .count();
    let resolved: Vec<&TradeResult> = trades
        .iter()
        .filter(|t| t.exit_reason != ExitReason::Unfilled)
        .collect();

    let total = resolved.len();
    let open_trades = resolved
        .iter()
        .filter(|t| t.exit_reason == ExitReason::Timeout)
        .count();

    let mut wins = 0usize;
    let mut losses = 0usize;
    let mut total_pnl = 0.0f64;
    let mut gross_wins = 0.0f64;
    let mut gross_losses = 0.0f64;

    for t in &resolved {
        let pnl_weighted = t.pnl_pct * t.signal.size_multiplier;
        total_pnl += pnl_weighted;

        if t.pnl_pct > 0.0 {
            wins += 1;
            gross_wins += pnl_weighted;
        } else {
            losses += 1;
            gross_losses += (-pnl_weighted).max(0.0);
        }
    }

    let (equity_curve, max_dd) = compute_equity_and_drawdown(&resolved);

    let win_rate = if total > 0 {
        (wins as f64 / total as f64) * 100.0
    } else {
        0.0
    };

    let avg_pnl = if total > 0 {
        total_pnl / total as f64
    } else {
        0.0
    };

    let profit_factor = if gross_losses > 0.0 {
        gross_wins / gross_losses
    } else if gross_wins > 0.0 {
        f64::INFINITY
    } else {
        0.0
    };

    BacktestResult {
        trades: trades.to_vec(),
        total_trades: total_all,
        wins,
        losses,
        open_trades,
        unfilled,
        win_rate,
        total_pnl_pct: total_pnl,
        avg_pnl_pct: avg_pnl,
        profit_factor,
        max_drawdown_pct: max_dd,
        equity_curve,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use claude_trader_models::{PositionType, ResolutionLevel, Signal};
    use std::collections::HashMap;
    use std::sync::Arc;

    fn make_trade(pnl: f64, exit_reason: ExitReason, size_mult: f64) -> TradeResult {
        let now = Utc::now();
        TradeResult {
            signal: Arc::new(Signal {
                signal_date: now,
                position_type: PositionType::Long,
                ticker: "BTCUSDT".to_string(),
                pattern: "test".to_string(),
                tp_pct: Some(3.0),
                sl_pct: Some(1.5),
                tp_price: None,
                sl_price: None,
                leverage: 1.0,
                market_type: claude_trader_models::MarketType::Futures,
                taker_fee_rate: 0.0005,
                entry_price: None,
                fill_timeout_seconds: 3600,
                entry_delay_seconds: None,
                max_holding_hours: 72,
                size_multiplier: size_mult,
                metadata: HashMap::new(),
            }),
            entry_price: 50000.0,
            entry_time: now,
            exit_price: 50000.0 * (1.0 + pnl / 100.0),
            exit_time: now,
            exit_reason,
            resolution_level: ResolutionLevel::Hour,
            tp_price: 51500.0,
            sl_price: 49250.0,
            pnl_pct: pnl,
            gross_pnl_pct: pnl + 0.1,
            fee_drag_pct: 0.1,
            entry_fallback: false,
            exit_fallback: false,
            random_resolved: false,
        }
    }

    #[test]
    fn test_empty_trades() {
        let result = compute_stats(&[]);
        assert_eq!(result.total_trades, 0);
        assert_eq!(result.wins, 0);
        assert_eq!(result.losses, 0);
        assert!((result.win_rate - 0.0).abs() < 1e-10);
        assert_eq!(result.equity_curve.len(), 1);
        assert!((result.equity_curve[0] - 100.0).abs() < 1e-10);
    }

    #[test]
    fn test_all_wins() {
        let trades = vec![
            make_trade(2.0, ExitReason::Tp, 1.0),
            make_trade(3.0, ExitReason::Tp, 1.0),
        ];
        let result = compute_stats(&trades);
        assert_eq!(result.wins, 2);
        assert_eq!(result.losses, 0);
        assert!((result.win_rate - 100.0).abs() < 1e-10);
        assert!(result.profit_factor.is_infinite());
        assert!((result.max_drawdown_pct - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_unfilled_excluded() {
        let trades = vec![
            make_trade(2.0, ExitReason::Tp, 1.0),
            make_trade(0.0, ExitReason::Unfilled, 1.0),
        ];
        let result = compute_stats(&trades);
        assert_eq!(result.total_trades, 2);
        assert_eq!(result.unfilled, 1);
        assert_eq!(result.wins, 1);
        assert_eq!(result.losses, 0);
    }

    #[test]
    fn test_size_multiplier() {
        let trades = vec![make_trade(2.0, ExitReason::Tp, 2.0)];
        let result = compute_stats(&trades);
        assert!((result.total_pnl_pct - 4.0).abs() < 1e-10);
    }
}
