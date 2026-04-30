//! Category summary computation — Sortino, preference score, drawdown.

use claude_trader_models::{
    compute_equity_and_drawdown, CategorySummary, ExitReason, PortfolioConfig, TradeResult,
    WindowResult,
};

const PREFERENCE_DRAWDOWN_FLOOR_PCT: f64 = 5.0;
const PREFERENCE_TRADE_SCALE: f64 = 80.0;

/// Build a category summary from window results.
pub fn build_category_summary(
    category: &str,
    window_results: &[&WindowResult],
    config: &PortfolioConfig,
) -> CategorySummary {
    let total_weeks = window_results.len();

    // Weekly PnLs
    let week_pnls: Vec<f64> = window_results
        .iter()
        .map(|wr| wr.backtest.total_pnl_pct)
        .collect();

    let total_pnl: f64 = week_pnls.iter().sum();
    let positive_weeks = week_pnls.iter().filter(|&&p| p > 0.0).count();
    let active_weeks = window_results
        .iter()
        .filter(|wr| {
            wr.backtest
                .trades
                .iter()
                .any(|t| t.exit_reason != ExitReason::Unfilled)
        })
        .count();

    let weekly_win_rate = if total_weeks > 0 {
        positive_weeks as f64 / total_weeks as f64
    } else {
        0.0
    };

    let worst_week = week_pnls.iter().copied().fold(f64::INFINITY, f64::min);
    let best_week = week_pnls.iter().copied().fold(f64::NEG_INFINITY, f64::max);

    // Trade-level aggregation
    let all_trades: Vec<&TradeResult> = window_results
        .iter()
        .flat_map(|wr| wr.backtest.trades.iter())
        .collect();

    let resolved: Vec<&TradeResult> = all_trades
        .iter()
        .filter(|t| t.exit_reason != ExitReason::Unfilled)
        .copied()
        .collect();

    let total_trades = all_trades.len();
    let resolved_trades = resolved.len();
    let short_trades = all_trades
        .iter()
        .filter(|t| !t.signal.position_type.is_long())
        .count();
    let long_trades = total_trades - short_trades;

    let wins = resolved.iter().filter(|t| t.pnl_pct > 0.0).count();
    let trade_win_rate = if resolved_trades > 0 {
        wins as f64 / resolved_trades as f64
    } else {
        0.0
    };

    // Profit factor (single pass)
    let (gross_wins, gross_losses) = resolved.iter().fold((0.0f64, 0.0f64), |(w, l), t| {
        let weighted = t.pnl_pct * t.signal.size_multiplier;
        if t.pnl_pct > 0.0 {
            (w + weighted, l)
        } else {
            (w, l + (-weighted).max(0.0))
        }
    });

    let profit_factor = if gross_losses > 0.0 {
        gross_wins / gross_losses
    } else if gross_wins > 0.0 {
        f64::INFINITY
    } else {
        0.0
    };

    // Sort resolved trades by entry_time so drawdown sees chronological order
    // (flat_map across windows does not guarantee global time ordering).
    let mut resolved = resolved;
    resolved.sort_by_key(|t| t.entry_time);

    // Max drawdown from chronological trade stream
    let (_, max_drawdown_pct) = compute_equity_and_drawdown(&resolved);

    // Sortino
    let sortino_ratio = annualized_sortino(window_results, config.risk_free_rate_annual);

    // PnL to MDD
    let pnl_to_mdd = if max_drawdown_pct > 0.0 {
        total_pnl / max_drawdown_pct
    } else if total_pnl > 0.0 {
        f64::INFINITY
    } else {
        0.0
    };

    // Weekly omega (compute pos/neg once, reuse for preference score)
    let weekly_pos: f64 = week_pnls.iter().filter(|&&p| p > 0.0).sum();
    let weekly_neg: f64 = week_pnls.iter().filter(|&&p| p < 0.0).map(|&p| -p).sum();
    let omega = if weekly_neg > 0.0 {
        weekly_pos / weekly_neg
    } else if weekly_pos > 0.0 {
        f64::INFINITY
    } else {
        0.0
    };

    // Coverage penalty
    let coverage_penalty = if total_weeks > 0 && resolved_trades > 0 {
        (active_weeks as f64 / total_weeks as f64).sqrt()
            * (resolved_trades as f64 / PREFERENCE_TRADE_SCALE).min(1.0)
    } else {
        0.0
    };

    // Preference eligibility
    let min_trades = 40usize.min(10usize.max(2 * total_weeks));
    let min_active = 8usize.min(total_weeks);
    let preference_eligible = total_pnl > 0.0
        && profit_factor > 1.0
        && resolved_trades >= min_trades
        && active_weeks >= min_active;

    // Preference score — uses weekly PnL omega, not trade-level.
    // `dd_comp` uses the mean weekly PnL (total / total_weeks), not the
    // raw sum, so scores are length-invariant and dev (58 weeks) and
    // eval (41 weeks) are directly comparable.
    let preference_score = if coverage_penalty > 0.0 && total_pnl > 0.0 && total_weeks > 0 {
        let omega_comp = weekly_pos / weekly_neg.max(1.0);
        let mean_weekly_pnl = total_pnl / total_weeks as f64;
        let dd_comp = mean_weekly_pnl / max_drawdown_pct.max(PREFERENCE_DRAWDOWN_FLOOR_PCT);
        coverage_penalty * omega_comp * dd_comp
    } else {
        0.0
    };

    CategorySummary {
        category: category.to_string(),
        windows: total_weeks,
        total_pnl,
        weekly_win_rate,
        positive_weeks,
        worst_week_pnl: if worst_week.is_infinite() {
            0.0
        } else {
            worst_week
        },
        best_week_pnl: if best_week.is_infinite() {
            0.0
        } else {
            best_week
        },
        total_trades,
        resolved_trades,
        short_trades,
        long_trades,
        active_weeks,
        trade_win_rate,
        profit_factor,
        sortino_ratio,
        max_drawdown_pct,
        pnl_to_mdd,
        weekly_omega_ratio: omega,
        coverage_penalty,
        preference_eligible,
        preference_score,
    }
}

/// Build overall summary across all windows.
pub fn overall_summary(
    window_results: &[WindowResult],
    config: &PortfolioConfig,
) -> CategorySummary {
    let refs: Vec<&WindowResult> = window_results.iter().collect();
    build_category_summary("ALL", &refs, config)
}

/// Annualized Sortino ratio.
fn annualized_sortino(window_results: &[&WindowResult], risk_free_annual: f64) -> f64 {
    if window_results.is_empty() {
        return 0.0;
    }

    let mut excess_returns = Vec::new();
    let mut total_days = 0.0f64;

    for wr in window_results {
        let days = (wr.window.end - wr.window.start).num_milliseconds() as f64 / 86_400_000.0;
        if days <= 0.0 {
            continue;
        }
        let period_return = wr.backtest.total_pnl_pct / 100.0;
        let rf_adj = (1.0 + risk_free_annual).powf(days / 365.25) - 1.0;
        excess_returns.push(period_return - rf_adj);
        total_days += days;
    }

    if excess_returns.is_empty() {
        return 0.0;
    }

    let n = excess_returns.len() as f64;
    let mean_excess: f64 = excess_returns.iter().sum::<f64>() / n;

    let downside_sq: f64 = excess_returns
        .iter()
        .map(|&e| {
            let d = e.min(0.0);
            d * d
        })
        .sum::<f64>()
        / n;

    let downside_dev = downside_sq.sqrt();
    if !downside_dev.is_finite() || downside_dev == 0.0 {
        return if mean_excess > 0.0 {
            f64::INFINITY
        } else {
            0.0
        };
    }

    let avg_days = total_days / n;
    if avg_days <= 0.0 || !avg_days.is_finite() {
        return 0.0;
    }
    let annualization = (365.25 / avg_days).sqrt();

    let result = annualization * mean_excess / downside_dev;
    if result.is_finite() { result } else { 0.0 }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, TimeZone, Utc};
    use claude_trader_models::{
        BacktestResult, EvalWindow, MarketType, PositionType, ResolutionLevel, Signal,
    };
    use std::collections::HashMap;
    use std::sync::Arc;

    fn signal_at(ts: chrono::DateTime<Utc>) -> Signal {
        Signal {
            signal_date: ts,
            position_type: PositionType::Long,
            ticker: "BTCUSDT".to_string(),
            pattern: "test".to_string(),
            tp_pct: Some(3.0),
            sl_pct: Some(1.5),
            tp_price: None,
            sl_price: None,
            leverage: 1.0,
            market_type: MarketType::Futures,
            taker_fee_rate: 0.0005,
            entry_price: None,
            fill_timeout_seconds: 3600,
            entry_delay_seconds: None,
            max_holding_hours: 72,
            size_multiplier: 1.0,
            metadata: HashMap::new(),
        }
    }

    fn trade(ts: chrono::DateTime<Utc>, pnl_pct: f64) -> TradeResult {
        TradeResult {
            signal: Arc::new(signal_at(ts)),
            entry_price: 100.0,
            entry_time: ts,
            exit_price: 100.0,
            exit_time: ts + Duration::hours(1),
            exit_reason: if pnl_pct > 0.0 {
                ExitReason::Tp
            } else {
                ExitReason::Sl
            },
            resolution_level: ResolutionLevel::Hour,
            tp_price: 103.0,
            sl_price: 98.5,
            pnl_pct,
            gross_pnl_pct: pnl_pct,
            fee_drag_pct: 0.0,
            entry_fallback: false,
            exit_fallback: false,
            random_resolved: false,
        }
    }

    fn window_result(
        name: &str,
        start: chrono::DateTime<Utc>,
        total_pnl_pct: f64,
        trades: Vec<TradeResult>,
    ) -> WindowResult {
        WindowResult {
            window: EvalWindow {
                name: name.to_string(),
                category: "development".to_string(),
                start,
                end: start + Duration::days(7),
            },
            backtest: BacktestResult {
                trades,
                total_trades: 0,
                wins: 0,
                losses: 0,
                open_trades: 0,
                unfilled: 0,
                win_rate: 0.0,
                total_pnl_pct,
                avg_pnl_pct: 0.0,
                profit_factor: 0.0,
                max_drawdown_pct: 0.0,
                equity_curve: vec![100.0],
            },
            signal_count: 0,
            short_count: 0,
            long_count: 0,
        }
    }

    #[test]
    fn test_win_rate_scales_match_python_fractions() {
        let start = Utc.with_ymd_and_hms(2024, 4, 15, 0, 0, 0).unwrap();
        let w1 = window_result(
            "Apr24_W1",
            start,
            2.0,
            vec![trade(start, 2.0), trade(start + Duration::hours(1), -1.0)],
        );
        let w2 = window_result(
            "Apr24_W2",
            start + Duration::days(7),
            -1.0,
            vec![trade(start + Duration::days(7), -1.0)],
        );
        let config = PortfolioConfig::default();
        let refs = vec![&w1, &w2];

        let summary = build_category_summary("development", &refs, &config);

        assert!((summary.weekly_win_rate - 0.5).abs() < 1e-12);
        assert!((summary.trade_win_rate - (1.0 / 3.0)).abs() < 1e-12);
    }
}
