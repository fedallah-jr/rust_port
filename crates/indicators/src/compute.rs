//! Indicator computation functions.
//!
//! Each indicator is implemented as a function that takes column references
//! and returns a new column. The main entry point `compute_indicators()`
//! resolves dependencies and computes in topological order.

use std::borrow::Cow;
use std::collections::HashMap;

use crate::registry::{resolve_order, RAW_INPUTS};
use crate::{IndicatorResult, OhlcvFrame};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const RSI_ALPHA: f64 = 1.0 / 14.0;
const EMA_20_ALPHA: f64 = 2.0 / 21.0;
const T3_SPAN: f64 = 5.0;
const T3_ALPHA: f64 = 2.0 / (T3_SPAN + 1.0);

// T3 coefficients (vfactor = 0.7)
const T3_VFACTOR: f64 = 0.7;
const T3_C1: f64 = -(T3_VFACTOR * T3_VFACTOR * T3_VFACTOR);
const T3_C2: f64 = 3.0 * T3_VFACTOR * T3_VFACTOR + 3.0 * T3_VFACTOR * T3_VFACTOR * T3_VFACTOR;
const T3_C3: f64 =
    -6.0 * T3_VFACTOR * T3_VFACTOR - 3.0 * T3_VFACTOR - 3.0 * T3_VFACTOR * T3_VFACTOR * T3_VFACTOR;
const T3_C4: f64 =
    1.0 + 3.0 * T3_VFACTOR + T3_VFACTOR * T3_VFACTOR * T3_VFACTOR + 3.0 * T3_VFACTOR * T3_VFACTOR;

// Momentum slope constants for n=20
const SLOPE_N: usize = 20;
const SLOPE_XS_SUM: f64 = 190.0; // sum(0..20)
const SLOPE_XS_DOT: f64 = 2470.0; // sum(i*i for i in 0..20)
const SLOPE_DENOM: f64 = SLOPE_N as f64 * SLOPE_XS_DOT - SLOPE_XS_SUM * SLOPE_XS_SUM;

// ---------------------------------------------------------------------------
// Main entry point
// ---------------------------------------------------------------------------

/// Compute requested indicators on an OHLCV frame.
///
/// Returns a map of indicator name → computed column. Only the requested
/// indicators are included (internal/dependency columns are dropped).
pub fn compute_indicators(frame: &OhlcvFrame, indicators: &[&str]) -> Result<IndicatorResult, String> {
    let n = frame.close.len();
    if frame.open.len() != n
        || frame.high.len() != n
        || frame.low.len() != n
        || frame.volume.len() != n
        || frame.taker_buy_volume.len() != n
    {
        return Err(format!(
            "OhlcvFrame column length mismatch: close={}, open={}, high={}, low={}, volume={}, taker_buy_volume={}",
            n,
            frame.open.len(),
            frame.high.len(),
            frame.low.len(),
            frame.volume.len(),
            frame.taker_buy_volume.len(),
        ));
    }
    if frame.is_empty() || indicators.is_empty() {
        return Ok(HashMap::new());
    }

    let order = resolve_order(indicators)?;

    // Columns store: raw inputs are borrowed, computed indicators are owned.
    let mut cols: HashMap<&str, Cow<'_, [f64]>> = HashMap::new();
    cols.insert("open", Cow::Borrowed(&frame.open));
    cols.insert("high", Cow::Borrowed(&frame.high));
    cols.insert("low", Cow::Borrowed(&frame.low));
    cols.insert("close", Cow::Borrowed(&frame.close));
    cols.insert("volume", Cow::Borrowed(&frame.volume));
    cols.insert("taker_buy_volume", Cow::Borrowed(&frame.taker_buy_volume));

    // Compute in topological order
    for &name in &order {
        let col = compute_single(name, &cols, n);
        cols.insert(name, Cow::Owned(col));
    }

    // Return only requested indicators (raw inputs are filtered out and dropped, not cloned)
    let requested: std::collections::HashSet<&str> = indicators.iter().copied().collect();
    let raw: std::collections::HashSet<&str> = RAW_INPUTS.iter().copied().collect();

    Ok(cols.into_iter()
        .filter(|(k, _)| requested.contains(k) && !raw.contains(k))
        .map(|(k, v)| (k.to_string(), v.into_owned()))
        .collect())
}

// ---------------------------------------------------------------------------
// Per-indicator computation
// ---------------------------------------------------------------------------

fn compute_single(name: &str, cols: &HashMap<&str, Cow<'_, [f64]>>, n: usize) -> Vec<f64> {
    match name {
        "_delta_close" => delta(&cols["close"]),
        "_gain_ewm_14" => ewm_mean_transform(
            &cols["_delta_close"],
            |v| if v > 0.0 { v } else { 0.0 },
            RSI_ALPHA,
            14,
        ),
        "_loss_ewm_14" => ewm_mean_transform(
            &cols["_delta_close"],
            |v| if v < 0.0 { -v } else { 0.0 },
            RSI_ALPHA,
            14,
        ),
        "rsi_14" => compute_rsi(&cols["_gain_ewm_14"], &cols["_loss_ewm_14"]),

        "true_range" => compute_true_range(&cols["high"], &cols["low"], &cols["close"]),
        "atr_14" => sma(&cols["true_range"], 14),
        "atr_72_avg" => sma(&cols["atr_14"], 72),
        "atr_ratio" => div(&cols["atr_14"], &cols["atr_72_avg"]),

        "ret_24h" => pct_change(&cols["close"], 24),
        "ret_48h" => pct_change(&cols["close"], 48),
        "ret_72h" => pct_change(&cols["close"], 72),

        "vol_sma_20" => sma(&cols["volume"], 20),
        "vol_ratio" => div(&cols["volume"], &cols["vol_sma_20"]),

        "_bb_ma_20" => sma(&cols["close"], 20),
        "_bb_std_20" => rolling_std(&cols["close"], 20),
        "bb_upper" => add_scaled(&cols["_bb_ma_20"], &cols["_bb_std_20"], 2.0),
        "bb_lower" => sub_scaled(&cols["_bb_ma_20"], &cols["_bb_std_20"], 2.0),
        "bb_pct_b" => compute_bb_pct_b(&cols["close"], &cols["bb_upper"], &cols["bb_lower"]),
        "bb_width" => compute_bb_width(&cols["bb_upper"], &cols["bb_lower"], &cols["_bb_ma_20"]),

        "ema_20" => ewm_mean(&cols["close"], EMA_20_ALPHA, 0),
        "kc_upper" => add_scaled(&cols["ema_20"], &cols["atr_14"], 1.5),
        "kc_lower" => sub_scaled(&cols["ema_20"], &cols["atr_14"], 1.5),

        "squeeze_on" => compute_squeeze_on(
            &cols["bb_upper"],
            &cols["bb_lower"],
            &cols["kc_upper"],
            &cols["kc_lower"],
        ),
        "squeeze_count" => compute_squeeze_count(&cols["squeeze_on"]),

        "mom_slope" => compute_mom_slope(&cols["close"]),

        "body" => sub(&cols["close"], &cols["open"]),
        "body_ratio" => compute_body_ratio(&cols["body"], &cols["high"], &cols["low"]),

        "_plus_dm" => compute_plus_dm(&cols["high"], &cols["low"]),
        "_minus_dm" => compute_minus_dm(&cols["high"], &cols["low"]),
        "_smoothed_plus_dm" => ewm_mean(&cols["_plus_dm"], RSI_ALPHA, 14),
        "_smoothed_minus_dm" => ewm_mean(&cols["_minus_dm"], RSI_ALPHA, 14),
        "_smoothed_tr" => ewm_mean(&cols["true_range"], RSI_ALPHA, 14),
        "_plus_di" => compute_di(&cols["_smoothed_plus_dm"], &cols["_smoothed_tr"]),
        "_minus_di" => compute_di(&cols["_smoothed_minus_dm"], &cols["_smoothed_tr"]),
        "_dx" => compute_dx(&cols["_plus_di"], &cols["_minus_di"]),
        "adx_14" => ewm_mean(&cols["_dx"], RSI_ALPHA, 14),

        "volume_delta" => {
            let tbv = &cols["taker_buy_volume"];
            let vol = &cols["volume"];
            tbv.iter()
                .zip(vol.iter())
                .map(|(&t, &v)| 2.0 * t - v)
                .collect()
        }
        "cvd" => cumsum(&cols["volume_delta"]),

        "_typ_price" => typical_price(&cols["high"], &cols["low"], &cols["close"]),
        "vwap_20" => rolling_vwap(&cols["_typ_price"], &cols["volume"], 20),
        "vwap_48" => rolling_vwap(&cols["_typ_price"], &cols["volume"], 48),
        "vwap_dev_20" => pct_deviation(&cols["close"], &cols["vwap_20"]),
        "vwap_dev_48" => pct_deviation(&cols["close"], &cols["vwap_48"]),

        "poc_48" => rolling_poc(
            &cols["high"],
            &cols["low"],
            &cols["close"],
            &cols["volume"],
            48,
            50,
        ),
        "poc_dev_48" => pct_deviation(&cols["close"], &cols["poc_48"]),

        "_t3_e1" => ewm_mean(&cols["close"], T3_ALPHA, 0),
        "_t3_e2" => ewm_mean(&cols["_t3_e1"], T3_ALPHA, 0),
        "_t3_e3" => ewm_mean(&cols["_t3_e2"], T3_ALPHA, 0),
        "_t3_e4" => ewm_mean(&cols["_t3_e3"], T3_ALPHA, 0),
        "_t3_e5" => ewm_mean(&cols["_t3_e4"], T3_ALPHA, 0),
        "_t3_e6" => ewm_mean(&cols["_t3_e5"], T3_ALPHA, 0),
        "t3" => compute_t3(
            &cols["_t3_e3"],
            &cols["_t3_e4"],
            &cols["_t3_e5"],
            &cols["_t3_e6"],
        ),

        _ => vec![f64::NAN; n],
    }
}

// ---------------------------------------------------------------------------
// Primitive operations
// ---------------------------------------------------------------------------

/// Difference series: out[i] = data[i] - data[i-1], out[0] = NaN.
fn delta(data: &[f64]) -> Vec<f64> {
    let mut out = vec![f64::NAN; data.len()];
    for i in 1..data.len() {
        out[i] = data[i] - data[i - 1];
    }
    out
}

/// Exponentially weighted moving mean.
///
/// Matches pandas `ewm(alpha=alpha, min_periods=min_periods).mean()`.
fn ewm_mean(data: &[f64], alpha: f64, min_periods: usize) -> Vec<f64> {
    ewm_mean_transform(data, std::convert::identity, alpha, min_periods)
}

/// Variant of `ewm_mean` that applies `transform` to each input element before
/// accumulating. Lets RSI gain/loss computations avoid materializing an
/// intermediate masked vector. NaN handling matches `ewm_mean`: if the
/// transformed value is NaN, the output is NaN and the running state is
/// preserved — same behavior as calling `ewm_mean(&transformed, alpha, mp)`.
fn ewm_mean_transform<F>(data: &[f64], transform: F, alpha: f64, min_periods: usize) -> Vec<f64>
where
    F: Fn(f64) -> f64,
{
    let n = data.len();
    let mut out = vec![f64::NAN; n];
    let mut numer = 0.0f64;
    let mut denom = 0.0f64;
    let one_minus_alpha = 1.0 - alpha;
    let mut count = 0usize;

    for i in 0..n {
        let v = transform(data[i]);
        if v.is_nan() {
            out[i] = f64::NAN;
            continue;
        }
        count += 1;
        if count == 1 {
            numer = v;
            denom = 1.0;
        } else {
            numer = numer * one_minus_alpha + v;
            denom = denom * one_minus_alpha + 1.0;
        }
        if count >= min_periods {
            out[i] = numer / denom;
        }
    }
    out
}

/// Simple moving average with window — O(n) sliding window.
/// Matches pandas `rolling(window).mean()` — requires `window` non-NaN values
/// in the window (min_periods=window). NaN inputs are handled correctly.
fn sma(data: &[f64], window: usize) -> Vec<f64> {
    let n = data.len();
    let mut out = vec![f64::NAN; n];
    if n < window || window == 0 {
        return out;
    }
    let mut sum = 0.0f64;
    let mut count = 0usize;

    // Seed first window
    for j in 0..window {
        if !data[j].is_nan() {
            sum += data[j];
            count += 1;
        }
    }
    if count == window {
        out[window - 1] = sum / window as f64;
    }

    // Slide
    for i in window..n {
        let leaving = data[i - window];
        if !leaving.is_nan() {
            sum -= leaving;
            count -= 1;
        }
        let entering = data[i];
        if !entering.is_nan() {
            sum += entering;
            count += 1;
        }
        if count == window {
            out[i] = sum / window as f64;
        }
    }
    out
}

/// Rolling standard deviation (ddof=1, matching pandas default) — O(n) sliding window.
/// NaN-aware: requires `window` non-NaN values.
fn rolling_std(data: &[f64], window: usize) -> Vec<f64> {
    let n = data.len();
    let mut out = vec![f64::NAN; n];
    if n < window || window < 2 {
        return out;
    }
    // Welford sliding window: tracks running mean and M2 (sum of squared
    // deviations from mean).  Numerically stable — avoids catastrophic
    // cancellation that plagues the sum_sq - sum²/n identity.
    let mut mean = 0.0f64;
    let mut m2 = 0.0f64;
    let mut count = 0usize;

    // Seed first window
    for j in 0..window {
        if !data[j].is_nan() {
            count += 1;
            let delta = data[j] - mean;
            mean += delta / count as f64;
            let delta2 = data[j] - mean;
            m2 += delta * delta2;
        }
    }
    if count == window {
        out[window - 1] = (m2 / (count as f64 - 1.0)).max(0.0).sqrt();
    }

    // Slide
    for i in window..n {
        let leaving = data[i - window];
        if !leaving.is_nan() {
            if count == 1 {
                mean = 0.0;
                m2 = 0.0;
            } else {
                let delta = leaving - mean;
                mean -= delta / (count - 1) as f64;
                let delta2 = leaving - mean;
                m2 -= delta * delta2;
            }
            count -= 1;
        }
        let entering = data[i];
        if !entering.is_nan() {
            count += 1;
            let delta = entering - mean;
            mean += delta / count as f64;
            let delta2 = entering - mean;
            m2 += delta * delta2;
        }
        if count == window {
            out[i] = (m2 / (count as f64 - 1.0)).max(0.0).sqrt();
        }
    }
    out
}

/// Percent change: out[i] = (data[i] / data[i-period] - 1) * 100.
fn pct_change(data: &[f64], period: usize) -> Vec<f64> {
    let n = data.len();
    let mut out = vec![f64::NAN; n];
    for i in period..n {
        if data[i - period] != 0.0 {
            out[i] = (data[i] / data[i - period] - 1.0) * 100.0;
        }
    }
    out
}

/// Element-wise division, NaN where denominator is 0 or NaN.
fn div(a: &[f64], b: &[f64]) -> Vec<f64> {
    a.iter()
        .zip(b.iter())
        .map(|(&x, &y)| {
            if y == 0.0 || y.is_nan() || x.is_nan() {
                f64::NAN
            } else {
                x / y
            }
        })
        .collect()
}

/// a + scale * b
fn add_scaled(a: &[f64], b: &[f64], scale: f64) -> Vec<f64> {
    a.iter()
        .zip(b.iter())
        .map(|(&x, &y)| x + scale * y)
        .collect()
}

/// a - scale * b
fn sub_scaled(a: &[f64], b: &[f64], scale: f64) -> Vec<f64> {
    a.iter()
        .zip(b.iter())
        .map(|(&x, &y)| x - scale * y)
        .collect()
}

/// Element-wise subtraction.
fn sub(a: &[f64], b: &[f64]) -> Vec<f64> {
    a.iter().zip(b.iter()).map(|(&x, &y)| x - y).collect()
}

/// Cumulative sum.
fn cumsum(data: &[f64]) -> Vec<f64> {
    let mut out = Vec::with_capacity(data.len());
    let mut acc = 0.0f64;
    for &v in data {
        acc += v;
        out.push(acc);
    }
    out
}

/// Typical price: (high + low + close) / 3. Used as the price series
/// inside rolling VWAP.
fn typical_price(high: &[f64], low: &[f64], close: &[f64]) -> Vec<f64> {
    high.iter()
        .zip(low.iter())
        .zip(close.iter())
        .map(|((&h, &l), &c)| (h + l + c) / 3.0)
        .collect()
}

/// Rolling volume-weighted average price over `window` bars:
/// `sum(typ[i-w+1..=i] * vol[i-w+1..=i]) / sum(vol[i-w+1..=i])`.
/// NaN for the first `window - 1` outputs; NaN when the window volume is
/// zero. Incremental O(n) with a sliding-window accumulator.
fn rolling_vwap(typ: &[f64], volume: &[f64], window: usize) -> Vec<f64> {
    let n = typ.len();
    let mut out = vec![f64::NAN; n];
    if window == 0 || n < window {
        return out;
    }
    let mut sum_pv = 0.0f64;
    let mut sum_v = 0.0f64;
    for i in 0..n {
        sum_pv += typ[i] * volume[i];
        sum_v += volume[i];
        if i >= window {
            sum_pv -= typ[i - window] * volume[i - window];
            sum_v -= volume[i - window];
        }
        if i + 1 >= window && sum_v > 0.0 {
            out[i] = sum_pv / sum_v;
        }
    }
    out
}

/// Percent deviation of `a` from `b`: `(a - b) / b * 100`. NaN when `b`
/// is NaN or zero; used for VWAP / POC deviation columns.
fn pct_deviation(a: &[f64], b: &[f64]) -> Vec<f64> {
    a.iter()
        .zip(b.iter())
        .map(|(&x, &y)| {
            if y.is_nan() || x.is_nan() || y == 0.0 {
                f64::NAN
            } else {
                (x / y - 1.0) * 100.0
            }
        })
        .collect()
}

/// Rolling Point of Control: over the last `window` bars, bin the price
/// range into `bins` equal buckets and return the centre price of the bin
/// with the largest cumulative volume. Each bar attributes its entire
/// volume to the bin containing its close. NaN for the first
/// `window - 1` outputs.
fn rolling_poc(
    high: &[f64],
    low: &[f64],
    close: &[f64],
    volume: &[f64],
    window: usize,
    bins: usize,
) -> Vec<f64> {
    let n = close.len();
    let mut out = vec![f64::NAN; n];
    if window == 0 || bins == 0 || n < window {
        return out;
    }
    let mut vol_by_bin = vec![0.0f64; bins];
    for i in (window - 1)..n {
        let start = i + 1 - window;
        let mut lo = f64::INFINITY;
        let mut hi = f64::NEG_INFINITY;
        for j in start..=i {
            if low[j] < lo {
                lo = low[j];
            }
            if high[j] > hi {
                hi = high[j];
            }
        }
        if !(hi.is_finite() && lo.is_finite()) || hi <= lo {
            out[i] = close[i];
            continue;
        }
        let bin_w = (hi - lo) / bins as f64;
        for b in vol_by_bin.iter_mut() {
            *b = 0.0;
        }
        for j in start..=i {
            let mut bi = ((close[j] - lo) / bin_w).floor() as isize;
            if bi < 0 {
                bi = 0;
            }
            if bi >= bins as isize {
                bi = bins as isize - 1;
            }
            vol_by_bin[bi as usize] += volume[j];
        }
        let mut best_bin = 0usize;
        let mut best_vol = f64::NEG_INFINITY;
        for (idx, &v) in vol_by_bin.iter().enumerate() {
            if v > best_vol {
                best_vol = v;
                best_bin = idx;
            }
        }
        out[i] = lo + (best_bin as f64 + 0.5) * bin_w;
    }
    out
}

// ---------------------------------------------------------------------------
// Indicator-specific computations
// ---------------------------------------------------------------------------

fn compute_rsi(gain_mean: &[f64], loss_mean: &[f64]) -> Vec<f64> {
    gain_mean
        .iter()
        .zip(loss_mean.iter())
        .map(|(&g, &l)| {
            if g.is_nan() || l.is_nan() {
                f64::NAN
            } else if l == 0.0 {
                if g == 0.0 {
                    f64::NAN
                } else {
                    100.0
                }
            } else {
                100.0 - (100.0 / (1.0 + g / l))
            }
        })
        .collect()
}

fn compute_true_range(high: &[f64], low: &[f64], close: &[f64]) -> Vec<f64> {
    let n = high.len();
    let mut out = vec![f64::NAN; n];
    if n > 0 {
        out[0] = high[0] - low[0];
    }
    for i in 1..n {
        let hl = high[i] - low[i];
        let hc = (high[i] - close[i - 1]).abs();
        let lc = (low[i] - close[i - 1]).abs();
        out[i] = hl.max(hc).max(lc);
    }
    out
}

fn compute_bb_pct_b(close: &[f64], upper: &[f64], lower: &[f64]) -> Vec<f64> {
    close
        .iter()
        .zip(upper.iter())
        .zip(lower.iter())
        .map(|((&c, &u), &l)| {
            let width = u - l;
            if width == 0.0 || width.is_nan() || c.is_nan() {
                f64::NAN
            } else {
                let v = (c - l) / width;
                if v.is_infinite() {
                    f64::NAN
                } else {
                    v
                }
            }
        })
        .collect()
}

fn compute_bb_width(upper: &[f64], lower: &[f64], ma: &[f64]) -> Vec<f64> {
    upper
        .iter()
        .zip(lower.iter())
        .zip(ma.iter())
        .map(|((&u, &l), &m)| {
            if m == 0.0 || m.is_nan() {
                f64::NAN
            } else {
                (u - l) / m
            }
        })
        .collect()
}

fn compute_squeeze_on(
    bb_upper: &[f64],
    bb_lower: &[f64],
    kc_upper: &[f64],
    kc_lower: &[f64],
) -> Vec<f64> {
    bb_lower
        .iter()
        .zip(kc_lower.iter())
        .zip(bb_upper.iter())
        .zip(kc_upper.iter())
        .map(|(((&bl, &kl), &bu), &ku)| {
            if bl.is_nan() || kl.is_nan() || bu.is_nan() || ku.is_nan() {
                f64::NAN
            } else if bl > kl && bu < ku {
                1.0 // True
            } else {
                0.0 // False
            }
        })
        .collect()
}

fn compute_squeeze_count(squeeze_on: &[f64]) -> Vec<f64> {
    let mut out = vec![f64::NAN; squeeze_on.len()];
    let mut count = 0.0f64;
    for i in 0..squeeze_on.len() {
        if squeeze_on[i].is_nan() {
            // Preserve NaN during warmup — don't collapse uncertainty to 0.
            count = 0.0;
            // out[i] stays NaN
        } else if squeeze_on[i] == 1.0 {
            count += 1.0;
            out[i] = count;
        } else {
            count = 0.0;
            out[i] = count;
        }
    }
    out
}

fn compute_mom_slope(close: &[f64]) -> Vec<f64> {
    let n = close.len();
    let mut out = vec![f64::NAN; n];
    if n < SLOPE_N {
        return out;
    }

    // Seed the first full window close[0..SLOPE_N] with naive sums.
    let mut y_sum: f64 = 0.0;
    let mut xy_sum: f64 = 0.0;
    for j in 0..SLOPE_N {
        let y = close[j];
        y_sum += y;
        xy_sum += j as f64 * y;
    }
    out[SLOPE_N - 1] = (SLOPE_N as f64 * xy_sum - SLOPE_XS_SUM * y_sum) / SLOPE_DENOM;

    // Streaming update. Sliding from window [y_old, ..., y_{k-1}] (local j=0..N-1) to
    // [y_old+1, ..., y_{k}] reindexes the dot product: the new window's local j
    // equals the old window's local j+1. Working through the algebra:
    //   new_xy = old_xy - (old_y_sum - y_old) + (N-1) * y_new
    //   new_y_sum = old_y_sum - y_old + y_new
    let last_idx = (SLOPE_N - 1) as f64;
    for i in SLOPE_N..n {
        let y_old = close[i - SLOPE_N];
        let y_new = close[i];
        xy_sum = xy_sum - (y_sum - y_old) + last_idx * y_new;
        y_sum = y_sum - y_old + y_new;
        out[i] = (SLOPE_N as f64 * xy_sum - SLOPE_XS_SUM * y_sum) / SLOPE_DENOM;
    }
    out
}

fn compute_body_ratio(body: &[f64], high: &[f64], low: &[f64]) -> Vec<f64> {
    body.iter()
        .zip(high.iter())
        .zip(low.iter())
        .map(|((&b, &h), &l)| {
            let range = h - l;
            if range == 0.0 {
                f64::NAN
            } else {
                b / range
            }
        })
        .collect()
}

fn compute_plus_dm(high: &[f64], low: &[f64]) -> Vec<f64> {
    let n = high.len();
    let mut out = vec![f64::NAN; n];
    for i in 1..n {
        let diff_high = high[i] - high[i - 1];
        let diff_low = low[i - 1] - low[i];
        out[i] = if diff_high > diff_low && diff_high > 0.0 {
            diff_high
        } else {
            0.0
        };
    }
    out
}

fn compute_minus_dm(high: &[f64], low: &[f64]) -> Vec<f64> {
    let n = high.len();
    let mut out = vec![f64::NAN; n];
    for i in 1..n {
        let diff_high = high[i] - high[i - 1];
        let diff_low = low[i - 1] - low[i];
        out[i] = if diff_low > diff_high && diff_low > 0.0 {
            diff_low
        } else {
            0.0
        };
    }
    out
}

fn compute_di(smoothed_dm: &[f64], smoothed_tr: &[f64]) -> Vec<f64> {
    smoothed_dm
        .iter()
        .zip(smoothed_tr.iter())
        .map(|(&dm, &tr)| {
            if tr == 0.0 || tr.is_nan() || dm.is_nan() {
                f64::NAN
            } else {
                100.0 * dm / tr
            }
        })
        .collect()
}

fn compute_dx(plus_di: &[f64], minus_di: &[f64]) -> Vec<f64> {
    plus_di
        .iter()
        .zip(minus_di.iter())
        .map(|(&p, &m)| {
            if p.is_nan() || m.is_nan() {
                return f64::NAN;
            }
            let sum = p + m;
            if sum == 0.0 {
                return f64::NAN;
            }
            let diff = (p - m).abs();
            let v = 100.0 * diff / sum;
            if v.is_infinite() {
                f64::NAN
            } else {
                v
            }
        })
        .collect()
}

fn compute_t3(e3: &[f64], e4: &[f64], e5: &[f64], e6: &[f64]) -> Vec<f64> {
    e3.iter()
        .zip(e4.iter())
        .zip(e5.iter())
        .zip(e6.iter())
        .map(|(((&v3, &v4), &v5), &v6)| T3_C1 * v6 + T3_C2 * v5 + T3_C3 * v4 + T3_C4 * v3)
        .collect()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Brute-force SMA for reference — O(n*w), used only in tests.
    fn sma_naive(data: &[f64], window: usize) -> Vec<f64> {
        let n = data.len();
        let mut out = vec![f64::NAN; n];
        if n < window || window == 0 {
            return out;
        }
        for i in (window - 1)..n {
            let start = i + 1 - window;
            let mut sum = 0.0f64;
            let mut count = 0usize;
            for j in start..=i {
                if !data[j].is_nan() {
                    sum += data[j];
                    count += 1;
                }
            }
            if count == window {
                out[i] = sum / window as f64;
            }
        }
        out
    }

    /// Brute-force rolling std for reference — O(n*w²), used only in tests.
    fn rolling_std_naive(data: &[f64], window: usize) -> Vec<f64> {
        let n = data.len();
        let mut out = vec![f64::NAN; n];
        if n < window || window < 2 {
            return out;
        }
        for i in (window - 1)..n {
            let start = i + 1 - window;
            let mut sum = 0.0f64;
            let mut count = 0usize;
            for j in start..=i {
                if !data[j].is_nan() {
                    sum += data[j];
                    count += 1;
                }
            }
            if count < window {
                continue;
            }
            let mean = sum / window as f64;
            let var: f64 = (start..=i)
                .filter(|&j| !data[j].is_nan())
                .map(|j| (data[j] - mean) * (data[j] - mean))
                .sum::<f64>()
                / (window as f64 - 1.0);
            out[i] = var.sqrt();
        }
        out
    }

    fn approx_eq(a: f64, b: f64) -> bool {
        if a.is_nan() && b.is_nan() {
            return true;
        }
        if a.is_nan() || b.is_nan() {
            return false;
        }
        let diff = (a - b).abs();
        let scale = a.abs().max(b.abs()).max(1.0);
        diff / scale < 1e-6
    }

    fn assert_vecs_approx_eq(actual: &[f64], expected: &[f64], label: &str) {
        assert_eq!(actual.len(), expected.len(), "{label}: length mismatch");
        for (i, (a, e)) in actual.iter().zip(expected.iter()).enumerate() {
            assert!(
                approx_eq(*a, *e),
                "{label}[{i}]: got {a}, expected {e}, diff={:.2e}",
                (a - e).abs()
            );
        }
    }

    #[test]
    fn test_sma_matches_naive() {
        let data: Vec<f64> = (0..200).map(|i| 1000.0 + (i as f64 * 7.3).sin() * 50.0).collect();
        for window in [3, 14, 20, 72] {
            let fast = sma(&data, window);
            let naive = sma_naive(&data, window);
            assert_vecs_approx_eq(&fast, &naive, &format!("sma(window={window})"));
        }
    }

    #[test]
    fn test_rolling_std_matches_naive() {
        let data: Vec<f64> = (0..200).map(|i| 1000.0 + (i as f64 * 7.3).sin() * 50.0).collect();
        for window in [3, 14, 20, 72] {
            let fast = rolling_std(&data, window);
            let naive = rolling_std_naive(&data, window);
            assert_vecs_approx_eq(&fast, &naive, &format!("rolling_std(window={window})"));
        }
    }

    #[test]
    fn test_sma_with_nan() {
        let mut data: Vec<f64> = (0..100).map(|i| i as f64).collect();
        data[10] = f64::NAN;
        data[50] = f64::NAN;
        let fast = sma(&data, 14);
        let naive = sma_naive(&data, 14);
        assert_vecs_approx_eq(&fast, &naive, "sma_nan");
    }

    #[test]
    fn test_rolling_std_with_nan() {
        let mut data: Vec<f64> = (0..100).map(|i| i as f64).collect();
        data[10] = f64::NAN;
        data[50] = f64::NAN;
        let fast = rolling_std(&data, 14);
        let naive = rolling_std_naive(&data, 14);
        assert_vecs_approx_eq(&fast, &naive, "rolling_std_nan");
    }

    #[test]
    fn test_rolling_std_large_values() {
        // Numerically challenging: large mean, small variance
        let data: Vec<f64> = (0..100).map(|i| 1e8 + i as f64 * 0.01).collect();
        let fast = rolling_std(&data, 20);
        let naive = rolling_std_naive(&data, 20);
        assert_vecs_approx_eq(&fast, &naive, "rolling_std_large_values");
    }

    #[test]
    fn test_sma_short_data() {
        let data = vec![1.0, 2.0, 3.0];
        let result = sma(&data, 5);
        assert!(result.iter().all(|v| v.is_nan()));
    }

    #[test]
    fn test_rolling_std_window_2() {
        let data = vec![1.0, 3.0, 5.0, 7.0];
        let fast = rolling_std(&data, 2);
        let naive = rolling_std_naive(&data, 2);
        assert_vecs_approx_eq(&fast, &naive, "rolling_std_window_2");
    }

    /// Brute-force mom_slope used as reference to test streaming version.
    fn compute_mom_slope_naive(close: &[f64]) -> Vec<f64> {
        let n = close.len();
        let mut out = vec![f64::NAN; n];
        if n < SLOPE_N {
            return out;
        }
        for i in (SLOPE_N - 1)..n {
            let window = &close[i + 1 - SLOPE_N..=i];
            let y_sum: f64 = window.iter().sum();
            let xy_sum: f64 = window.iter().enumerate().map(|(j, &y)| j as f64 * y).sum();
            out[i] = (SLOPE_N as f64 * xy_sum - SLOPE_XS_SUM * y_sum) / SLOPE_DENOM;
        }
        out
    }

    #[test]
    fn test_mom_slope_streaming_matches_naive() {
        let data: Vec<f64> = (0..200)
            .map(|i| 1000.0 + (i as f64 * 7.3).sin() * 50.0 + (i as f64 * 0.3))
            .collect();
        let fast = compute_mom_slope(&data);
        let naive = compute_mom_slope_naive(&data);
        assert_vecs_approx_eq(&fast, &naive, "mom_slope");
    }

    #[test]
    fn test_mom_slope_short_data() {
        let data: Vec<f64> = (0..SLOPE_N - 1).map(|i| i as f64).collect();
        let out = compute_mom_slope(&data);
        assert!(out.iter().all(|v| v.is_nan()));
    }

    #[test]
    fn test_mom_slope_constant_input_is_zero_slope() {
        let data = vec![42.0; 100];
        let out = compute_mom_slope(&data);
        for (i, &v) in out.iter().enumerate() {
            if i < SLOPE_N - 1 {
                assert!(v.is_nan());
            } else {
                assert!(v.abs() < 1e-9, "expected 0 at index {i}, got {v}");
            }
        }
    }

    fn rolling_vwap_naive(typ: &[f64], volume: &[f64], window: usize) -> Vec<f64> {
        let n = typ.len();
        let mut out = vec![f64::NAN; n];
        if window == 0 || n < window {
            return out;
        }
        for i in (window - 1)..n {
            let start = i + 1 - window;
            let mut pv = 0.0f64;
            let mut v = 0.0f64;
            for j in start..=i {
                pv += typ[j] * volume[j];
                v += volume[j];
            }
            if v > 0.0 {
                out[i] = pv / v;
            }
        }
        out
    }

    #[test]
    fn test_rolling_vwap_matches_naive() {
        let n = 200;
        let high: Vec<f64> = (0..n).map(|i| 100.0 + (i as f64 * 0.1).sin() * 3.0 + 0.5).collect();
        let low: Vec<f64> = (0..n).map(|i| 100.0 + (i as f64 * 0.1).sin() * 3.0 - 0.5).collect();
        let close: Vec<f64> = (0..n).map(|i| 100.0 + (i as f64 * 0.1).sin() * 3.0).collect();
        let volume: Vec<f64> = (0..n).map(|i| 1000.0 + (i as f64 * 0.2).cos() * 200.0).collect();
        let typ = typical_price(&high, &low, &close);
        for window in [20, 48] {
            let fast = rolling_vwap(&typ, &volume, window);
            let naive = rolling_vwap_naive(&typ, &volume, window);
            assert_vecs_approx_eq(&fast, &naive, &format!("rolling_vwap(window={window})"));
        }
    }

    #[test]
    fn test_rolling_vwap_constant_price_equals_price() {
        // When price is constant, VWAP should equal the price regardless
        // of volume distribution.
        let n = 60;
        let typ = vec![42.0; n];
        let volume: Vec<f64> = (1..=n).map(|i| i as f64).collect();
        let out = rolling_vwap(&typ, &volume, 20);
        for (i, &v) in out.iter().enumerate() {
            if i < 19 {
                assert!(v.is_nan());
            } else {
                assert!((v - 42.0).abs() < 1e-9);
            }
        }
    }

    #[test]
    fn test_pct_deviation_basic() {
        let a = vec![110.0, 100.0, 95.0, 100.0];
        let b = vec![100.0, 100.0, 100.0, 0.0];
        let out = pct_deviation(&a, &b);
        assert!((out[0] - 10.0).abs() < 1e-9);
        assert!((out[1] - 0.0).abs() < 1e-9);
        assert!((out[2] - (-5.0)).abs() < 1e-9);
        assert!(out[3].is_nan()); // zero denominator
    }

    #[test]
    fn test_rolling_poc_is_within_window_range() {
        let n = 120;
        let high: Vec<f64> = (0..n).map(|i| 100.0 + (i as f64 * 0.1).sin() * 5.0 + 1.0).collect();
        let low: Vec<f64> = (0..n).map(|i| 100.0 + (i as f64 * 0.1).sin() * 5.0 - 1.0).collect();
        let close: Vec<f64> = (0..n).map(|i| 100.0 + (i as f64 * 0.1).sin() * 5.0).collect();
        let volume: Vec<f64> = (0..n).map(|i| 1000.0 + (i as f64 * 0.2).cos() * 200.0).collect();
        let out = rolling_poc(&high, &low, &close, &volume, 48, 50);
        for i in 0..47 {
            assert!(out[i].is_nan());
        }
        for i in 47..n {
            let start = i + 1 - 48;
            let lo = low[start..=i].iter().cloned().fold(f64::INFINITY, f64::min);
            let hi = high[start..=i].iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            assert!(
                out[i] >= lo && out[i] <= hi,
                "poc[{i}] = {} outside [{lo}, {hi}]",
                out[i]
            );
        }
    }

    #[test]
    fn test_rolling_poc_concentrated_volume() {
        // All bars span the same range, but volume is concentrated at a
        // specific close price. POC should lock onto that price.
        let n = 60;
        let high = vec![110.0; n];
        let low = vec![90.0; n];
        let close: Vec<f64> = (0..n).map(|i| if i % 2 == 0 { 100.0 } else { 95.0 }).collect();
        let volume: Vec<f64> = (0..n).map(|i| if i % 2 == 0 { 1000.0 } else { 10.0 }).collect();
        let out = rolling_poc(&high, &low, &close, &volume, 48, 50);
        for i in 47..n {
            // With 50 bins over [90, 110] each bin is 0.4 wide; bin
            // centred on 100 should win. Allow one bin tolerance.
            assert!(
                (out[i] - 100.0).abs() <= 0.5,
                "poc[{i}] = {}, expected ~100",
                out[i]
            );
        }
    }
}
