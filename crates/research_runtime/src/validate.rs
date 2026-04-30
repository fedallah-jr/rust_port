//! No-lookahead validation.
//!
//! Reference signals are generated using the same pipeline as evaluation
//! (calibration, BTC structure, funding, key levels) via
//! `generate_all_signals`.
//!
//! For each emitted signal, the replay truncates all inputs to data
//! available at signal time and re-runs `generate_signals()` with the
//! correct calibration params, context, and HTF data that were active at
//! that time during the reference run. If the signal still appears in the
//! truncated replay, it passes. If not, it's a lookahead violation.

use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::time::Instant;

use chrono::{DateTime, Duration, Utc};
use claude_trader_data::{BinanceClient, CandleStore};
use claude_trader_evaluator::windows::group_into_periods;
use claude_trader_models::{Candle, ContextMap, HtfData, MarketType, Signal};

use crate::{
    ensure_additional_interval_candles, ensure_candles, generate_all_signals,
    validate_strategy_interval, CalibrationInterval, ResearchStrategy, RunConfig,
};

use crate::{COOLDOWN_WARMUP_DAYS, DATA_BUFFER_HOURS};

fn rebuild_truncated_candles(
    store: &mut CandleStore,
    analysis_interval: &str,
    api_symbols: &[String],
    fetch_start: DateTime<Utc>,
    signal_date: DateTime<Utc>,
    interval_duration: Duration,
    truncated: &mut BTreeMap<String, Vec<Candle>>,
    vec_pool: &mut Vec<Vec<Candle>>,
) {
    let fetch_start_ms = fetch_start.timestamp_millis();

    for (_, v) in std::mem::take(truncated) {
        vec_pool.push(v);
    }

    for sym in api_symbols {
        if analysis_interval == "1m" {
            let candles = store.get_range(
                sym,
                analysis_interval,
                fetch_start,
                signal_date + interval_duration,
            );
            let idx = candles.partition_point(|c| c.close_time <= signal_date);
            if idx > 0 {
                let mut v = candles;
                v.truncate(idx);
                truncated.insert(sym.clone(), v);
            }
        } else {
            let full = store.get_all_ref(sym, analysis_interval);
            let lo = full.partition_point(|c| c.close_time.timestamp_millis() < fetch_start_ms);
            let hi = full.partition_point(|c| c.close_time <= signal_date);
            if lo < hi {
                let mut v = vec_pool.pop().unwrap_or_default();
                v.clear();
                v.extend_from_slice(&full[lo..hi]);
                truncated.insert(sym.clone(), v);
            }
        }
    }
}

fn active_params_for_signal(
    intervals: &[CalibrationInterval],
    params_cursor: &mut usize,
    signal_date: DateTime<Utc>,
) -> HashMap<String, serde_json::Value> {
    while *params_cursor < intervals.len() && intervals[*params_cursor].end <= signal_date {
        *params_cursor += 1;
    }
    if *params_cursor < intervals.len()
        && signal_date >= intervals[*params_cursor].start
        && signal_date < intervals[*params_cursor].end
    {
        intervals[*params_cursor].params.clone()
    } else {
        HashMap::new()
    }
}

/// Run no-lookahead validation. Returns exit code (0 = pass, 1 = fail).
pub fn run_validation(strategy: &dyn ResearchStrategy, config: &RunConfig) -> i32 {
    let t_total = Instant::now();

    let windows = config.window_set.windows();
    let api_symbols = strategy.symbols();
    let indicator_cols = strategy.indicator_columns();
    let mut warmup_bars =
        claude_trader_indicators::required_warmup(indicator_cols) + strategy.extra_warmup_bars();

    let calib_config = strategy.calibration_config();

    let (analysis_interval, interval_duration) = validate_strategy_interval(strategy);
    let interval_secs = interval_duration.num_seconds();

    if let Some(ref cc) = calib_config {
        let calib_bars =
            ((cc.lookback_hours as f64) * 3600.0 / interval_secs as f64).ceil() as usize;
        if calib_bars > warmup_bars {
            warmup_bars = calib_bars;
        }
    }

    println!("{} — No-Lookahead Validation", strategy.name());
    println!("Windows: {} {}", windows.len(), config.window_set.label());
    println!();

    let warmup_duration = interval_duration * (warmup_bars + 1) as i32;

    let periods = group_into_periods(&windows, Duration::days(COOLDOWN_WARMUP_DAYS));
    let global_start = periods.iter().map(|p| p.start).min().unwrap()
        - Duration::days(COOLDOWN_WARMUP_DAYS)
        - warmup_duration;
    let global_end =
        periods.iter().map(|p| p.end).max().unwrap() + Duration::hours(DATA_BUFFER_HOURS);

    let mut store = CandleStore::new();
    let client = BinanceClient::new(MarketType::Futures);

    let mut failed_symbols: Vec<String> = Vec::new();
    for sym in &api_symbols {
        match ensure_candles(
            &mut store,
            &client,
            sym,
            &analysis_interval,
            global_start,
            global_end,
        ) {
            Ok(candles) if candles.is_empty() => {
                eprintln!("  ERROR: Empty {} response for {sym}", analysis_interval);
                failed_symbols.push(sym.clone());
            }
            Err(e) => {
                eprintln!("  ERROR: {e}");
                failed_symbols.push(sym.clone());
            }
            Ok(_) => {}
        }
    }
    if !failed_symbols.is_empty() {
        eprintln!(
            "\nFATAL: Missing {} candle data for {} symbol(s): {}",
            analysis_interval,
            failed_symbols.len(),
            failed_symbols.join(", "),
        );
        eprintln!("Cannot validate with incomplete data. Aborting.");
        return 1;
    }

    let iv_indicator_map = strategy.indicator_columns_per_interval();
    let earliest_period_start = periods.iter().map(|p| p.start).min().unwrap();
    ensure_additional_interval_candles(
        &mut store,
        &client,
        strategy,
        &api_symbols,
        &iv_indicator_map,
        &calib_config,
        earliest_period_start,
        global_end,
    );

    let gen_result = generate_all_signals(
        strategy,
        &mut store,
        &client,
        &api_symbols,
        &windows,
        warmup_bars,
        &analysis_interval,
        interval_duration,
        global_start,
        global_end,
    );

    let pipeline_state = gen_result.state;

    let window_signals: Vec<&Signal> = gen_result
        .raw_signals
        .iter()
        .filter(|s| {
            windows
                .iter()
                .any(|w| s.signal_date >= w.start && s.signal_date < w.end)
        })
        .map(|s| s.as_ref())
        .collect();

    println!("Reference signals (pre-cooldown): {}", window_signals.len());

    if window_signals.is_empty() {
        println!("No signals generated — nothing to validate.");
        println!("\nFAIL — strategy produced zero signals (cannot verify correctness)");
        return 1;
    }

    let intervals = &pipeline_state.calibration_intervals;
    let mut params_cursor: usize = 0;

    let mut vec_pool: Vec<Vec<Candle>> = Vec::with_capacity(api_symbols.len());
    let mut truncated: BTreeMap<String, Vec<Candle>> = BTreeMap::new();

    let warmup_dur = interval_duration * (warmup_bars + 1) as i32;
    let mut period_idx = 0usize;

    let mut passed = 0usize;
    let mut failed = 0usize;
    let mut failures: Vec<(String, DateTime<Utc>, String)> = Vec::new();

    for (i, signal) in window_signals.iter().enumerate() {
        let signal_date = signal.signal_date;

        while period_idx + 1 < periods.len() && signal_date >= periods[period_idx + 1].start {
            period_idx += 1;
        }
        let eval_sig_start = periods[period_idx].start - Duration::days(COOLDOWN_WARMUP_DAYS);
        let fetch_start = eval_sig_start - warmup_dur;
        rebuild_truncated_candles(
            &mut store,
            &analysis_interval,
            &api_symbols,
            fetch_start,
            signal_date,
            interval_duration,
            &mut truncated,
            &mut vec_pool,
        );

        let active_params = active_params_for_signal(intervals, &mut params_cursor, signal_date);

        let mut sig_ctx = pipeline_state.full_ctx.clone();
        sig_ctx.clip_in_place(signal_date);
        let sig_htf = pipeline_state.full_htf.truncated_at(signal_date);

        let replay_matched = replay_signal(
            strategy,
            &truncated,
            &active_params,
            &sig_ctx,
            &sig_htf,
            warmup_bars,
            interval_duration,
            signal,
        );

        if replay_matched {
            passed += 1;
        } else {
            failed += 1;
            failures.push((
                signal.ticker.clone(),
                signal.signal_date,
                format!(
                    "{:?} at {}",
                    signal.position_type,
                    signal.signal_date.format("%Y-%m-%d %H:%M")
                ),
            ));
        }

        if (i + 1) % 50 == 0 || i + 1 == window_signals.len() {
            eprint!(
                "\r  Validated {}/{} signals ({} passed, {} failed)",
                i + 1,
                window_signals.len(),
                passed,
                failed,
            );
        }
    }
    eprintln!();

    println!();
    println!(
        "Validation complete in {:.1}s",
        t_total.elapsed().as_secs_f64()
    );
    println!("  Passed: {passed}");
    println!("  Failed: {failed}");

    if !failures.is_empty() {
        println!("\nLookahead violations:");
        for (ticker, _time, desc) in &failures {
            println!("  {ticker} — {desc}");
        }

        let artifact = serde_json::json!({
            "strategy": strategy.name(),
            "window_set": config.window_set.label(),
            "total_signals": window_signals.len(),
            "passed": passed,
            "failed": failed,
            "failures": failures.iter().map(|(ticker, time, desc)| {
                serde_json::json!({
                    "ticker": ticker,
                    "signal_date": time.to_rfc3339(),
                    "description": desc,
                })
            }).collect::<Vec<_>>(),
            "timestamp": Utc::now().to_rfc3339(),
        });

        let artifact_path = "validation_result.json";
        if let Ok(json) = serde_json::to_string_pretty(&artifact) {
            let _ = fs::write(artifact_path, json);
            println!("\nArtifact saved: {artifact_path}");
        }

        println!("\nFAIL — {failed} lookahead violation(s) detected");
        1
    } else {
        println!("\nPASS — all signals reproduce without future data");
        0
    }
}

/// Approximate equality for `Option<f64>` — absorbs floating-point rounding
/// drift from SMA sliding-window accumulation while still catching real
/// lookahead-induced differences (which shift tp/sl by 0.1+ pct points).
fn f64_opt_approx_eq(a: Option<f64>, b: Option<f64>) -> bool {
    match (a, b) {
        (Some(x), Some(y)) => (x - y).abs() < 1e-10,
        (None, None) => true,
        _ => false,
    }
}

/// Replay a single signal with pre-computed truncated data.
fn replay_signal(
    strategy: &dyn ResearchStrategy,
    truncated_candles: &BTreeMap<String, Vec<Candle>>,
    active_params: &HashMap<String, serde_json::Value>,
    ctx: &ContextMap,
    htf: &HtfData,
    warmup_bars: usize,
    interval_duration: Duration,
    reference: &Signal,
) -> bool {
    let signal_date = reference.signal_date;
    let warmup_dur = interval_duration * (warmup_bars + 1) as i32;
    let sig_start = signal_date - warmup_dur - Duration::days(COOLDOWN_WARMUP_DAYS);

    let gen_end = signal_date + Duration::seconds(1);
    let candles_view: BTreeMap<String, &[Candle]> = truncated_candles
        .iter()
        .map(|(sym, v)| (sym.clone(), v.as_slice()))
        .collect();
    let re_signals = strategy.generate_signals(
        &candles_view,
        sig_start,
        gen_end,
        active_params,
        ctx,
        htf,
    );

    re_signals.iter().any(|s| {
        s.signal_date == reference.signal_date
            && s.ticker == reference.ticker
            && s.position_type == reference.position_type
            && f64_opt_approx_eq(s.tp_pct, reference.tp_pct)
            && f64_opt_approx_eq(s.sl_pct, reference.sl_pct)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use serde_json::json;
    use std::collections::BTreeSet;

    fn dt(y: i32, m: u32, d: u32, h: u32, min: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(y, m, d, h, min, 0).unwrap()
    }

    fn candle(start: DateTime<Utc>, interval: Duration, price: f64) -> Candle {
        Candle {
            open_time: start,
            close_time: start + interval,
            open: price,
            high: price + 1.0,
            low: price - 1.0,
            close: price + 0.5,
            volume: price * 10.0,
            taker_buy_volume: price * 5.0,
        }
    }

    fn assert_candle_map_eq(
        actual: &BTreeMap<String, Vec<Candle>>,
        expected: &BTreeMap<String, Vec<Candle>>,
    ) {
        let actual_keys: BTreeSet<&String> = actual.keys().collect();
        let expected_keys: BTreeSet<&String> = expected.keys().collect();
        assert_eq!(actual_keys, expected_keys);
        for key in actual_keys {
            assert_eq!(
                serde_json::to_value(actual.get(key).unwrap()).unwrap(),
                serde_json::to_value(expected.get(key).unwrap()).unwrap(),
                "mismatch for symbol {key}"
            );
        }
    }

    fn naive_truncated_candles(
        store: &mut CandleStore,
        analysis_interval: &str,
        api_symbols: &[String],
        fetch_start: DateTime<Utc>,
        signal_date: DateTime<Utc>,
        interval_duration: Duration,
    ) -> BTreeMap<String, Vec<Candle>> {
        let mut truncated = BTreeMap::new();
        for sym in api_symbols {
            let candles = store.get_range(
                sym,
                analysis_interval,
                fetch_start,
                signal_date + interval_duration,
            );
            let filtered: Vec<Candle> = candles
                .into_iter()
                .filter(|c| c.close_time <= signal_date)
                .collect();
            if !filtered.is_empty() {
                truncated.insert(sym.clone(), filtered);
            }
        }
        truncated
    }

    #[test]
    fn rebuild_truncated_candles_matches_naive_for_cached_intervals() {
        let sym = "UT_VALIDATE_TRUNC_1H".to_string();
        let api_symbols = vec![sym.clone()];
        let interval = Duration::hours(1);
        let candles = vec![
            candle(dt(2024, 1, 1, 0, 0), interval, 100.0),
            candle(dt(2024, 1, 1, 1, 0), interval, 101.0),
            candle(dt(2024, 1, 1, 2, 0), interval, 102.0),
            candle(dt(2024, 1, 1, 3, 0), interval, 103.0),
        ];

        let mut store = CandleStore::new();
        store.put(&sym, "1h", &candles);

        let mut truncated = BTreeMap::new();
        let mut pool = Vec::new();

        for signal_date in [dt(2024, 1, 1, 2, 0), dt(2024, 1, 1, 4, 0)] {
            rebuild_truncated_candles(
                &mut store,
                "1h",
                &api_symbols,
                dt(2024, 1, 1, 0, 0),
                signal_date,
                interval,
                &mut truncated,
                &mut pool,
            );
            let expected = naive_truncated_candles(
                &mut store,
                "1h",
                &api_symbols,
                dt(2024, 1, 1, 0, 0),
                signal_date,
                interval,
            );
            assert_candle_map_eq(&truncated, &expected);
        }
    }

    #[test]
    fn rebuild_truncated_candles_matches_naive_for_1m_intervals() {
        let sym = "UT_VALIDATE_TRUNC_1M".to_string();
        let api_symbols = vec![sym.clone()];
        let interval = Duration::minutes(1);
        let candles = vec![
            candle(dt(2024, 1, 1, 0, 0), interval, 100.0),
            candle(dt(2024, 1, 1, 0, 1), interval, 101.0),
            candle(dt(2024, 1, 1, 0, 2), interval, 102.0),
            candle(dt(2024, 1, 1, 0, 3), interval, 103.0),
        ];

        let mut store = CandleStore::new();
        store.put(&sym, "1m", &candles);

        let mut truncated = BTreeMap::new();
        let mut pool = Vec::new();

        for signal_date in [dt(2024, 1, 1, 0, 2), dt(2024, 1, 1, 0, 4)] {
            rebuild_truncated_candles(
                &mut store,
                "1m",
                &api_symbols,
                dt(2024, 1, 1, 0, 0),
                signal_date,
                interval,
                &mut truncated,
                &mut pool,
            );
            let expected = naive_truncated_candles(
                &mut store,
                "1m",
                &api_symbols,
                dt(2024, 1, 1, 0, 0),
                signal_date,
                interval,
            );
            assert_candle_map_eq(&truncated, &expected);
        }
    }

    #[test]
    fn active_params_for_signal_matches_linear_scan() {
        let intervals = vec![
            CalibrationInterval {
                start: dt(2024, 1, 1, 0, 0),
                end: dt(2024, 1, 1, 2, 0),
                params: HashMap::from([("alpha".into(), json!(1))]),
            },
            CalibrationInterval {
                start: dt(2024, 1, 1, 3, 0),
                end: dt(2024, 1, 1, 5, 0),
                params: HashMap::from([("alpha".into(), json!(2))]),
            },
        ];
        let mut cursor = 0usize;

        for signal_date in [
            dt(2023, 12, 31, 23, 0),
            dt(2024, 1, 1, 0, 30),
            dt(2024, 1, 1, 2, 0),
            dt(2024, 1, 1, 2, 30),
            dt(2024, 1, 1, 3, 30),
            dt(2024, 1, 1, 6, 0),
        ] {
            let actual = active_params_for_signal(&intervals, &mut cursor, signal_date);
            let expected = intervals
                .iter()
                .find(|interval| signal_date >= interval.start && signal_date < interval.end)
                .map(|interval| interval.params.clone())
                .unwrap_or_default();
            assert_eq!(actual, expected);
        }
    }
}
