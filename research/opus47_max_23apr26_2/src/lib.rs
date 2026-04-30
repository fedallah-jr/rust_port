//! HTF-Confirmed Donchian Trend Breakout (HTB) — v18 frozen final.
//!
//! 36-version journey (this file is v18, the dev-best configuration).
//! Journey summary belongs in results.tsv; this module stays focused
//! on the actual implementation.

use std::collections::{BTreeMap, HashMap};

use chrono::{DateTime, Timelike, Utc};
use claude_trader_indicators::{compute_indicators, OhlcvFrame};
use claude_trader_models::{
    Candle, ContextKey, ContextMap, ContextValue, CooldownSpec, HtfData, MarketBias, MarketType,
    PositionType, Signal,
};
use claude_trader_research_runtime::ResearchStrategy;
use claude_trader_strategy_blocks::breakout::{donchian_high_break, donchian_low_break};

pub struct Opus47Max23apr262;

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

const SYMBOLS: &[&str] = &[
    "BTCUSDT", "ETHUSDT", "SOLUSDT", "BNBUSDT", "XRPUSDT", "ADAUSDT", "AVAXUSDT", "DOGEUSDT",
    "LINKUSDT", "LTCUSDT",
];

const INDICATOR_COLUMNS: &[&str] = &[
    "atr_14",
    "rsi_14",
    "vol_ratio",
    "body_ratio",
    "adx_14",
];

const ANALYSIS_INTERVAL: &str = "1h";
const HTF_INTERVAL: &str = "4h";

const DONCHIAN_WINDOW: usize = 20;
const DONCHIAN_MIN_MOVE_PCT: f64 = 0.004;

const VOL_RATIO_MIN: f64 = 1.2;
const BODY_ABS_MIN: f64 = 0.35;
const ADX_MIN: f64 = 25.0;
const RSI_LONG_MAX: f64 = 78.0;
const RSI_SHORT_MIN: f64 = 22.0;

const HTF_SLOPE_BARS: usize = 3;

const TP_ATR_MULT: f64 = 3.0;
const SL_ATR_MULT: f64 = 1.5;
const MIN_TP_PCT: f64 = 1.5;
const MAX_TP_PCT: f64 = 8.0;
const MIN_SL_PCT: f64 = 0.7;
const MAX_SL_PCT: f64 = 3.5;

const MAX_HOLDING_HOURS: i64 = 72;
const COOLDOWN_HOURS: f64 = 24.0;

const ENTRY_HOUR_MIN_UTC: u32 = 8;
const ENTRY_HOUR_MAX_UTC: u32 = 18;

// ---------------------------------------------------------------------------
// Strategy implementation
// ---------------------------------------------------------------------------

impl ResearchStrategy for Opus47Max23apr262 {
    fn name(&self) -> &str {
        "opus47_max_23apr26_2"
    }

    fn description(&self) -> String {
        "HTB v18 (frozen final after 36 versions): 1h Donchian-20 \
         breakout ≥0.4%, 4h EMA-20 sustained trend (above + rising \
         3 HTF bars), BTC not bearish, 08-18 UTC. Body ≥0.35, \
         vol_ratio ≥1.2, ADX ≥25, RSI ≤78/≥22. Symmetric short. \
         TP/SL = 3×/1.5× ATR clamped [1.5,8]/[0.7,3.5]%. 72h hold, \
         24h cooldown, 10 majors. Dev Pref 0.282 Gen 0.292 PnL +168%; \
         Eval FAILED (Gen 0.126)."
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

    fn additional_intervals(&self) -> Vec<&str> {
        vec![HTF_INTERVAL]
    }

    fn indicator_columns_per_interval(&self) -> HashMap<&str, Vec<&str>> {
        let mut m = HashMap::new();
        m.insert(HTF_INTERVAL, vec!["ema_20"]);
        m
    }

    fn required_context(&self) -> Vec<ContextKey> {
        vec![ContextKey::BtcStructure]
    }

    fn generate_signals(
        &self,
        candles: &BTreeMap<String, &[Candle]>,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        _active_params: &HashMap<String, serde_json::Value>,
        ctx: &ContextMap,
        htf: &HtfData,
    ) -> Vec<Signal> {
        let warmup = claude_trader_indicators::required_warmup(INDICATOR_COLUMNS)
            .max(DONCHIAN_WINDOW + 2);
        let mut signals = Vec::new();

        for (symbol, &symbol_candles) in candles {
            if symbol_candles.len() < warmup + 3 {
                continue;
            }

            let highs: Vec<f64> = symbol_candles.iter().map(|c| c.high).collect();
            let lows: Vec<f64> = symbol_candles.iter().map(|c| c.low).collect();

            let ohlcv = OhlcvFrame {
                open: symbol_candles.iter().map(|c| c.open).collect(),
                high: highs.clone(),
                low: lows.clone(),
                close: symbol_candles.iter().map(|c| c.close).collect(),
                volume: symbol_candles.iter().map(|c| c.volume).collect(),
                taker_buy_volume: symbol_candles.iter().map(|c| c.taker_buy_volume).collect(),
            };
            let ind = compute_indicators(&ohlcv, INDICATOR_COLUMNS)
                .expect("invalid INDICATOR_COLUMNS");

            let get = |col: &str, i: usize| -> f64 {
                ind.get(col).and_then(|v| v.get(i).copied()).unwrap_or(f64::NAN)
            };

            let htf_candles: &[Candle] = htf
                .additional_candles
                .get(HTF_INTERVAL)
                .and_then(|m| m.get(symbol))
                .map(|v| v.as_slice())
                .unwrap_or(&[]);
            let htf_ema: Option<&Vec<f64>> = htf
                .additional_indicators
                .get(HTF_INTERVAL)
                .and_then(|m| m.get(symbol))
                .and_then(|inds| inds.get("ema_20"));

            let start_idx = warmup.max(3);
            for i in start_idx..symbol_candles.len() {
                let close_time = symbol_candles[i].close_time;
                if close_time < start || close_time >= end {
                    continue;
                }

                let hour = close_time.hour();
                if hour < ENTRY_HOUR_MIN_UTC || hour >= ENTRY_HOUR_MAX_UTC {
                    continue;
                }

                let close = symbol_candles[i].close;
                if !close.is_finite() || close <= 0.0 {
                    continue;
                }

                let htf_dir = htf_trend_at(close_time, htf_candles, htf_ema);
                if htf_dir == 0 {
                    continue;
                }
                let btc_bias = match ctx.context_at(&ContextKey::BtcStructure, close_time) {
                    Some(ContextValue::Bias(b)) => b,
                    _ => MarketBias::Neutral,
                };

                let body_ratio = get("body_ratio", i);
                if !body_ratio.is_finite() {
                    continue;
                }
                let vr = get("vol_ratio", i);
                if !vr.is_finite() || vr < VOL_RATIO_MIN {
                    continue;
                }
                let adx = get("adx_14", i);
                if !adx.is_finite() || adx < ADX_MIN {
                    continue;
                }
                let rsi = get("rsi_14", i);
                if !rsi.is_finite() {
                    continue;
                }

                let long_break = donchian_high_break(
                    &highs,
                    close,
                    i,
                    DONCHIAN_WINDOW,
                    DONCHIAN_MIN_MOVE_PCT,
                );
                let short_break = donchian_low_break(
                    &lows,
                    close,
                    i,
                    DONCHIAN_WINDOW,
                    DONCHIAN_MIN_MOVE_PCT,
                );

                let side: Option<PositionType> = match (long_break, short_break) {
                    (Some(_), None)
                        if htf_dir > 0 && btc_bias != MarketBias::Bearish =>
                    {
                        if body_ratio >= BODY_ABS_MIN && rsi <= RSI_LONG_MAX {
                            Some(PositionType::Long)
                        } else {
                            None
                        }
                    }
                    (None, Some(_))
                        if htf_dir < 0 && btc_bias != MarketBias::Bullish =>
                    {
                        if body_ratio <= -BODY_ABS_MIN && rsi >= RSI_SHORT_MIN {
                            Some(PositionType::Short)
                        } else {
                            None
                        }
                    }
                    _ => None,
                };

                let Some(side) = side else { continue };

                let atr = get("atr_14", i);
                if !atr.is_finite() || atr <= 0.0 {
                    continue;
                }
                let atr_pct = (atr / close) * 100.0;
                let tp_pct = (atr_pct * TP_ATR_MULT).clamp(MIN_TP_PCT, MAX_TP_PCT);
                let sl_pct = (atr_pct * SL_ATR_MULT).clamp(MIN_SL_PCT, MAX_SL_PCT);

                let pattern = match side {
                    PositionType::Long => "htb_donchian_long",
                    PositionType::Short => "htb_donchian_short",
                };

                let mut metadata = HashMap::new();
                metadata.insert("atr_pct".into(), serde_json::json!(format!("{atr_pct:.3}")));
                metadata.insert("adx".into(), serde_json::json!(format!("{adx:.1}")));
                metadata.insert("vr".into(), serde_json::json!(format!("{vr:.2}")));
                metadata.insert("body".into(), serde_json::json!(format!("{body_ratio:.2}")));
                metadata.insert("htf_dir".into(), serde_json::json!(htf_dir));

                signals.push(Signal {
                    signal_date: close_time,
                    position_type: side,
                    ticker: symbol.clone(),
                    pattern: pattern.to_string(),
                    tp_pct: Some(tp_pct),
                    sl_pct: Some(sl_pct),
                    tp_price: None,
                    sl_price: None,
                    leverage: 1.0,
                    market_type: MarketType::Futures,
                    taker_fee_rate: 0.0005,
                    entry_price: None,
                    fill_timeout_seconds: 3600,
                    entry_delay_seconds: None,
                    max_holding_hours: MAX_HOLDING_HOURS,
                    size_multiplier: 1.0,
                    metadata,
                });
            }
        }

        signals.sort_by_key(|s| s.signal_date);
        signals
    }
}

/// HTF trend: +1 when most recent 4h close > 4h ema_20 AND ema_20
/// rising over last `HTF_SLOPE_BARS`; -1 mirrored; 0 otherwise.
fn htf_trend_at(
    t: DateTime<Utc>,
    htf_candles: &[Candle],
    htf_ema: Option<&Vec<f64>>,
) -> i32 {
    if htf_candles.is_empty() {
        return 0;
    }
    let Some(ema_vec) = htf_ema else { return 0 };

    let idx = match htf_candles
        .partition_point(|c| c.close_time <= t)
        .checked_sub(1)
    {
        Some(i) => i,
        None => return 0,
    };
    if idx < HTF_SLOPE_BARS {
        return 0;
    }
    let c = &htf_candles[idx];
    let ema_now = match ema_vec.get(idx).copied() {
        Some(v) if v.is_finite() => v,
        _ => return 0,
    };
    let ema_prev = match ema_vec.get(idx - HTF_SLOPE_BARS).copied() {
        Some(v) if v.is_finite() => v,
        _ => return 0,
    };

    let above = c.close > ema_now;
    let below = c.close < ema_now;
    let rising = ema_now > ema_prev;
    let falling = ema_now < ema_prev;
    if above && rising {
        1
    } else if below && falling {
        -1
    } else {
        0
    }
}
