//! Adaptive Volatility-Cycle Strategy — V23
//!
//! V20 base (eval pref 41.95) + 4h squeeze release as secondary signal.
//! 1h signals filtered by 4h context (proven V20 approach).
//! 4h signals add trades from longer squeeze cycles with wider TP/SL.

use std::collections::{BTreeMap, HashMap};

use chrono::{DateTime, Utc};
use claude_trader_indicators::{compute_indicators, OhlcvFrame};
use claude_trader_models::{
    Candle, ContextKey, ContextMap, ContextValue, CooldownSpec, HtfData, MarketBias,
    MarketDataRequest, MarketType, PositionType, Signal,
};
use claude_trader_research_runtime::ResearchStrategy;
use serde_json::json;

pub struct Opus46Max16apr261;

/// Backtest universe for the live-safe opus46 deployment.
///
/// `MATICUSDT` was removed from Binance USD-M after the Polygon -> POL
/// migration. We intentionally do not substitute `POLUSDT`; adding a new
/// instrument changes the strategy universe and needs a separate re-test.
const SYMBOLS: &[&str] = &[
    "BTCUSDT", "ETHUSDT", "SOLUSDT", "NEARUSDT",
    "DOTUSDT", "ADAUSDT", "APTUSDT", "LTCUSDT",
    "AVAXUSDT", "LINKUSDT", "BNBUSDT", "ARBUSDT",
    "INJUSDT", "RENDERUSDT", "XRPUSDT", "DOGEUSDT",
    "SUIUSDT", "TONUSDT", "FILUSDT", "OPUSDT",
    "TRXUSDT",
];

const INDICATOR_COLUMNS_1H: &[&str] = &[
    "atr_14", "atr_ratio",
    "kc_upper", "kc_lower",
    "squeeze_on", "squeeze_count",
    "mom_slope", "ema_20",
    "vol_ratio",
    "body_ratio",
];

const INDICATOR_COLUMNS_4H: &[&str] = &[
    "atr_14", "atr_ratio",
    "kc_upper", "kc_lower",
    "squeeze_on", "squeeze_count",
    "mom_slope", "ema_20",
    "vol_ratio",
    "body_ratio",
];

const ANALYSIS_INTERVAL: &str = "1h";

const RELEASE_1H_MIN_SQ: f64 = 5.0;
const RELEASE_1H_TP_ATR: f64 = 3.0;
const RELEASE_1H_SL_ATR: f64 = 1.4;
const RELEASE_1H_MAX_HOLD: i64 = 60;

const RELEASE_4H_MIN_SQ: f64 = 3.0;
const RELEASE_4H_TP_ATR: f64 = 3.5;
const RELEASE_4H_SL_ATR: f64 = 1.6;
const RELEASE_4H_MAX_HOLD: i64 = 96;

const FUNDING_SHORT_BOOST: f64 = 0.0003;

fn get_ind(ind: &HashMap<String, Vec<f64>>, col: &str, i: usize, default: f64) -> f64 {
    ind.get(col)
        .and_then(|v| v.get(i).copied())
        .map(|v| if v.is_nan() { default } else { v })
        .unwrap_or(default)
}

fn latest_funding_rate(ctx: &ContextMap, symbol: &str, t: DateTime<Utc>) -> Option<f64> {
    match ctx.context_at(&ContextKey::Funding(symbol.to_string()), t) {
        Some(ContextValue::Funding(f)) => Some(f.rate),
        _ => None,
    }
}

fn atr_to_pct(atr: f64, close: f64, multiple: f64) -> f64 {
    (multiple * atr / close) * 100.0
}

fn btc_bias_bullish(ctx: &ContextMap, t: DateTime<Utc>) -> bool {
    matches!(
        ctx.context_at(&ContextKey::BtcStructure, t),
        Some(ContextValue::Bias(MarketBias::Bullish))
    )
}

fn htf_bearish_context(htf: &HtfData, symbol: &str, t: DateTime<Utc>) -> bool {
    let candles_4h = htf
        .additional_candles
        .get("4h")
        .and_then(|m| m.get(symbol));

    let candles_4h = match candles_4h {
        Some(c) if c.len() >= 25 => c,
        _ => return true,
    };

    let idx = candles_4h.partition_point(|c| c.close_time < t);
    if idx < 2 {
        return true;
    }
    let latest = idx - 1;

    let close_4h = candles_4h[latest].close;
    let open_4h = candles_4h[latest].open;
    let prev_close_4h = candles_4h[latest - 1].close;
    close_4h < open_4h || close_4h < prev_close_4h
}

/// Generate signals from 4h squeeze releases.
fn gen_4h_signals(
    htf: &HtfData,
    ctx: &ContextMap,
    symbol: &str,
    is_btc: bool,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
) -> Vec<Signal> {
    let mut signals = Vec::new();

    let candles_4h = match htf
        .additional_candles
        .get("4h")
        .and_then(|m| m.get(symbol))
    {
        Some(c) if c.len() >= 80 => c,
        _ => return signals,
    };

    let ind = match htf
        .additional_indicators
        .get("4h")
        .and_then(|m| m.get(symbol))
    {
        Some(i) => i,
        None => return signals,
    };

    let warmup = claude_trader_indicators::required_warmup(INDICATOR_COLUMNS_4H);

    for i in (warmup.max(5) + 1)..candles_4h.len() {
        let close_time = candles_4h[i].close_time;
        if close_time < start || close_time >= end {
            continue;
        }

        let close = candles_4h[i].close;
        let atr = get_ind(&ind, "atr_14", i, close * 0.01);
        let squeeze_on = get_ind(&ind, "squeeze_on", i - 1, 0.0);
        let prev_sq = get_ind(&ind, "squeeze_count", i - 2, 0.0);
        let mom_slope = get_ind(&ind, "mom_slope", i - 1, 0.0);
        let atr_ratio = get_ind(&ind, "atr_ratio", i, 1.0);
        let body_ratio = get_ind(&ind, "body_ratio", i - 1, 0.5);
        let vol_ratio = get_ind(&ind, "vol_ratio", i - 1, 1.0);

        if atr_ratio > 3.5 {
            continue;
        }

        let released = squeeze_on == 0.0
            && prev_sq >= RELEASE_4H_MIN_SQ
            && mom_slope < 0.0
            && body_ratio.abs() > 0.4
            && vol_ratio > 1.0;

        if !released {
            continue;
        }

        if !is_btc && btc_bias_bullish(ctx, close_time) {
            continue;
        }

        let tp = atr_to_pct(atr, close, RELEASE_4H_TP_ATR).max(2.0).min(12.0);
        let sl = atr_to_pct(atr, close, RELEASE_4H_SL_ATR).max(0.8).min(6.0);

        let mut md = HashMap::new();
        md.insert("tf".into(), json!("4h"));
        md.insert("mom".into(), json!(format!("{mom_slope:.4}")));
        md.insert("sq".into(), json!(prev_sq));

        signals.push(Signal {
            signal_date: close_time,
            position_type: PositionType::Short,
            ticker: symbol.to_string(),
            pattern: "release_short_4h".to_string(),
            tp_pct: Some(tp),
            sl_pct: Some(sl),
            tp_price: None,
            sl_price: None,
            leverage: 1.0,
            market_type: MarketType::Futures,
            taker_fee_rate: 0.0005,
            entry_price: None,
            fill_timeout_seconds: 3600,
            entry_delay_seconds: None,
            max_holding_hours: RELEASE_4H_MAX_HOLD,
            size_multiplier: 1.0,
            metadata: md,
        });
    }

    signals
}

impl ResearchStrategy for Opus46Max16apr261 {
    fn name(&self) -> &str {
        "opus46_max_16apr26_1_v24"
    }

    fn description(&self) -> String {
        "v24 21-symbol live-safe Binance USD-M universe; MATICUSDT removed, no POL substitution".to_string()
    }

    fn symbols(&self) -> Vec<String> {
        SYMBOLS.iter().map(|s| s.to_string()).collect()
    }

    fn indicator_columns(&self) -> &[&str] {
        INDICATOR_COLUMNS_1H
    }

    fn cooldown_spec(&self, signal: &Signal) -> CooldownSpec {
        CooldownSpec::symbol_side(signal, 6.0)
    }

    fn analysis_interval(&self) -> &str {
        ANALYSIS_INTERVAL
    }

    fn additional_intervals(&self) -> Vec<&str> {
        vec!["4h"]
    }

    fn indicator_columns_per_interval(&self) -> HashMap<&str, Vec<&str>> {
        let mut m = HashMap::new();
        m.insert("4h", INDICATOR_COLUMNS_4H.to_vec());
        m
    }

    fn extra_warmup_bars_per_interval(&self) -> HashMap<&str, usize> {
        let mut m = HashMap::new();
        m.insert("4h", 80);
        m
    }

    fn extra_warmup_bars(&self) -> usize {
        120
    }

    fn required_context(&self) -> Vec<ContextKey> {
        let mut v = vec![ContextKey::BtcStructure];
        for sym in self.symbols() {
            v.push(ContextKey::Funding(sym));
        }
        v
    }

    fn market_data_request(&self) -> MarketDataRequest {
        MarketDataRequest {
            ohlcv_interval: self.analysis_interval().to_string(),
            ..Default::default()
        }
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
        let warmup = claude_trader_indicators::required_warmup(INDICATOR_COLUMNS_1H);
        let mut signals = Vec::new();

        for (symbol, &symbol_candles) in candles {
            if symbol_candles.len() < warmup + 5 {
                continue;
            }

            let ohlcv = OhlcvFrame {
                open: symbol_candles.iter().map(|c| c.open).collect(),
                high: symbol_candles.iter().map(|c| c.high).collect(),
                low: symbol_candles.iter().map(|c| c.low).collect(),
                close: symbol_candles.iter().map(|c| c.close).collect(),
                volume: symbol_candles.iter().map(|c| c.volume).collect(),
                taker_buy_volume: symbol_candles
                    .iter()
                    .map(|c| c.taker_buy_volume)
                    .collect(),
            };
            let ind = compute_indicators(&ohlcv, INDICATOR_COLUMNS_1H)
                .expect("invalid INDICATOR_COLUMNS_1H");

            let is_btc = symbol == "BTCUSDT";

            for i in warmup.max(5)..symbol_candles.len() {
                let close_time = symbol_candles[i].close_time;
                if close_time < start || close_time >= end {
                    continue;
                }

                let close = symbol_candles[i].close;
                let atr = get_ind(&ind, "atr_14", i, close * 0.01);
                let squeeze_on = get_ind(&ind, "squeeze_on", i, 0.0);
                let prev_squeeze_count = get_ind(&ind, "squeeze_count", i - 1, 0.0);
                let mom_slope = get_ind(&ind, "mom_slope", i, 0.0);
                let atr_ratio = get_ind(&ind, "atr_ratio", i, 1.0);
                let body_ratio = get_ind(&ind, "body_ratio", i, 0.5);
                let vol_ratio = get_ind(&ind, "vol_ratio", i, 1.0);

                if atr_ratio > 3.5 {
                    continue;
                }

                let released = squeeze_on == 0.0
                    && prev_squeeze_count >= RELEASE_1H_MIN_SQ
                    && mom_slope < 0.0
                    && body_ratio.abs() > 0.4
                    && vol_ratio > 1.0;

                if !released {
                    continue;
                }
                if !is_btc && btc_bias_bullish(ctx, close_time) {
                    continue;
                }
                if !htf_bearish_context(htf, symbol, close_time) {
                    continue;
                }

                let funding = latest_funding_rate(ctx, symbol, close_time);
                let funding_favors = funding.map_or(false, |f| f > FUNDING_SHORT_BOOST);

                let tp = atr_to_pct(atr, close, RELEASE_1H_TP_ATR).max(1.5).min(10.0);
                let sl = atr_to_pct(atr, close, RELEASE_1H_SL_ATR).max(0.6).min(5.0);

                let mut md = HashMap::new();
                md.insert("tf".into(), json!("1h"));
                md.insert("mom".into(), json!(format!("{mom_slope:.4}")));
                md.insert("sq".into(), json!(prev_squeeze_count));

                signals.push(Signal {
                    signal_date: close_time,
                    position_type: PositionType::Short,
                    ticker: symbol.clone(),
                    pattern: "release_short_1h".to_string(),
                    tp_pct: Some(tp),
                    sl_pct: Some(sl),
                    tp_price: None,
                    sl_price: None,
                    leverage: 1.0,
                    market_type: MarketType::Futures,
                    taker_fee_rate: 0.0005,
                    entry_price: None,
                    fill_timeout_seconds: 3600,
                    entry_delay_seconds: None,
                    max_holding_hours: RELEASE_1H_MAX_HOLD,
                    size_multiplier: if funding_favors { 1.3 } else { 1.0 },
                    metadata: md,
                });
            }

            signals.extend(gen_4h_signals(htf, ctx, symbol, is_btc, start, end));
        }

        signals.sort_by_key(|s| s.signal_date);
        signals
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use claude_trader_research_runtime::ResearchStrategy;

    #[test]
    fn live_safe_universe_excludes_delisted_maticusdt() {
        let strategy = Opus46Max16apr261;
        let symbols = strategy.symbols();
        assert_eq!(symbols.len(), 21);
        assert!(!symbols.iter().any(|s| s == "MATICUSDT"));
    }
}
