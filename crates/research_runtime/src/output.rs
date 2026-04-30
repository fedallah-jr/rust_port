//! Output serialization — trades.csv, meta.json, category_summary.csv, results.tsv.
//!
//! Matches Python's output layout:
//!   outputs/strategy_eval/<slug>_<window_label>_<mode>_<timestamp>/

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use chrono::Utc;
use claude_trader_models::ExitReason;

use crate::metrics::{compute_generalization_score, CV_MEAN_FLOOR_PP, GENERALIZATION_BUCKET_WINDOWS};
use crate::{CalibrationInterval, EvaluationResult};

pub const RESULTS_TSV_HEADER: &str =
    "strategy_name\tstrategy_description\tstrategy_score_dev\tstrategy_score_eval\tperformance_description";

/// Find the repo root by walking up from cwd looking for `program.md`
/// (the definitive repo-root marker) or `rust_port/`.
fn find_repo_root() -> PathBuf {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let mut dir = cwd.as_path();
    loop {
        // program.md lives at repo root
        if dir.join("program.md").is_file() {
            return dir.to_path_buf();
        }
        // If we're inside rust_port/, the parent is the repo root
        if dir.file_name().map(|n| n == "rust_port").unwrap_or(false)
            && dir.join("Cargo.toml").is_file()
        {
            return dir.parent().unwrap_or(dir).to_path_buf();
        }
        // rust_port/ is a child → we're at repo root
        if dir.join("rust_port").is_dir() {
            return dir.to_path_buf();
        }
        match dir.parent() {
            Some(p) => dir = p,
            None => return cwd,
        }
    }
}

/// Build the output directory path matching Python's format:
///   <repo_root>/outputs/strategy_eval/<slug>_<window>_<mode>_<timestamp>/
fn output_dir(result: &EvaluationResult) -> PathBuf {
    let root = find_repo_root();

    // Sanitize strategy name → slug (same as Python re.sub)
    let slug: String = result
        .strategy_name
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '.' || c == '_' || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim_matches(|c: char| c == '_' || c == '.' || c == '-')
        .to_string();
    let slug = if slug.is_empty() {
        "strategy".to_string()
    } else {
        slug
    };

    let mode = if result.approximate {
        "approx"
    } else {
        "exact"
    };
    let stamp = Utc::now().format("%Y%m%dT%H%M%SZ");

    root.join("outputs").join("strategy_eval").join(format!(
        "{slug}_{window}_{mode}_{stamp}",
        window = result.window_set_label
    ))
}

/// Save all output files for an evaluation run.
pub fn save_outputs(result: &EvaluationResult) -> Result<PathBuf, String> {
    let dir = output_dir(result);
    fs::create_dir_all(&dir).map_err(|e| format!("Failed to create output dir: {e}"))?;

    write_trades_csv(&dir, result)?;
    write_meta_json(&dir, result)?;
    write_category_summary_csv(&dir, result)?;
    write_per_pattern_csv(&dir, result)?;
    write_per_symbol_csv(&dir, result)?;
    write_per_regime_vs_pattern_csv(&dir, result)?;
    if !result.calibration_intervals.is_empty() {
        write_calibration_log(&dir, &result.calibration_intervals)?;
    }

    // Append to results.tsv in the experiment crate directory
    if let Some(tsv_path) = find_experiment_results_tsv() {
        if let Err(e) = append_results_tsv(&tsv_path, result) {
            eprintln!("WARNING: Failed to append results.tsv: {e}");
        }
    }

    println!("\nOutputs saved to: {}", dir.display());
    Ok(dir)
}

/// Find results.tsv by looking for it relative to the binary's Cargo.toml.
///
/// When running `cargo run -p ct-research-foo`, the binary is in the
/// experiment crate whose results.tsv lives next to Cargo.toml.
fn find_experiment_results_tsv() -> Option<PathBuf> {
    // Try CARGO_MANIFEST_DIR (set during `cargo run`)
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let path = PathBuf::from(manifest_dir).join("results.tsv");
        return Some(path);
    }

    // Fallback: look for results.tsv relative to the binary location
    if let Ok(exe) = std::env::current_exe() {
        // Binary might be in target/release/ — walk up looking for research/
        let mut dir = exe.parent();
        while let Some(d) = dir {
            if d.join("results.tsv").is_file() {
                return Some(d.join("results.tsv"));
            }
            dir = d.parent();
        }
    }

    None
}

fn write_trades_csv(dir: &Path, result: &EvaluationResult) -> Result<(), String> {
    let path = dir.join("trades.csv");
    let mut f = fs::File::create(&path).map_err(|e| format!("trades.csv: {e}"))?;

    writeln!(
        f,
        "signal_date,ticker,position_type,pattern,entry_price,entry_time,exit_price,exit_time,exit_reason,pnl_pct,gross_pnl_pct,fee_drag_pct,tp_price,sl_price,leverage,max_holding_hours"
    )
    .map_err(|e| format!("trades.csv write: {e}"))?;

    for t in result
        .report
        .window_results
        .iter()
        .flat_map(|wr| &wr.backtest.trades)
    {
        let pos = if t.signal.position_type.is_long() {
            "Long"
        } else {
            "Short"
        };
        writeln!(
            f,
            "{},{},{},{},{},{},{},{},{:?},{:.6},{:.6},{:.6},{},{},{},{}",
            t.signal.signal_date.format("%Y-%m-%d %H:%M:%S"),
            t.signal.ticker,
            pos,
            t.signal.pattern,
            t.entry_price,
            t.entry_time.format("%Y-%m-%d %H:%M:%S"),
            t.exit_price,
            t.exit_time.format("%Y-%m-%d %H:%M:%S"),
            t.exit_reason,
            t.pnl_pct,
            t.gross_pnl_pct,
            t.fee_drag_pct,
            t.tp_price,
            t.sl_price,
            t.signal.leverage,
            t.signal.max_holding_hours,
        )
        .map_err(|e| format!("trades.csv write: {e}"))?;
    }

    Ok(())
}

fn write_meta_json(dir: &Path, result: &EvaluationResult) -> Result<(), String> {
    let path = dir.join("meta.json");

    let all_trades = || {
        result
            .report
            .window_results
            .iter()
            .flat_map(|wr| &wr.backtest.trades)
    };
    let total_trades = all_trades().count();
    let resolved = all_trades()
        .filter(|t| t.exit_reason != ExitReason::Unfilled)
        .count();
    let wins = all_trades().filter(|t| t.pnl_pct > 0.0).count();

    let generalization = compute_generalization_score(&result.report.window_results);

    let meta = serde_json::json!({
        "strategy": result.strategy_name,
        "window_set": result.window_set_label,
        "approximate": result.approximate,
        "timestamp": Utc::now().to_rfc3339(),
        "runtime": "rust",
        "symbols": result.symbols,
        "total_trades": total_trades,
        "resolved_trades": resolved,
        "wins": wins,
        "overall": {
            "pnl": result.overall.total_pnl,
            "win_rate": result.overall.weekly_win_rate,
            "profit_factor": result.overall.profit_factor,
            "sortino": result.overall.sortino_ratio,
            "max_drawdown": result.overall.max_drawdown_pct,
            "omega": result.overall.weekly_omega_ratio,
            "preference_score": result.overall.preference_score,
            "preference_eligible": result.overall.preference_eligible,
        },
        "generalization": {
            "score": generalization.score,
            "cv": generalization.cv,
            "bucket_count": generalization.bucket_count,
            "mean_bucket_pnl": generalization.mean_bucket_pnl,
            "std_bucket_pnl": generalization.std_bucket_pnl,
            "bucket_windows": GENERALIZATION_BUCKET_WINDOWS,
            "mean_floor_pp": CV_MEAN_FLOOR_PP,
        },
    });

    let json = serde_json::to_string_pretty(&meta).map_err(|e| format!("meta.json: {e}"))?;
    fs::write(&path, json).map_err(|e| format!("meta.json: {e}"))?;
    Ok(())
}

fn write_category_summary_csv(dir: &Path, result: &EvaluationResult) -> Result<(), String> {
    let path = dir.join("category_summary.csv");
    let mut f = fs::File::create(&path).map_err(|e| format!("category_summary.csv: {e}"))?;

    writeln!(
        f,
        "category,windows,total_pnl,weekly_win_rate,positive_weeks,worst_week_pnl,best_week_pnl,total_trades,short_trades,long_trades,trade_win_rate,profit_factor,sortino_ratio,max_drawdown_pct,omega_ratio,preference_score,preference_eligible"
    )
    .map_err(|e| format!("category_summary.csv write: {e}"))?;

    let all: Vec<&claude_trader_models::CategorySummary> = result
        .summaries
        .iter()
        .chain(std::iter::once(&result.overall))
        .collect();

    for cs in all {
        writeln!(
            f,
            "{},{},{:.4},{:.4},{},{:.4},{:.4},{},{},{},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{}",
            cs.category,
            cs.windows,
            cs.total_pnl,
            cs.weekly_win_rate,
            cs.positive_weeks,
            cs.worst_week_pnl,
            cs.best_week_pnl,
            cs.total_trades,
            cs.short_trades,
            cs.long_trades,
            cs.trade_win_rate,
            cs.profit_factor,
            cs.sortino_ratio,
            cs.max_drawdown_pct,
            cs.weekly_omega_ratio,
            cs.preference_score,
            cs.preference_eligible,
        )
        .map_err(|e| format!("category_summary.csv write: {e}"))?;
    }

    Ok(())
}

/// Aggregate stats for a group of trades — shared helper for the three
/// diagnostic CSVs. Only resolved trades (not `Unfilled`) are counted.
#[derive(Default, Clone)]
struct AggStats {
    total: usize,
    tp: usize,
    sl: usize,
    timeout: usize,
    manual: usize,
    longs: usize,
    shorts: usize,
    total_pnl: f64,
    gross_win: f64,
    gross_loss_abs: f64,
    win_pnl_sum: f64,
    loss_pnl_sum: f64,
    win_count: usize,
    loss_count: usize,
}

impl AggStats {
    fn add(&mut self, t: &claude_trader_models::TradeResult) {
        if t.exit_reason == ExitReason::Unfilled {
            return;
        }
        self.total += 1;
        match t.exit_reason {
            ExitReason::Tp => self.tp += 1,
            ExitReason::Sl => self.sl += 1,
            ExitReason::Timeout => self.timeout += 1,
            _ => self.manual += 1,
        }
        if t.signal.position_type.is_long() {
            self.longs += 1;
        } else {
            self.shorts += 1;
        }
        self.total_pnl += t.pnl_pct;
        if t.pnl_pct > 0.0 {
            self.win_count += 1;
            self.win_pnl_sum += t.pnl_pct;
            self.gross_win += t.pnl_pct;
        } else if t.pnl_pct < 0.0 {
            self.loss_count += 1;
            self.loss_pnl_sum += t.pnl_pct;
            self.gross_loss_abs += -t.pnl_pct;
        }
    }

    fn trade_win_rate(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            100.0 * self.win_count as f64 / self.total as f64
        }
    }
    fn avg_pnl(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            self.total_pnl / self.total as f64
        }
    }
    fn avg_win(&self) -> f64 {
        if self.win_count == 0 {
            0.0
        } else {
            self.win_pnl_sum / self.win_count as f64
        }
    }
    fn avg_loss(&self) -> f64 {
        if self.loss_count == 0 {
            0.0
        } else {
            self.loss_pnl_sum / self.loss_count as f64
        }
    }
    fn profit_factor(&self) -> f64 {
        if self.gross_loss_abs <= 0.0 {
            if self.gross_win > 0.0 { f64::INFINITY } else { 0.0 }
        } else {
            self.gross_win / self.gross_loss_abs
        }
    }
}

fn format_f64(v: f64) -> String {
    if v.is_infinite() {
        if v.is_sign_positive() { "inf".to_string() } else { "-inf".to_string() }
    } else if v.is_nan() {
        "nan".to_string()
    } else {
        format!("{v:.4}")
    }
}

/// `per_pattern.csv`: contribution of each strategy pattern. Use this to
/// see which sub-signal is carrying the strategy and which is just adding
/// noise. Writes one row per distinct `signal.pattern` string.
fn write_per_pattern_csv(dir: &Path, result: &EvaluationResult) -> Result<(), String> {
    let path = dir.join("per_pattern.csv");
    let mut f = fs::File::create(&path).map_err(|e| format!("per_pattern.csv: {e}"))?;
    writeln!(
        f,
        "pattern,trades,wins_tp,losses_sl,timeouts,other_exits,longs,shorts,trade_win_rate_pct,total_pnl_pct,avg_pnl_pct,avg_win_pct,avg_loss_pct,profit_factor,gross_win_pct,gross_loss_pct"
    )
    .map_err(|e| format!("per_pattern.csv write: {e}"))?;

    let mut agg: std::collections::BTreeMap<String, AggStats> = std::collections::BTreeMap::new();
    for t in result.report.window_results.iter().flat_map(|wr| &wr.backtest.trades) {
        let key = if t.signal.pattern.is_empty() {
            "<unnamed>".to_string()
        } else {
            t.signal.pattern.clone()
        };
        agg.entry(key).or_default().add(t);
    }

    for (pattern, s) in agg {
        writeln!(
            f,
            "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
            pattern,
            s.total,
            s.tp,
            s.sl,
            s.timeout,
            s.manual,
            s.longs,
            s.shorts,
            format_f64(s.trade_win_rate()),
            format_f64(s.total_pnl),
            format_f64(s.avg_pnl()),
            format_f64(s.avg_win()),
            format_f64(s.avg_loss()),
            format_f64(s.profit_factor()),
            format_f64(s.gross_win),
            format_f64(s.gross_loss_abs),
        )
        .map_err(|e| format!("per_pattern.csv write: {e}"))?;
    }
    Ok(())
}

/// `per_symbol.csv`: contribution of each ticker. Use this to spot symbols
/// that are either carrying the strategy or dragging it down, without
/// cherry-picking (symbol selection on dev contributes to eval overfitting
/// — prefer understanding the distribution to filtering by it).
fn write_per_symbol_csv(dir: &Path, result: &EvaluationResult) -> Result<(), String> {
    let path = dir.join("per_symbol.csv");
    let mut f = fs::File::create(&path).map_err(|e| format!("per_symbol.csv: {e}"))?;
    writeln!(
        f,
        "symbol,trades,wins_tp,losses_sl,timeouts,other_exits,longs,shorts,trade_win_rate_pct,total_pnl_pct,avg_pnl_pct,profit_factor"
    )
    .map_err(|e| format!("per_symbol.csv write: {e}"))?;

    let mut agg: std::collections::BTreeMap<String, AggStats> = std::collections::BTreeMap::new();
    for t in result.report.window_results.iter().flat_map(|wr| &wr.backtest.trades) {
        agg.entry(t.signal.ticker.clone()).or_default().add(t);
    }

    for (symbol, s) in agg {
        writeln!(
            f,
            "{},{},{},{},{},{},{},{},{},{},{},{}",
            symbol,
            s.total,
            s.tp,
            s.sl,
            s.timeout,
            s.manual,
            s.longs,
            s.shorts,
            format_f64(s.trade_win_rate()),
            format_f64(s.total_pnl),
            format_f64(s.avg_pnl()),
            format_f64(s.profit_factor()),
        )
        .map_err(|e| format!("per_symbol.csv write: {e}"))?;
    }
    Ok(())
}

/// `per_regime_vs_pattern.csv`: long-form cross-tab of pattern × window
/// category. Use this to see which sub-strategy earns its keep in which
/// regime — e.g. whether a breakout signal works in sideways, or whether
/// a divergence short only works in bearish regimes.
fn write_per_regime_vs_pattern_csv(dir: &Path, result: &EvaluationResult) -> Result<(), String> {
    let path = dir.join("per_regime_vs_pattern.csv");
    let mut f = fs::File::create(&path).map_err(|e| format!("per_regime_vs_pattern.csv: {e}"))?;
    writeln!(
        f,
        "category,pattern,trades,wins_tp,losses_sl,timeouts,other_exits,longs,shorts,trade_win_rate_pct,total_pnl_pct,avg_pnl_pct,profit_factor"
    )
    .map_err(|e| format!("per_regime_vs_pattern.csv write: {e}"))?;

    type Key = (String, String);
    let mut agg: std::collections::BTreeMap<Key, AggStats> = std::collections::BTreeMap::new();
    for wr in &result.report.window_results {
        let category = wr.window.category.clone();
        for t in &wr.backtest.trades {
            let pattern = if t.signal.pattern.is_empty() {
                "<unnamed>".to_string()
            } else {
                t.signal.pattern.clone()
            };
            agg.entry((category.clone(), pattern)).or_default().add(t);
        }
    }

    for ((category, pattern), s) in agg {
        writeln!(
            f,
            "{},{},{},{},{},{},{},{},{},{},{},{},{}",
            category,
            pattern,
            s.total,
            s.tp,
            s.sl,
            s.timeout,
            s.manual,
            s.longs,
            s.shorts,
            format_f64(s.trade_win_rate()),
            format_f64(s.total_pnl),
            format_f64(s.avg_pnl()),
            format_f64(s.profit_factor()),
        )
        .map_err(|e| format!("per_regime_vs_pattern.csv write: {e}"))?;
    }
    Ok(())
}

fn write_calibration_log(dir: &Path, intervals: &[CalibrationInterval]) -> Result<(), String> {
    let path = dir.join("calibration_log.txt");
    let mut f = fs::File::create(&path).map_err(|e| format!("calibration_log.txt: {e}"))?;

    writeln!(f, "Calibration Log — {} intervals", intervals.len())
        .map_err(|e| format!("calibration_log.txt write: {e}"))?;
    writeln!(f, "{}", "=".repeat(80)).map_err(|e| format!("calibration_log.txt write: {e}"))?;

    for (idx, interval) in intervals.iter().enumerate() {
        let params_str: Vec<String> = {
            let mut keys: Vec<&String> = interval.params.keys().collect();
            keys.sort();
            keys.iter()
                .map(|k| format!("{}={}", k, interval.params[*k]))
                .collect()
        };
        writeln!(
            f,
            "[{:>3}] {} -> {}  {}",
            idx + 1,
            interval.start.format("%Y-%m-%d %H:%M"),
            interval.end.format("%Y-%m-%d %H:%M"),
            params_str.join("  "),
        )
        .map_err(|e| format!("calibration_log.txt write: {e}"))?;
    }

    Ok(())
}

/// Upsert a row in results.tsv using the program.md 5-column schema:
///   strategy_name \t strategy_description \t strategy_score_dev \t strategy_score_eval \t performance_description
///
/// One row per `strategy_name`. A dev run fills `strategy_score_dev`; an eval
/// run fills `strategy_score_eval`; re-running either updates only its own
/// score column. `strategy_description` is always refreshed from
/// `strategy.description()` (the strategy owns it). `performance_description`
/// is written once when the row is first created (auto-generated metrics as
/// a starting point) and then left alone on subsequent runs so manual edits
/// survive.
pub fn append_results_tsv(tsv_path: &Path, result: &EvaluationResult) -> Result<(), String> {
    let score_str = format!("{:.2}", result.overall.preference_score);
    let (new_dev, new_eval) = match result.window_set_label.as_str() {
        "development" => (Some(score_str), None),
        "evaluation" => (None, Some(score_str)),
        other => (Some(format!("{} ({})", score_str, other)), None),
    };

    let generalization = compute_generalization_score(&result.report.window_results);
    let initial_perf_desc = format!(
        "PNL {:+.2}% | PF {:.2} | Sort {:.2} | MDD {:.2}% | Gen {:.3} | {} trades | {}",
        result.overall.total_pnl,
        result.overall.profit_factor,
        result.overall.sortino_ratio,
        result.overall.max_drawdown_pct,
        generalization.score,
        result.overall.total_trades,
        if result.overall.preference_eligible {
            "eligible"
        } else {
            "not eligible"
        },
    );

    let existing = if tsv_path.exists() {
        fs::read_to_string(tsv_path).map_err(|e| format!("results.tsv read: {e}"))?
    } else {
        String::new()
    };

    let mut rows: Vec<Vec<String>> = Vec::new();
    let mut has_header = false;
    for line in existing.lines() {
        if line.is_empty() {
            continue;
        }
        let cols: Vec<String> = line.split('\t').map(|s| s.to_string()).collect();
        if !has_header && cols.first().map(|s| s.as_str()) == Some("strategy_name") {
            has_header = true;
            rows.push(cols);
            continue;
        }
        rows.push(cols);
    }

    let pad = |cols: &mut Vec<String>| {
        while cols.len() < 5 {
            cols.push("-".to_string());
        }
    };

    let target_row_idx = rows.iter().position(|cols| {
        cols.first().map(|s| s.as_str()) == Some(result.strategy_name.as_str())
    });

    match target_row_idx {
        Some(idx) => {
            let cols = &mut rows[idx];
            pad(cols);
            cols[1] = result.strategy_description.clone();
            if let Some(dev) = new_dev {
                cols[2] = dev;
            }
            if let Some(eval) = new_eval {
                cols[3] = eval;
            }
            // cols[4] (performance_description) is preserved — agent-owned.
        }
        None => {
            let new_row = vec![
                result.strategy_name.clone(),
                result.strategy_description.clone(),
                new_dev.unwrap_or_else(|| "-".to_string()),
                new_eval.unwrap_or_else(|| "-".to_string()),
                initial_perf_desc,
            ];
            rows.push(new_row);
        }
    }

    let mut out = String::new();
    if !has_header {
        out.push_str(RESULTS_TSV_HEADER);
        out.push('\n');
    }
    for cols in &rows {
        out.push_str(&cols.join("\t"));
        out.push('\n');
    }

    fs::write(tsv_path, out).map_err(|e| format!("results.tsv write: {e}"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    use claude_trader_models::{CategorySummary, EvaluationReport, PortfolioConfig};

    #[test]
    fn append_results_tsv_writes_canonical_header() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "ct_results_tsv_{}_{}.tsv",
            std::process::id(),
            unique
        ));
        let _ = std::fs::remove_file(&path);

        let result = EvaluationResult {
            strategy_name: "test_strategy".to_string(),
            strategy_description: "test_strategy hypothesis".to_string(),
            window_set_label: "development".to_string(),
            approximate: false,
            summaries: Vec::new(),
            overall: CategorySummary {
                total_pnl: 12.34,
                profit_factor: 1.23,
                sortino_ratio: 0.98,
                max_drawdown_pct: 4.56,
                total_trades: 7,
                preference_eligible: true,
                preference_score: 1.11,
                ..Default::default()
            },
            report: EvaluationReport {
                window_results: Vec::new(),
                config: PortfolioConfig::default(),
                symbols: Vec::new(),
            },
            symbols: Vec::new(),
            calibration_intervals: Vec::new(),
        };

        append_results_tsv(&path, &result).unwrap();

        let contents = std::fs::read_to_string(&path).unwrap();
        let lines: Vec<&str> = contents.lines().collect();
        assert_eq!(lines[0], RESULTS_TSV_HEADER);
        assert!(
            lines[1].starts_with("test_strategy\t"),
            "first data row should be the upserted strategy, got: {}",
            lines[1]
        );

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn append_results_tsv_upserts_one_row_per_strategy() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "ct_results_tsv_upsert_{}_{}.tsv",
            std::process::id(),
            unique
        ));
        let _ = std::fs::remove_file(&path);

        let base_overall = CategorySummary {
            total_pnl: 20.0,
            profit_factor: 1.5,
            sortino_ratio: 2.0,
            max_drawdown_pct: 10.0,
            total_trades: 100,
            preference_eligible: true,
            preference_score: 3.0,
            ..Default::default()
        };
        let mk = |label: &str, score: f64| EvaluationResult {
            strategy_name: "strat_x".to_string(),
            strategy_description: "desc from trait".to_string(),
            window_set_label: label.to_string(),
            approximate: false,
            summaries: Vec::new(),
            overall: CategorySummary {
                preference_score: score,
                ..base_overall.clone()
            },
            report: EvaluationReport {
                window_results: Vec::new(),
                config: PortfolioConfig::default(),
                symbols: Vec::new(),
            },
            symbols: Vec::new(),
            calibration_intervals: Vec::new(),
        };

        // First: dev run — creates row with dev score filled, eval '-'.
        append_results_tsv(&path, &mk("development", 4.0)).unwrap();

        // Simulate the agent manually editing performance_description.
        let edited = std::fs::read_to_string(&path)
            .unwrap()
            .replace(
                "PNL +20.00% | PF 1.50 | Sort 2.00 | MDD 10.00% | Gen 0.000 | 100 trades | eligible",
                "agent-written explanation",
            );
        std::fs::write(&path, edited).unwrap();

        // Re-run dev with a different score — row is updated, not duplicated.
        append_results_tsv(&path, &mk("development", 5.5)).unwrap();

        // Eval run — fills eval column on the SAME row.
        append_results_tsv(&path, &mk("evaluation", 2.25)).unwrap();

        let contents = std::fs::read_to_string(&path).unwrap();
        let lines: Vec<&str> = contents.lines().collect();
        assert_eq!(lines[0], RESULTS_TSV_HEADER);

        let strat_rows: Vec<&&str> =
            lines.iter().filter(|l| l.starts_with("strat_x\t")).collect();
        assert_eq!(strat_rows.len(), 1, "expected one row per strategy");

        let cols: Vec<&str> = strat_rows[0].split('\t').collect();
        assert_eq!(cols[0], "strat_x");
        assert_eq!(cols[1], "desc from trait");
        assert_eq!(cols[2], "5.50");
        assert_eq!(cols[3], "2.25");
        assert_eq!(
            cols[4], "agent-written explanation",
            "performance_description must be preserved across re-runs"
        );

        let _ = std::fs::remove_file(path);
    }
}
