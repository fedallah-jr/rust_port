//! 4h accepted-value flow-thrust research branch.
//!
//! Novelty relative to the in-tree strategies:
//! - not EMA pullback
//! - not squeeze release
//! - not inside-bar expansion
//! - not divergence
//! - not support/resistance pierce reclaim
//! - not a plain Donchian / squeeze breakout
//!
//! Entry family:
//! - require price to already be accepted above or below structural value
//! - require the prior bar to be a contained pause inside that accepted state
//! - enter on a fresh CVD-backed thrust through the recent 3-bar range

use std::collections::{BTreeMap, HashMap};

use chrono::{DateTime, Utc};
use claude_trader_indicators::{compute_indicators, OhlcvFrame};
use claude_trader_models::{
    Candle, ContextKey, ContextMap, ContextValue, CooldownSpec, FundingContext, HtfData, KeyLevels,
    MarketBias, MarketType, PositionType, Signal,
};
use claude_trader_research_runtime::ResearchStrategy;
use serde_json::json;

pub struct Gpt54Mid22apr261;

const SYMBOLS: &[&str] = &[
    "BTCUSDT",
    "ETHUSDT",
    "SOLUSDT",
    "BNBUSDT",
    "XRPUSDT",
    "DOGEUSDT",
    "ADAUSDT",
    "LINKUSDT",
    "AVAXUSDT",
    "LTCUSDT",
    "TRXUSDT",
    "TONUSDT",
    "NEARUSDT",
    "DOTUSDT",
    "APTUSDT",
    "ARBUSDT",
    "INJUSDT",
    "RENDERUSDT",
    "SUIUSDT",
    "FILUSDT",
    "OPUSDT",
    "MATICUSDT",
];

const INDICATOR_COLUMNS: &[&str] = &[
    "atr_14",
    "atr_ratio",
    "vol_ratio",
    "body_ratio",
    "vwap_20",
    "poc_48",
    "t3",
    "cvd",
];

const ANALYSIS_INTERVAL: &str = "4h";
const EXTRA_WARMUP_BARS: usize = 120;
const COOLDOWN_HOURS: f64 = 24.0;

const BTC_24H_BARS: usize = 6;
const ALT_24H_BARS: usize = 6;
const RANGE_LOOKBACK: usize = 3;
const CVD_LOOKBACK: usize = 4;

const MAX_ATR_RATIO: f64 = 2.8;
const MIN_THRUST_VOL_RATIO: f64 = 1.20;
const MIN_THRUST_BODY_RATIO: f64 = 0.55;
const MAX_SETUP_BODY_RATIO: f64 = 0.35;

const BTC_24H_LONG_MIN: f64 = 0.0;
const BTC_24H_SHORT_MAX: f64 = -0.01;
const ALT_24H_LONG_MIN: f64 = -0.02;
const ALT_24H_LONG_MAX: f64 = 0.12;
const ALT_24H_SHORT_MIN: f64 = -0.12;
const ALT_24H_SHORT_MAX: f64 = -0.02;

const MAX_LONG_EUPHORIA_Z: f64 = 1.0;
const SHORT_CROWD_Z: f64 = -0.5;
const LONG_CROWD_Z: f64 = 0.5;

const TP_ATR_MULT: f64 = 2.4;
const SL_ATR_MULT: f64 = 1.1;
const MAX_HOLD_HOURS: i64 = 72;
const BASE_SIZE: f64 = 1.0;
const CROWD_BOOST_SIZE: f64 = 1.2;

fn get_ind(ind: &HashMap<String, Vec<f64>>, col: &str, i: usize, default: f64) -> f64 {
    ind.get(col)
        .and_then(|v| v.get(i).copied())
        .map(|v| if v.is_nan() { default } else { v })
        .unwrap_or(default)
}

fn atr_to_pct(atr: f64, close: f64, multiple: f64) -> f64 {
    (multiple * atr / close) * 100.0
}

fn bar_return(candles: &[Candle], i: usize, bars: usize) -> f64 {
    if i < bars {
        return 0.0;
    }
    let prior = candles[i - bars].close;
    if prior > 0.0 {
        candles[i].close / prior - 1.0
    } else {
        0.0
    }
}

fn btc_bias_at(ctx: &ContextMap, t: DateTime<Utc>) -> MarketBias {
    match ctx.context_at(&ContextKey::BtcStructure, t) {
        Some(ContextValue::Bias(b)) => b,
        _ => MarketBias::Neutral,
    }
}

fn funding_at(ctx: &ContextMap, symbol: &str, t: DateTime<Utc>) -> Option<FundingContext> {
    match ctx.context_at(&ContextKey::Funding(symbol.to_string()), t) {
        Some(ContextValue::Funding(f)) => Some(f),
        _ => None,
    }
}

fn key_levels_at(ctx: &ContextMap, symbol: &str, t: DateTime<Utc>) -> Option<KeyLevels> {
    match ctx.context_at(&ContextKey::KeyLevels(symbol.to_string()), t) {
        Some(ContextValue::KeyLevels(levels)) => Some(levels),
        _ => None,
    }
}

fn crowded_shorts(funding: FundingContext) -> bool {
    funding.rate <= 0.0 && funding.cumulative_7d <= 0.0 && funding.zscore_30d <= SHORT_CROWD_Z
}

fn crowded_longs(funding: FundingContext) -> bool {
    funding.rate >= 0.0 && funding.cumulative_7d >= 0.0 && funding.zscore_30d >= LONG_CROWD_Z
}

fn recent_max_high(candles: &[Candle], i: usize, lookback: usize) -> f64 {
    let start = i - lookback;
    candles[start..i]
        .iter()
        .map(|c| c.high)
        .fold(f64::NEG_INFINITY, f64::max)
}

fn recent_min_low(candles: &[Candle], i: usize, lookback: usize) -> f64 {
    let start = i - lookback;
    candles[start..i]
        .iter()
        .map(|c| c.low)
        .fold(f64::INFINITY, f64::min)
}

fn recent_max_cvd(ind: &HashMap<String, Vec<f64>>, i: usize, lookback: usize) -> f64 {
    let start = i - lookback;
    (start..i)
        .map(|idx| get_ind(ind, "cvd", idx, 0.0))
        .fold(f64::NEG_INFINITY, f64::max)
}

fn recent_min_cvd(ind: &HashMap<String, Vec<f64>>, i: usize, lookback: usize) -> f64 {
    let start = i - lookback;
    (start..i)
        .map(|idx| get_ind(ind, "cvd", idx, 0.0))
        .fold(f64::INFINITY, f64::min)
}

impl ResearchStrategy for Gpt54Mid22apr261 {
    fn name(&self) -> &str {
        "gpt54_mid_22apr26_1_v66"
    }

    fn description(&self) -> String {
        "V66: accepted-value flow thrust on 4h, alt-only. Keep BTC as macro-only and keep the \
         accepted-value thrust entry, but cap long-side funding at a moderate level so we retain \
         more coverage than the stricter euphoria cap while still filtering the most crowded \
         late-trend breakouts."
            .to_string()
    }

    fn symbols(&self) -> Vec<String> {
        SYMBOLS.iter().map(|s| s.to_string()).collect()
    }

    fn indicator_columns(&self) -> &[&str] {
        INDICATOR_COLUMNS
    }

    fn cooldown_spec(&self, signal: &Signal) -> CooldownSpec {
        CooldownSpec::symbol_pattern(signal, COOLDOWN_HOURS)
    }

    fn analysis_interval(&self) -> &str {
        ANALYSIS_INTERVAL
    }

    fn extra_warmup_bars(&self) -> usize {
        EXTRA_WARMUP_BARS
    }

    fn required_context(&self) -> Vec<ContextKey> {
        let mut keys = vec![ContextKey::BtcStructure];
        for symbol in self.symbols() {
            keys.push(ContextKey::Funding(symbol.clone()));
            keys.push(ContextKey::KeyLevels(symbol));
        }
        keys
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

        let btc_candles = match candles.get("BTCUSDT") {
            Some(series) if series.len() >= warmup + BTC_24H_BARS + RANGE_LOOKBACK + 1 => *series,
            _ => return signals,
        };

        let mut btc_ret_24h = HashMap::with_capacity(btc_candles.len());
        for i in BTC_24H_BARS..btc_candles.len() {
            btc_ret_24h.insert(
                btc_candles[i].close_time,
                bar_return(btc_candles, i, BTC_24H_BARS),
            );
        }

        for (symbol, &symbol_candles) in candles {
            if symbol_candles.len() < warmup + ALT_24H_BARS + RANGE_LOOKBACK + 1 {
                continue;
            }
            if symbol == "BTCUSDT" {
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
            let ind =
                compute_indicators(&ohlcv, INDICATOR_COLUMNS).expect("invalid INDICATOR_COLUMNS");

            for i in
                warmup.max(ALT_24H_BARS + RANGE_LOOKBACK).max(CVD_LOOKBACK)..symbol_candles.len()
            {
                let candle = &symbol_candles[i];
                let prev = &symbol_candles[i - 1];
                let close_time = candle.close_time;
                if close_time < start || close_time >= end {
                    continue;
                }

                let funding = match funding_at(ctx, symbol, close_time) {
                    Some(v) => v,
                    None => continue,
                };
                let levels = match key_levels_at(ctx, symbol, close_time) {
                    Some(v) => v,
                    None => continue,
                };
                let daily_open = match levels.daily_open {
                    Some(v) if v > 0.0 => v,
                    _ => continue,
                };
                let weekly_open = match levels.weekly_open {
                    Some(v) if v > 0.0 => v,
                    _ => continue,
                };

                let close = candle.close;
                let atr = get_ind(&ind, "atr_14", i, close * 0.01);
                let atr_ratio = get_ind(&ind, "atr_ratio", i, 1.0);
                let vol_ratio = get_ind(&ind, "vol_ratio", i, 1.0);
                let body_ratio = get_ind(&ind, "body_ratio", i, 0.0).abs();
                let prev_body_ratio = get_ind(&ind, "body_ratio", i - 1, 0.0).abs();
                let vwap_20 = get_ind(&ind, "vwap_20", i, close);
                let vwap_20_prev = get_ind(&ind, "vwap_20", i - 1, prev.close);
                let poc_48 = get_ind(&ind, "poc_48", i, close);
                let poc_48_prev = get_ind(&ind, "poc_48", i - 1, prev.close);
                let t3 = get_ind(&ind, "t3", i, close);
                let t3_prev = get_ind(&ind, "t3", i - 1, prev.close);
                let cvd = get_ind(&ind, "cvd", i, 0.0);

                if atr <= 0.0 || atr_ratio > MAX_ATR_RATIO {
                    continue;
                }
                if vol_ratio < MIN_THRUST_VOL_RATIO || body_ratio < MIN_THRUST_BODY_RATIO {
                    continue;
                }
                if prev_body_ratio > MAX_SETUP_BODY_RATIO {
                    continue;
                }

                let bias = btc_bias_at(ctx, close_time);
                let btc_ret = btc_ret_24h.get(&close_time).copied().unwrap_or(0.0);
                let alt_ret = bar_return(symbol_candles, i, ALT_24H_BARS);
                let range_high = recent_max_high(symbol_candles, i, RANGE_LOOKBACK);
                let range_low = recent_min_low(symbol_candles, i, RANGE_LOOKBACK);
                let cvd_prev_max = recent_max_cvd(&ind, i, CVD_LOOKBACK);
                let cvd_prev_min = recent_min_cvd(&ind, i, CVD_LOOKBACK);

                let long_setup = matches!(bias, MarketBias::Bullish)
                    && funding.zscore_30d <= MAX_LONG_EUPHORIA_Z
                    && btc_ret >= BTC_24H_LONG_MIN
                    && (ALT_24H_LONG_MIN..=ALT_24H_LONG_MAX).contains(&alt_ret)
                    && prev.close > weekly_open
                    && prev.close > daily_open
                    && prev.close > vwap_20_prev
                    && prev.close > poc_48_prev
                    && prev.close > t3_prev
                    && close > weekly_open
                    && close > daily_open
                    && close > vwap_20
                    && close > poc_48
                    && close > t3
                    && close > candle.open
                    && close > range_high
                    && cvd > cvd_prev_max;

                let short_setup = matches!(bias, MarketBias::Bearish)
                    && crowded_longs(funding)
                    && btc_ret <= BTC_24H_SHORT_MAX
                    && (ALT_24H_SHORT_MIN..=ALT_24H_SHORT_MAX).contains(&alt_ret)
                    && prev.close < weekly_open
                    && prev.close < daily_open
                    && prev.close < vwap_20_prev
                    && prev.close < poc_48_prev
                    && prev.close < t3_prev
                    && close < weekly_open
                    && close < daily_open
                    && close < vwap_20
                    && close < poc_48
                    && close < t3
                    && close < candle.open
                    && close < range_low
                    && cvd < cvd_prev_min;

                let (position_type, pattern, size_multiplier) = if long_setup {
                    (
                        PositionType::Long,
                        "value_hold_thrust_long",
                        if crowded_shorts(funding) {
                            CROWD_BOOST_SIZE
                        } else {
                            BASE_SIZE
                        },
                    )
                } else if short_setup {
                    (
                        PositionType::Short,
                        "value_hold_thrust_short",
                        if crowded_longs(funding) {
                            CROWD_BOOST_SIZE
                        } else {
                            BASE_SIZE
                        },
                    )
                } else {
                    continue;
                };

                let tp_pct = atr_to_pct(atr, close, TP_ATR_MULT).clamp(2.0, 9.0);
                let sl_pct = atr_to_pct(atr, close, SL_ATR_MULT).clamp(0.8, 4.5);

                let mut metadata = HashMap::new();
                metadata.insert("daily_open".into(), json!(format!("{daily_open:.4}")));
                metadata.insert("weekly_open".into(), json!(format!("{weekly_open:.4}")));
                metadata.insert("vwap20".into(), json!(format!("{vwap_20:.4}")));
                metadata.insert("poc48".into(), json!(format!("{poc_48:.4}")));
                metadata.insert(
                    "funding_z".into(),
                    json!(format!("{:.3}", funding.zscore_30d)),
                );
                metadata.insert("btc_24h".into(), json!(format!("{btc_ret:.4}")));
                metadata.insert("alt_24h".into(), json!(format!("{alt_ret:.4}")));

                signals.push(Signal {
                    signal_date: close_time,
                    position_type,
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
                    max_holding_hours: MAX_HOLD_HOURS,
                    size_multiplier,
                    metadata,
                });
            }
        }

        signals.sort_by_key(|s| s.signal_date);
        signals
    }
}
