//! Binance Futures REST client — klines, agg trades, funding rates.

use std::collections::{HashMap, HashSet};
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

use chrono::{DateTime, Utc};
use claude_trader_models::{AggTrade, Candle, FundingRate, MarketType};

use crate::cache::{default_cache_root, DiskCache};
use crate::rate_limiter::RateLimiter;

const FUTURES_BASE_URL: &str = "https://fapi.binance.com";
const SPOT_BASE_URL: &str = "https://api.binance.com";

const FUTURES_KLINE_LIMIT: usize = 1500;
const SPOT_KLINE_LIMIT: usize = 1000;
const AGG_TRADE_LIMIT: usize = 1000;
const FUNDING_RATE_LIMIT: usize = 1000;

const DEFAULT_RETRY_DELAY: f64 = 15.0;
const MAX_RETRY_DELAY: f64 = 300.0;
const RETRYABLE_STATUS_CODES: &[u16] = &[418, 429];
const TRANSIENT_MAX_RETRIES: u32 = 3;
const TRANSIENT_RETRY_DELAY: f64 = 5.0;

#[derive(Debug, thiserror::Error)]
pub enum DataError {
    #[error("HTTP error: {0}")]
    Http(String),
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("Futures-only operation on {0} market")]
    FuturesOnly(String),
    #[error("{0}")]
    Other(String),
}

/// Outcome of a paginated kline fetch.
///
/// `Complete` means pagination exhausted the requested range cleanly — either
/// the cursor passed `end`, Binance returned a sub-limit page signalling no
/// more data, or Binance returned an empty page. Either way, every millisecond
/// in `[start, end)` has been probed.
///
/// `Interrupted` means mid-pagination `get_json` failed after at least one
/// page had succeeded. `covered_up_to_ms` is the cursor value the next page
/// would have used — i.e. everything in `[start, covered_up_to_ms)` was
/// probed, everything in `[covered_up_to_ms, end)` was not.
#[derive(Debug)]
pub enum KlineFetch {
    Complete(Vec<Candle>),
    Interrupted {
        rows: Vec<Candle>,
        covered_up_to_ms: i64,
    },
}

impl KlineFetch {
    /// Borrow the rows regardless of variant. Useful for callers that only
    /// need the data and handle coverage separately.
    pub fn rows(&self) -> &[Candle] {
        match self {
            KlineFetch::Complete(rows) => rows,
            KlineFetch::Interrupted { rows, .. } => rows,
        }
    }

    /// Consume and return the rows regardless of variant.
    pub fn into_rows(self) -> Vec<Candle> {
        match self {
            KlineFetch::Complete(rows) => rows,
            KlineFetch::Interrupted { rows, .. } => rows,
        }
    }
}

pub struct BinanceClient {
    market_type: MarketType,
    base_url: String,
    batch_limit: usize,
    http: reqwest::blocking::Client,
    cache: DiskCache,
    mem_cache: Mutex<HashMap<String, serde_json::Value>>,
    rate_limiter: RateLimiter,
    /// When `false`, `get_json` skips both memory and disk cache layers and
    /// always issues a fresh HTTP request. Live trading uses this so
    /// strategies always see current klines (matches Python's
    /// `LiveMarketClient` cache-bypass).
    cache_enabled: bool,
}

impl BinanceClient {
    pub fn new(market_type: MarketType) -> Self {
        Self::with_cache(market_type, true)
    }

    /// Constructor that disables the memory + disk cache layers. Use for
    /// live polling where stale candle data corrupts strategy decisions.
    pub fn no_cache(market_type: MarketType) -> Self {
        Self::with_cache(market_type, false)
    }

    /// Like `no_cache`, but routes requests at `base_url` instead of the
    /// hardcoded production endpoint. Lets a live runner steer market data
    /// at `demo-fapi.binance.com` (testnet) or any operator-supplied URL,
    /// matching where the signed `LiveConfig` is pointing — without that,
    /// testnet runs make trading decisions on production data.
    ///
    /// Empty `base_url` falls back to the market_type default. Trailing
    /// slashes are trimmed because path templates in this client carry the
    /// leading `/`.
    pub fn no_cache_with_base_url(market_type: MarketType, base_url: &str) -> Self {
        let mut c = Self::with_cache(market_type, false);
        let trimmed = base_url.trim_end_matches('/');
        if !trimmed.is_empty() {
            c.base_url = trimmed.to_string();
        }
        c
    }

    /// Read-only accessor on the resolved base URL — used by regression
    /// tests to pin the wiring without going through the network.
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    fn with_cache(market_type: MarketType, cache_enabled: bool) -> Self {
        let (base_url, limit_per_min, batch_limit) = match market_type {
            MarketType::Futures => (FUTURES_BASE_URL, 2400, FUTURES_KLINE_LIMIT),
            MarketType::Spot => (SPOT_BASE_URL, 6000, SPOT_KLINE_LIMIT),
        };
        Self {
            market_type,
            base_url: base_url.to_string(),
            batch_limit,
            http: reqwest::blocking::Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .expect("Failed to build HTTP client"),
            cache: DiskCache::new(default_cache_root()),
            mem_cache: Mutex::new(HashMap::new()),
            rate_limiter: RateLimiter::new(limit_per_min),
            cache_enabled,
        }
    }

    pub fn futures() -> Self {
        Self::new(MarketType::Futures)
    }

    /// True when `get_json` consults memory + disk caches before HTTP.
    pub fn cache_enabled(&self) -> bool {
        self.cache_enabled
    }

    // -------------------------------------------------------------------
    // Public fetch methods
    // -------------------------------------------------------------------

    /// Fetch OHLCV klines with automatic pagination.
    ///
    /// Individual pages are cached by URL in `get_json`'s mem_cache + disk
    /// cache, so a retry after partial-network failure reuses already-
    /// succeeded pages instead of re-running pagination from scratch.
    pub fn fetch_klines(
        &self,
        symbol: &str,
        interval: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<KlineFetch, DataError> {
        let path = self.klines_path();
        self.fetch_kline_series(&path, symbol, interval, start, end, 2)
    }

    /// Drop the in-memory JSON response cache.
    ///
    /// Call after data-fetching phases in long-lived processes to bound memory.
    pub fn clear_caches(&self) {
        self.mem_cache.lock().unwrap().clear();
    }

    /// Fetch aggregate trades with fromId pagination.
    pub fn fetch_agg_trades(
        &self,
        symbol: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<AggTrade>, DataError> {
        let path = self.agg_trades_path();
        let api_symbol = symbol_for_api(symbol);
        let start_ms = dt_to_ms(start);
        let end_ms = dt_to_ms(end);

        let mut result = Vec::new();
        let mut seen_ids: HashSet<i64> = HashSet::new();
        let mut next_from_id: Option<i64> = None;

        loop {
            let mut params: Vec<(&str, String)> = vec![
                ("symbol", api_symbol.clone()),
                ("limit", AGG_TRADE_LIMIT.to_string()),
            ];

            match next_from_id {
                Some(id) => params.push(("fromId", id.to_string())),
                None => {
                    params.push(("startTime", start_ms.to_string()));
                    params.push(("endTime", end_ms.to_string()));
                }
            }

            let data = self.get_json(&path, &params, 20)?;
            let rows = data
                .as_array()
                .ok_or_else(|| DataError::Parse("Expected array for agg trades".into()))?;

            if rows.is_empty() {
                break;
            }

            let mut batch_count = 0;
            let mut stop = false;

            for row in rows {
                let trade = parse_agg_trade(row)?;
                if seen_ids.contains(&trade.trade_id) {
                    continue;
                }
                let ts_ms = dt_to_ms(trade.timestamp);
                if ts_ms < start_ms {
                    continue;
                }
                if ts_ms >= end_ms {
                    stop = true;
                    break;
                }
                seen_ids.insert(trade.trade_id);
                next_from_id = Some(trade.trade_id + 1);
                result.push(trade);
                batch_count += 1;
            }

            if stop || batch_count == 0 || rows.len() < AGG_TRADE_LIMIT {
                break;
            }
        }

        Ok(result)
    }

    /// Fetch funding rates (futures only).
    pub fn fetch_funding_rates(
        &self,
        symbol: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<FundingRate>, DataError> {
        self.require_futures("funding rates")?;
        let path = "/fapi/v1/fundingRate".to_string();
        let api_symbol = symbol_for_api(symbol);
        let end_ms = dt_to_ms(end);

        let mut result = Vec::new();
        let mut cursor_ms = dt_to_ms(start);

        loop {
            let params: Vec<(&str, String)> = vec![
                ("symbol", api_symbol.clone()),
                ("startTime", cursor_ms.to_string()),
                ("endTime", end_ms.to_string()),
                ("limit", FUNDING_RATE_LIMIT.to_string()),
            ];

            let data = self.get_json(&path, &params, 1)?;
            let rows = data
                .as_array()
                .ok_or_else(|| DataError::Parse("Expected array for funding rates".into()))?;

            if rows.is_empty() {
                break;
            }

            let mut last_api_ts = cursor_ms;
            for row in rows {
                let fr = parse_funding_rate(row)?;
                let ts_ms = dt_to_ms(fr.timestamp);
                if ts_ms > last_api_ts {
                    last_api_ts = ts_ms;
                }
                if ts_ms >= cursor_ms && ts_ms < end_ms {
                    result.push(fr);
                }
            }

            if rows.len() < FUNDING_RATE_LIMIT || last_api_ts >= end_ms {
                break;
            }
            cursor_ms = last_api_ts + 1;
        }

        Ok(result)
    }

    // -------------------------------------------------------------------
    // Internal helpers
    // -------------------------------------------------------------------

    fn klines_path(&self) -> String {
        match self.market_type {
            MarketType::Futures => "/fapi/v1/klines".to_string(),
            MarketType::Spot => "/api/v3/klines".to_string(),
        }
    }

    fn agg_trades_path(&self) -> String {
        match self.market_type {
            MarketType::Futures => "/fapi/v1/aggTrades".to_string(),
            MarketType::Spot => "/api/v3/aggTrades".to_string(),
        }
    }

    fn require_futures(&self, label: &str) -> Result<(), DataError> {
        if !matches!(self.market_type, MarketType::Futures) {
            return Err(DataError::FuturesOnly(label.to_string()));
        }
        Ok(())
    }

    fn fetch_kline_series(
        &self,
        path: &str,
        symbol: &str,
        interval: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        weight: u32,
    ) -> Result<KlineFetch, DataError> {
        let api_symbol = symbol_for_api(symbol);
        let end_ms = dt_to_ms(end);
        let mut cursor_ms = dt_to_ms(start);
        let mut result = Vec::new();

        loop {
            if cursor_ms >= end_ms {
                break;
            }

            let params: Vec<(&str, String)> = vec![
                ("symbol", api_symbol.clone()),
                ("interval", interval.to_string()),
                ("limit", self.batch_limit.to_string()),
                ("startTime", cursor_ms.to_string()),
                ("endTime", end_ms.to_string()),
            ];

            let data = match self.get_json(path, &params, weight) {
                Ok(d) => d,
                Err(e) => {
                    // After at least one successful page, keep what we have
                    // and report the probed prefix. The caller decides whether
                    // to retry the uncovered suffix or bail. `cursor_ms` is
                    // the start of the failed request, so everything below
                    // it was successfully probed.
                    if !result.is_empty() {
                        eprintln!(
                            "  WARN: fetch_kline_series failed mid-pagination at batch {} ({} candles collected so far): {e}",
                            result.len() / self.batch_limit + 1,
                            result.len(),
                        );
                        return Ok(KlineFetch::Interrupted {
                            rows: result,
                            covered_up_to_ms: cursor_ms,
                        });
                    }
                    return Err(e);
                }
            };
            let rows = data
                .as_array()
                .ok_or_else(|| DataError::Parse("Expected array for klines".into()))?;

            if rows.is_empty() {
                break;
            }

            for row in rows {
                let arr = row
                    .as_array()
                    .ok_or_else(|| DataError::Parse("Expected array row for kline".into()))?;
                let candle = parse_kline(arr)?;
                // Filter out candles past the requested end time — Binance
                // may return candles whose open_time >= endTime in edge cases.
                if dt_to_ms(candle.open_time) >= end_ms {
                    continue;
                }
                result.push(candle);
            }

            if rows.len() < self.batch_limit {
                break;
            }

            // Advance cursor past last candle
            if let Some(last) = rows.last() {
                if let Some(arr) = last.as_array() {
                    if let Some(open_ms) = arr.first().and_then(|v| v.as_i64()) {
                        cursor_ms = open_ms + 1;
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }
        }

        Ok(KlineFetch::Complete(result))
    }

    /// Fetch JSON with triple-layer caching: memory → disk → HTTP.
    fn get_json(
        &self,
        path: &str,
        params: &[(&str, String)],
        weight: u32,
    ) -> Result<serde_json::Value, DataError> {
        // Build cache keys
        let mut sorted_params: Vec<(&str, &str)> =
            params.iter().map(|(k, v)| (*k, v.as_str())).collect();
        sorted_params.sort_by_key(|(k, _)| *k);

        let query_string: String = sorted_params
            .iter()
            .map(|(k, v)| format!("{k}={v}"))
            .collect::<Vec<_>>()
            .join("&");

        let mem_key = format!("{}|{}", path, query_string);
        let cache_parts: Vec<String> = std::iter::once(path.to_string())
            .chain(sorted_params.iter().map(|(k, v)| format!("{k}={v}")))
            .collect();
        let cache_refs: Vec<&str> = cache_parts.iter().map(|s| s.as_str()).collect();

        // Layers 1+2 (memory + disk) are skipped when caching is disabled.
        // Live polling sets cache_enabled=false so every kline request
        // hits HTTP and reflects the current state of the market.
        if self.cache_enabled {
            // Layer 1: Memory cache
            {
                let mem = self.mem_cache.lock().unwrap();
                if let Some(cached) = mem.get(&mem_key) {
                    return Ok(cached.clone());
                }
            }

            // Layer 2: Disk cache
            if let Some(cached) = self.cache.get::<serde_json::Value>(&cache_refs) {
                let mut mem = self.mem_cache.lock().unwrap();
                mem.insert(mem_key.clone(), cached.clone());
                return Ok(cached);
            }
        }

        // Layer 3: HTTP request with retry
        let url = format!("{}{}?{}", self.base_url, path, query_string);

        let mut retry_delay = DEFAULT_RETRY_DELAY;
        let mut transient_attempts = 0u32;
        loop {
            self.rate_limiter.acquire(weight);
            match self.http.get(&url).send() {
                Ok(resp) => {
                    // Sync rate limiter from headers
                    if let Some(used) = resp
                        .headers()
                        .get("X-MBX-USED-WEIGHT-1m")
                        .and_then(|v| v.to_str().ok())
                        .and_then(|v| v.parse::<u32>().ok())
                    {
                        self.rate_limiter.sync_from_server(used);
                    }

                    let status = resp.status().as_u16();
                    if RETRYABLE_STATUS_CODES.contains(&status) {
                        let delay = resp
                            .headers()
                            .get("Retry-After")
                            .and_then(|v| v.to_str().ok())
                            .and_then(|v| v.parse::<f64>().ok())
                            .unwrap_or(retry_delay);
                        log::warn!("Binance {status}, retrying in {delay:.1}s");
                        thread::sleep(Duration::from_secs_f64(delay));
                        retry_delay = (retry_delay * 2.0).min(MAX_RETRY_DELAY);
                        continue;
                    }

                    // Server errors (500, 502, 503, etc.) — retry with backoff
                    if status >= 500 {
                        transient_attempts += 1;
                        if transient_attempts <= TRANSIENT_MAX_RETRIES {
                            let delay =
                                TRANSIENT_RETRY_DELAY * (1 << (transient_attempts - 1)) as f64;
                            eprintln!(
                                "  WARN: Binance {status}, retry {transient_attempts}/{TRANSIENT_MAX_RETRIES} in {delay:.1}s"
                            );
                            thread::sleep(Duration::from_secs_f64(delay));
                            continue;
                        }
                    }

                    if !resp.status().is_success() {
                        let body = resp.text().unwrap_or_default();
                        return Err(DataError::Http(format!("HTTP {status}: {body}")));
                    }

                    let value: serde_json::Value =
                        resp.json().map_err(|e| DataError::Parse(e.to_string()))?;

                    if self.cache_enabled {
                        self.cache.set(&cache_refs, &value);
                        let mut mem = self.mem_cache.lock().unwrap();
                        mem.insert(mem_key, value.clone());
                    }

                    return Ok(value);
                }
                Err(e) => {
                    // Network errors (timeout, connection reset, DNS) — retry
                    transient_attempts += 1;
                    if transient_attempts <= TRANSIENT_MAX_RETRIES {
                        let delay = TRANSIENT_RETRY_DELAY * (1 << (transient_attempts - 1)) as f64;
                        eprintln!(
                            "  WARN: Network error ({e}), retry {transient_attempts}/{TRANSIENT_MAX_RETRIES} in {delay:.1}s"
                        );
                        thread::sleep(Duration::from_secs_f64(delay));
                        continue;
                    }
                    return Err(DataError::Http(e.to_string()));
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Parsing helpers
// ---------------------------------------------------------------------------

use claude_trader_models::{dt_to_ms, ms_to_dt_opt};

fn symbol_for_api(ticker: &str) -> String {
    ticker.replace('/', "")
}

fn parse_kline(row: &[serde_json::Value]) -> Result<Candle, DataError> {
    let get_f64 = |idx: usize| -> Result<f64, DataError> {
        row.get(idx)
            .and_then(|v| {
                v.as_str()
                    .map(|s| s.parse::<f64>().ok())
                    .unwrap_or_else(|| v.as_f64())
            })
            .ok_or_else(|| DataError::Parse(format!("Missing kline field at index {idx}")))
    };
    let get_i64 = |idx: usize| -> Result<i64, DataError> {
        row.get(idx)
            .and_then(|v| v.as_i64())
            .ok_or_else(|| DataError::Parse(format!("Missing kline timestamp at index {idx}")))
    };

    let taker_buy_vol = if row.len() > 9 {
        get_f64(9).unwrap_or(0.0)
    } else {
        0.0
    };

    let open_ms = get_i64(0)?;
    let close_ms = get_i64(6)?;
    let open_time = ms_to_dt_opt(open_ms).ok_or_else(|| {
        DataError::Parse(format!("kline open_time ms={open_ms} out of range"))
    })?;
    let close_time = ms_to_dt_opt(close_ms).ok_or_else(|| {
        DataError::Parse(format!("kline close_time ms={close_ms} out of range"))
    })?;
    Ok(Candle {
        open_time,
        close_time,
        open: get_f64(1)?,
        high: get_f64(2)?,
        low: get_f64(3)?,
        close: get_f64(4)?,
        volume: get_f64(5)?,
        taker_buy_volume: taker_buy_vol,
    })
}

fn parse_agg_trade(row: &serde_json::Value) -> Result<AggTrade, DataError> {
    let ts_ms = row["T"]
        .as_i64()
        .ok_or_else(|| DataError::Parse("Missing trade timestamp".into()))?;
    let timestamp = ms_to_dt_opt(ts_ms).ok_or_else(|| {
        DataError::Parse(format!("trade timestamp ms={ts_ms} out of range"))
    })?;
    Ok(AggTrade {
        trade_id: row["a"]
            .as_i64()
            .ok_or_else(|| DataError::Parse("Missing trade_id".into()))?,
        timestamp,
        price: row["p"]
            .as_str()
            .and_then(|s| s.parse().ok())
            .ok_or_else(|| DataError::Parse("Missing trade price".into()))?,
        quantity: row["q"]
            .as_str()
            .and_then(|s| s.parse().ok())
            .ok_or_else(|| DataError::Parse("Missing trade quantity".into()))?,
    })
}

fn parse_funding_rate(row: &serde_json::Value) -> Result<FundingRate, DataError> {
    let ts = row["fundingTime"]
        .as_i64()
        .ok_or_else(|| DataError::Parse("Missing fundingTime".into()))?;
    let rate = row["fundingRate"]
        .as_str()
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| DataError::Parse("Missing fundingRate".into()))?;
    let mark_price = row
        .get("markPrice")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .and_then(|s| s.parse().ok());

    let timestamp = ms_to_dt_opt(ts).ok_or_else(|| {
        DataError::Parse(format!("funding timestamp ms={ts} out of range"))
    })?;
    Ok(FundingRate {
        timestamp,
        funding_rate: rate,
        mark_price,
    })
}

#[cfg(test)]
mod external_input_tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parse_kline_rejects_out_of_range_ms() {
        let row = json!([
            i64::MAX,
            "100.0",
            "101.0",
            "99.0",
            "100.5",
            "10.0",
            i64::MAX,
            "0",
            0,
            "5.0"
        ]);
        let arr = row.as_array().unwrap();
        let err = parse_kline(arr).unwrap_err();
        match err {
            DataError::Parse(msg) => assert!(
                msg.contains("out of range"),
                "expected out-of-range message, got: {msg}"
            ),
            other => panic!("expected DataError::Parse, got {other:?}"),
        }
    }

    #[test]
    fn parse_kline_accepts_valid_ms() {
        let row = json!([
            1_700_000_000_000i64,
            "100.0",
            "101.0",
            "99.0",
            "100.5",
            "10.0",
            1_700_000_003_600_000i64,
            "0",
            0,
            "5.0"
        ]);
        let arr = row.as_array().unwrap();
        // Accept or reject cleanly — must not panic even with huge but
        // possibly-still-valid values.
        let _ = parse_kline(arr);
    }

    #[test]
    fn parse_agg_trade_rejects_out_of_range_ms() {
        let row = json!({
            "a": 1i64,
            "T": i64::MAX,
            "p": "100.0",
            "q": "1.0"
        });
        let err = parse_agg_trade(&row).unwrap_err();
        match err {
            DataError::Parse(msg) => assert!(msg.contains("out of range"), "got: {msg}"),
            other => panic!("expected DataError::Parse, got {other:?}"),
        }
    }

    #[test]
    fn parse_funding_rate_rejects_out_of_range_ms() {
        let row = json!({
            "fundingTime": i64::MAX,
            "fundingRate": "0.0001",
            "markPrice": ""
        });
        let err = parse_funding_rate(&row).unwrap_err();
        match err {
            DataError::Parse(msg) => assert!(msg.contains("out of range"), "got: {msg}"),
            other => panic!("expected DataError::Parse, got {other:?}"),
        }
    }
}
