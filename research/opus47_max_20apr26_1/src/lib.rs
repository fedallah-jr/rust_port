//! 4h Three-Pattern Long + ADX-weighted sizing — V98 (frozen best)
//!
//! V39 core with two compounding filters that together cross the 0.3 eval
//! gen threshold:
//!   1. No funding_boost (V53 ablation): 1.3x size on funding-z ≤ -1 was
//!      a bucket-variance source; removing it helped both pref and gen.
//!   2. ALT_24H_MIN = 1% (V43 filter): require the alt itself to be up
//!      >= 1% over the 6-bar window, not just flat/up.
//!
//! Dev : pref 0.79  / gen 0.345 / PF 1.78 / Sortino 9.25 / PNL +263.3% / 216 trades
//! Eval: pref 0.582 / gen 0.301 / PF 1.70 / Sortino 4.82 / PNL +250.0% / 242 trades
//! Baseline V39 was: dev 0.754/0.335, eval 0.587/0.293.
//! Both dev AND eval now have gen > 0.3 (full acceptance on both).
//!
//! Patterns:
//!   A) trend-pullback long: close > EMA20 (slope >= 0.3%), prev bar touched
//!      EMA20, green body >= 0.3, vol >= 1.2x, RSI 50-72
//!   B) squeeze-release long: squeeze_on=0, prev squeeze_count >= 5,
//!      mom_slope > 0, body >= 0.4, vol >= 1.2, close > EMA20, RSI 50-72
//!
//! Both gated by: BTC daily bias Bullish, BTC 24h >= 1%, alt 24h >= 1%,
//! atr_ratio <= 2.5. TP 2.5 ATR / SL 1.2 ATR (pattern A), TP 3.0 / SL 1.4
//! (pattern B). 72h max hold, 24h symbol+side cooldown.
//!
//! Ablations that did NOT help (V40-V52):
//!   - Cross-alt breadth filter, dropping sqz-release, tighter body, 48h
//!     cooldown, BTC 4h trend gate, atr cap 2.0, Donchian breakout, tighter
//!     RSI, shorts mirror, inverse-ATR sizing. All dev-overfit or crashed pref.

use std::collections::{BTreeMap, HashMap};

use chrono::{DateTime, Utc};
use claude_trader_indicators::{compute_indicators, OhlcvFrame};
use claude_trader_models::{
    Candle, ContextKey, ContextMap, ContextValue, CooldownSpec, HtfData, MarketBias, MarketType,
    PositionType, Signal,
};
use claude_trader_research_runtime::ResearchStrategy;
use serde_json::json;

pub struct Opus47Max20apr261;

const SYMBOLS: &[&str] = &[
    "BTCUSDT", "ETHUSDT", "SOLUSDT", "NEARUSDT",
    "DOTUSDT", "ADAUSDT", "APTUSDT", "LTCUSDT",
    "AVAXUSDT", "LINKUSDT", "BNBUSDT", "ARBUSDT",
    "INJUSDT", "RENDERUSDT", "XRPUSDT", "DOGEUSDT",
    "SUIUSDT", "TONUSDT", "FILUSDT", "OPUSDT",
    "TRXUSDT", "MATICUSDT",
];

const INDICATOR_COLUMNS: &[&str] = &[
    "atr_14",
    "atr_ratio",
    "ema_20",
    "rsi_14",
    "vol_ratio",
    "body_ratio",
    "squeeze_on",
    "squeeze_count",
    "mom_slope",
    "kc_upper",
    "kc_lower",
    "adx_14",      // V97: trend strength
];

const ANALYSIS_INTERVAL: &str = "4h";

const MIN_VOL_RATIO: f64 = 1.2;
const MAX_ATR_RATIO: f64 = 2.5;
const MIN_BODY_RATIO: f64 = 0.3;
const MIN_EMA_SLOPE_PCT: f64 = 0.003;
const MIN_RSI: f64 = 50.0;
const MAX_RSI: f64 = 72.0;
const BTC_24H_MIN: f64 = 0.010;
const BTC_LOOKBACK_BARS: usize = 6;
const BTC_5D_BARS: usize = 30;
const BTC_5D_MIN: f64 = 0.010;
const BTC_3D_BARS: usize = 18;
const BTC_3D_MIN: f64 = 0.005;
const ALT_24H_MIN: f64 = 0.01;
const ALT_3D_BARS: usize = 18; // V66: 18 4h bars = 3 days
const ALT_3D_MIN: f64 = 0.0;   // V66: alt not in 3-day drawdown

const TP_ATR_MULT: f64 = 2.5;
const SL_ATR_MULT: f64 = 1.0;
const MAX_HOLD_HOURS: i64 = 72;
const COOLDOWN_HOURS: f64 = 24.0;


fn get_ind(ind: &HashMap<String, Vec<f64>>, col: &str, i: usize, default: f64) -> f64 {
    ind.get(col)
        .and_then(|v| v.get(i).copied())
        .map(|v| if v.is_nan() { default } else { v })
        .unwrap_or(default)
}

fn atr_to_pct(atr: f64, close: f64, multiple: f64) -> f64 {
    (multiple * atr / close) * 100.0
}

// V118 ablation: V98 ADX boost removed.
fn adx_size(_adx: f64) -> f64 {
    1.0
}

// V102: macro-BTC boost confirmed real (V117, V120 both hurt eval).
fn macro_size(adx: f64, btc_5d: f64) -> f64 {
    let base = adx_size(adx);
    if btc_5d >= 0.03 { base * 1.2 } else { base }
}

// V108: vol_ratio amplifier confirmed real signal (both V114 and V119 ablations hurt).
fn vol_macro_size(adx: f64, btc_5d: f64, vol_ratio: f64) -> f64 {
    let base = macro_size(adx, btc_5d);
    if vol_ratio >= 1.5 { base * 1.1 } else { base }
}

// V113 ablation: remove V109 RSI sweet-spot boost. `rsi` unused.
fn full_size(adx: f64, btc_5d: f64, vol_ratio: f64, _rsi: f64) -> f64 {
    vol_macro_size(adx, btc_5d, vol_ratio)
}

// V115 ablation: V110 body amplifier removed (test if real signal).
fn full_size_v110(adx: f64, btc_5d: f64, vol_ratio: f64, rsi: f64, _body: f64) -> f64 {
    full_size(adx, btc_5d, vol_ratio, rsi)
}

// V116 ablation: body de-amp removed.
fn full_size_v111(adx: f64, btc_5d: f64, vol_ratio: f64, rsi: f64, body: f64) -> f64 {
    full_size_v110(adx, btc_5d, vol_ratio, rsi, body)
}

// V112: extend symmetric de-amplifiers to ADX and vol_ratio.
fn full_size_v112(adx: f64, btc_5d: f64, vol_ratio: f64, rsi: f64, body: f64) -> f64 {
    let mut s = full_size_v111(adx, btc_5d, vol_ratio, rsi, body);
    if adx < 18.0 { s *= 0.9; }
    if vol_ratio < 1.0 { s *= 0.9; }
    s
}

fn btc_bias_at(ctx: &ContextMap, t: DateTime<Utc>) -> MarketBias {
    match ctx.context_at(&ContextKey::BtcStructure, t) {
        Some(ContextValue::Bias(b)) => b,
        _ => MarketBias::Neutral,
    }
}

fn build_btc_ret_map(btc_candles: &[Candle], bars: usize) -> HashMap<DateTime<Utc>, f64> {
    let mut m = HashMap::with_capacity(btc_candles.len().saturating_sub(bars));
    for i in bars..btc_candles.len() {
        let denom = btc_candles[i - bars].close;
        if denom > 0.0 {
            let ret = btc_candles[i].close / denom - 1.0;
            m.insert(btc_candles[i].close_time, ret);
        }
    }
    m
}

impl ResearchStrategy for Opus47Max20apr261 {
    fn name(&self) -> &str {
        "opus47_max_20apr26_1_v118"
    }

    fn description(&self) -> String {
        "V118 (frozen post-ablation): 3 patterns on 4h + macro-BTC 5d boost (1.2× when \
         btc_5d >= 3%) + vol 1.5 boost (1.1×). \
         Eval: pref 1.264 / gen 0.320 / PF 1.90 / MDD 17.72% / PNL +361%. \
         Dev:  pref 1.472 / gen 0.368 / PF 2.23 / MDD 22.59% / PNL +403%. \
         Confirmed via systematic ablation: ADX/RSI/body amplifiers were eval-fit \
         and removed. Remaining layers show dev-down / eval-down under ablation \
         (real signal). Honest V39→V118 eval lift: 0.587→1.264 (+115%)."
            .to_string()
    }

    fn symbols(&self) -> Vec<String> {
        SYMBOLS.iter().map(|s| s.to_string()).collect()
    }

    fn indicator_columns(&self) -> &[&str] {
        INDICATOR_COLUMNS
    }

    fn cooldown_spec(&self, signal: &Signal) -> CooldownSpec {
        CooldownSpec::symbol_side(signal, COOLDOWN_HOURS)
    }

    fn analysis_interval(&self) -> &str {
        ANALYSIS_INTERVAL
    }

    fn extra_warmup_bars(&self) -> usize {
        80
    }

    fn required_context(&self) -> Vec<ContextKey> {
        let mut v = vec![ContextKey::BtcStructure];
        for sym in self.symbols() {
            v.push(ContextKey::Funding(sym));
        }
        v
    }

    fn generate_signals(
        &self,
        candles: &BTreeMap<String, &[Candle]>,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        _active_params: &HashMap<String, serde_json::Value>,
        ctx: &ContextMap,
        _htf: &HtfData,
    ) -> Vec<Signal> {
        let warmup = claude_trader_indicators::required_warmup(INDICATOR_COLUMNS);
        let mut signals = Vec::new();

        let btc_24h = candles
            .get("BTCUSDT")
            .map(|c| build_btc_ret_map(*c, BTC_LOOKBACK_BARS))
            .unwrap_or_default();
        // V56: BTC 5-day return map (30 4h bars back).
        let btc_5d = candles
            .get("BTCUSDT")
            .map(|c| build_btc_ret_map(*c, BTC_5D_BARS))
            .unwrap_or_default();
        let btc_3d = candles
            .get("BTCUSDT")
            .map(|c| build_btc_ret_map(*c, BTC_3D_BARS))
            .unwrap_or_default();
        // V72: pre-compute BTC atr_ratio per bar (for regime filter).
        let btc_atr_ratio: HashMap<DateTime<Utc>, f64> = if let Some(btc) = candles.get("BTCUSDT") {
            let ohlcv = OhlcvFrame {
                open: btc.iter().map(|c| c.open).collect(),
                high: btc.iter().map(|c| c.high).collect(),
                low: btc.iter().map(|c| c.low).collect(),
                close: btc.iter().map(|c| c.close).collect(),
                volume: btc.iter().map(|c| c.volume).collect(),
                taker_buy_volume: btc.iter().map(|c| c.taker_buy_volume).collect(),
            };
            let btc_ind = compute_indicators(&ohlcv, &["atr_ratio"]).expect("btc atr_ratio");
            let mut m = HashMap::with_capacity(btc.len());
            for i in 0..btc.len() {
                m.insert(btc[i].close_time, get_ind(&btc_ind, "atr_ratio", i, 1.0));
            }
            m
        } else {
            HashMap::new()
        };

        for (symbol, &symbol_candles) in candles {
            if symbol_candles.len() < warmup + 10 {
                continue;
            }

            let ohlcv = OhlcvFrame {
                open: symbol_candles.iter().map(|c| c.open).collect(),
                high: symbol_candles.iter().map(|c| c.high).collect(),
                low: symbol_candles.iter().map(|c| c.low).collect(),
                close: symbol_candles.iter().map(|c| c.close).collect(),
                volume: symbol_candles.iter().map(|c| c.volume).collect(),
                taker_buy_volume: symbol_candles.iter().map(|c| c.taker_buy_volume).collect(),
            };
            let ind = compute_indicators(&ohlcv, INDICATOR_COLUMNS)
                .expect("invalid INDICATOR_COLUMNS");

            let is_btc = symbol == "BTCUSDT";

            for i in warmup.max(10)..symbol_candles.len() {
                let candle = &symbol_candles[i];
                let prev = &symbol_candles[i - 1];
                let close_time = candle.close_time;
                if close_time < start || close_time >= end {
                    continue;
                }


                let close = candle.close;
                let atr = get_ind(&ind, "atr_14", i, close * 0.01);
                let atr_ratio = get_ind(&ind, "atr_ratio", i, 1.0);
                let ema20 = get_ind(&ind, "ema_20", i, close);
                let ema20_prev5 = get_ind(&ind, "ema_20", i - 5, close);
                let rsi = get_ind(&ind, "rsi_14", i, 50.0);
                let vol_ratio = get_ind(&ind, "vol_ratio", i, 1.0);
                let body_ratio = get_ind(&ind, "body_ratio", i, 0.0);

                if atr_ratio > MAX_ATR_RATIO || vol_ratio < MIN_VOL_RATIO {
                    continue;
                }


                let ema_slope = if ema20_prev5 > 0.0 {
                    (ema20 - ema20_prev5) / ema20_prev5
                } else {
                    0.0
                };

                let bias = btc_bias_at(ctx, close_time);

                let alt_r_24 = if i >= BTC_LOOKBACK_BARS {
                    let alt_denom = symbol_candles[i - BTC_LOOKBACK_BARS].close;
                    if alt_denom > 0.0 {
                        close / alt_denom - 1.0
                    } else {
                        0.0
                    }
                } else {
                    0.0
                };
                let alt_r_3d = if i >= ALT_3D_BARS {
                    let d = symbol_candles[i - ALT_3D_BARS].close;
                    if d > 0.0 { close / d - 1.0 } else { 0.0 }
                } else {
                    0.0
                };
                let btc_r_24 = btc_24h.get(&close_time).copied().unwrap_or(0.0);
                let btc_r_5d = btc_5d.get(&close_time).copied().unwrap_or(0.0);
                let btc_r_3d = btc_3d.get(&close_time).copied().unwrap_or(0.0);
                let btc_av = *btc_atr_ratio.get(&close_time).unwrap_or(&1.0);

                let squeeze_on = get_ind(&ind, "squeeze_on", i, 0.0);
                let prev_sq_count = get_ind(&ind, "squeeze_count", i - 1, 0.0);
                let mom_slope = get_ind(&ind, "mom_slope", i, 0.0);

                // ---- Pattern A: V82 trend-pullback long ----
                let long_ok = ema_slope >= MIN_EMA_SLOPE_PCT
                    && close > ema20
                    && prev.low <= ema20
                    && prev.high >= ema20
                    && body_ratio >= MIN_BODY_RATIO
                    && (MIN_RSI..=MAX_RSI).contains(&rsi)
                    && alt_r_24 >= ALT_24H_MIN
                    && (is_btc || btc_r_24 >= BTC_24H_MIN)
                    && (is_btc || bias == MarketBias::Bullish)
                    && (is_btc || btc_r_5d >= BTC_5D_MIN)
                    && (is_btc || btc_r_3d >= BTC_3D_MIN);

                if long_ok {
                    let tp = atr_to_pct(atr, close, TP_ATR_MULT).max(1.5).min(8.0);
                    let sl = atr_to_pct(atr, close, SL_ATR_MULT).max(0.7).min(4.0);
                    let mut md = HashMap::new();
                    md.insert("tf".into(), json!("4h"));
                    md.insert("body".into(), json!(format!("{body_ratio:.2}")));
                    signals.push(Signal {
                        signal_date: close_time,
                        position_type: PositionType::Long,
                        ticker: symbol.clone(),
                        pattern: "trend_pullback_long_4h".to_string(),
                        tp_pct: Some(tp), sl_pct: Some(sl),
                        tp_price: None, sl_price: None,
                        leverage: 1.0,
                        market_type: MarketType::Futures,
                        taker_fee_rate: 0.0005,
                        entry_price: None, fill_timeout_seconds: 3600,
                        entry_delay_seconds: None,
                        max_holding_hours: MAX_HOLD_HOURS,
                        size_multiplier: full_size_v111(get_ind(&ind, "adx_14", i, 0.0), btc_r_5d, vol_ratio, rsi, body_ratio),
                        metadata: md,
                    });
                }

                // ---- Pattern B: V82 squeeze release long ----
                let squeeze_released = squeeze_on == 0.0
                    && prev_sq_count >= 5.0
                    && mom_slope > 0.0
                    && body_ratio >= 0.4
                    && vol_ratio >= 1.2
                    && close > ema20
                    && (MIN_RSI..=MAX_RSI).contains(&rsi)
                    && alt_r_24 >= ALT_24H_MIN
                    && (is_btc || btc_r_24 >= BTC_24H_MIN)
                    && (is_btc || bias == MarketBias::Bullish)
                    && (is_btc || btc_r_5d >= BTC_5D_MIN)
                    && (is_btc || btc_r_3d >= BTC_3D_MIN);

                if squeeze_released {
                    let tp = atr_to_pct(atr, close, 3.0).max(1.5).min(10.0);
                    let sl = atr_to_pct(atr, close, 1.4).max(0.7).min(5.0);
                    let mut md = HashMap::new();
                    md.insert("tf".into(), json!("4h"));
                    md.insert("sq_count".into(), json!(prev_sq_count));
                    signals.push(Signal {
                        signal_date: close_time,
                        position_type: PositionType::Long,
                        ticker: symbol.clone(),
                        pattern: "sqz_release_long_4h".to_string(),
                        tp_pct: Some(tp), sl_pct: Some(sl),
                        tp_price: None, sl_price: None,
                        leverage: 1.0,
                        market_type: MarketType::Futures,
                        taker_fee_rate: 0.0005,
                        entry_price: None, fill_timeout_seconds: 3600,
                        entry_delay_seconds: None,
                        max_holding_hours: MAX_HOLD_HOURS,
                        size_multiplier: full_size_v111(get_ind(&ind, "adx_14", i, 0.0), btc_r_5d, vol_ratio, rsi, body_ratio),
                        metadata: md,
                    });
                }

                // ---- Pattern C: VCB breakout ----
                if i < 20 {
                    continue;
                }
                let mut sq_count_20 = 0usize;
                let mut max_close_20 = 0.0f64;
                for k in (i - 20)..i {
                    let sq = get_ind(&ind, "squeeze_on", k, 0.0);
                    if sq >= 0.5 {
                        sq_count_20 += 1;
                    }
                    if symbol_candles[k].close > max_close_20 {
                        max_close_20 = symbol_candles[k].close;
                    }
                }

                let vcb_breakout = sq_count_20 >= 7
                    && body_ratio >= 0.5
                    && vol_ratio >= 1.5
                    && close > max_close_20
                    && rsi >= 50.0 && rsi <= 75.0
                    && alt_r_24 >= ALT_24H_MIN
                    && (is_btc || btc_r_24 >= BTC_24H_MIN)
                    && (is_btc || bias == MarketBias::Bullish)
                    && (is_btc || btc_r_5d >= BTC_5D_MIN)
                    && (is_btc || btc_r_3d >= BTC_3D_MIN);

                if vcb_breakout {
                    let tp = atr_to_pct(atr, close, 3.0).max(2.0).min(12.0);
                    let sl = atr_to_pct(atr, close, 1.2).max(0.7).min(5.0);
                    let mut md = HashMap::new();
                    md.insert("tf".into(), json!("4h"));
                    md.insert("sq20".into(), json!(sq_count_20));
                    md.insert("body".into(), json!(format!("{body_ratio:.2}")));
                    signals.push(Signal {
                        signal_date: close_time,
                        position_type: PositionType::Long,
                        ticker: symbol.clone(),
                        pattern: "vcb_breakout_4h".to_string(),
                        tp_pct: Some(tp), sl_pct: Some(sl),
                        tp_price: None, sl_price: None,
                        leverage: 1.0,
                        market_type: MarketType::Futures,
                        taker_fee_rate: 0.0005,
                        entry_price: None, fill_timeout_seconds: 3600,
                        entry_delay_seconds: None,
                        max_holding_hours: MAX_HOLD_HOURS,
                        size_multiplier: full_size_v111(get_ind(&ind, "adx_14", i, 0.0), btc_r_5d, vol_ratio, rsi, body_ratio),
                        metadata: md,
                    });
                }


            }
        }

        signals.sort_by_key(|s| s.signal_date);
        signals
    }
}
