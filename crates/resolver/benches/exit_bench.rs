//! Benchmarks for the exit resolver — measures baseline cost of resolve_exit
//! under realistic candle distributions. Used to decide whether per-signal
//! `Vec<MiniCandle>` allocation is worth replacing with `SmallVec`.
//!
//! Run: `cargo bench -p claude-trader-resolver --bench exit_bench`

use claude_trader_resolver::exit::{resolve_exit, MiniCandle, MiniTrade};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use smallvec::SmallVec;

fn make_candles(n: usize, seed: f64) -> Vec<MiniCandle> {
    let mut out = Vec::with_capacity(n);
    let mut price = 100.0 + seed;
    for i in 0..n {
        let open_ms = i as i64 * 3_600_000;
        let close_ms = open_ms + 3_600_000;
        let drift = (i as f64 * 0.13 + seed).sin() * 0.5;
        let high = price + 0.6 + drift.abs();
        let low = price - 0.6 - drift.abs();
        out.push(MiniCandle {
            open_time_ms: open_ms,
            close_time_ms: close_ms,
            high,
            low,
        });
        price += drift;
    }
    out
}

/// Always-open window — loop runs through every hour candle without finding a barrier.
/// Exercises the full iteration and worst-case path.
fn bench_resolve_exit_timeout(c: &mut Criterion) {
    let hours = make_candles(72, 0.0);
    // TP/SL far outside the candle range, so nothing hits.
    let tp = 1_000.0;
    let sl = -1_000.0;
    let entry_ms = 0;
    let end_ms = Some(72 * 3_600_000);
    let mut fetch_min = |_s: i64, _e: i64| -> Vec<MiniCandle> { Vec::new() };
    let mut fetch_trades = |_s: i64, _e: i64| -> Vec<MiniTrade> { Vec::new() };

    c.bench_function("resolve_exit_timeout_72h", |b| {
        b.iter(|| {
            black_box(resolve_exit(
                black_box(&hours),
                black_box(true),
                black_box(tp),
                black_box(sl),
                black_box(entry_ms),
                black_box(end_ms),
                &mut fetch_min,
                &mut fetch_trades,
            ))
        });
    });
}

/// Early-TP case — barrier hit in the 2nd hour, realistic for a trending exit.
fn bench_resolve_exit_early_tp(c: &mut Criterion) {
    let mut hours = make_candles(72, 0.2);
    // Force a TP hit at hour 2.
    hours[2].high = 120.0;
    let tp = 105.0;
    let sl = 90.0;
    let entry_ms = 0;
    let end_ms = Some(72 * 3_600_000);
    let mut fetch_min = |_s: i64, _e: i64| -> Vec<MiniCandle> { Vec::new() };
    let mut fetch_trades = |_s: i64, _e: i64| -> Vec<MiniTrade> { Vec::new() };

    c.bench_function("resolve_exit_tp_hit_hour2", |b| {
        b.iter(|| {
            black_box(resolve_exit(
                black_box(&hours),
                black_box(true),
                black_box(tp),
                black_box(sl),
                black_box(entry_ms),
                black_box(end_ms),
                &mut fetch_min,
                &mut fetch_trades,
            ))
        });
    });
}

/// Isolated Vec<MiniCandle> allocation measurement — the exact line from kernel.rs:91
/// (`hour_candles.iter().map(MiniCandle::from_candle).collect()`), stripped of the
/// resolver logic. Tells us the per-signal cost of the allocation alone.
fn bench_mini_candle_alloc(c: &mut Criterion) {
    use claude_trader_models::Candle;
    use chrono::{DateTime, TimeZone, Utc};

    fn mk(open_ms: i64, close_ms: i64, price: f64) -> Candle {
        Candle {
            open_time: DateTime::<Utc>::from_timestamp_millis(open_ms).unwrap(),
            close_time: DateTime::<Utc>::from_timestamp_millis(close_ms).unwrap(),
            open: price,
            high: price + 1.0,
            low: price - 1.0,
            close: price + 0.5,
            volume: 1000.0,
            taker_buy_volume: 500.0,
        }
    }

    let candles: Vec<Candle> = (0..72)
        .map(|i| mk(i as i64 * 3_600_000, (i as i64 + 1) * 3_600_000, 100.0 + i as f64 * 0.1))
        .collect();

    // Prevent unused warnings
    let _ = Utc.timestamp_millis_opt(0);

    c.bench_function("mini_candle_collect_72_vec", |b| {
        b.iter(|| {
            let v: Vec<MiniCandle> = black_box(candles.iter())
                .map(MiniCandle::from_candle)
                .collect();
            black_box(v)
        });
    });

    c.bench_function("mini_candle_collect_72_smallvec", |b| {
        b.iter(|| {
            let v: SmallVec<[MiniCandle; 72]> = black_box(candles.iter())
                .map(MiniCandle::from_candle)
                .collect();
            black_box(v)
        });
    });
}

criterion_group!(
    benches,
    bench_resolve_exit_timeout,
    bench_resolve_exit_early_tp,
    bench_mini_candle_alloc
);
criterion_main!(benches);
