//! Exchange-info cache + Decimal-based filter rounding.
//!
//! Mirrors `live/executor.OrderExecutor._symbol_info / _round_quantity_* /
//! _round_price / _minimum_quantity / _minimum_notional` from the Python
//! runtime, with one binding upgrade: all step/tick math runs in
//! `rust_decimal::Decimal` so `0.1 + 0.2 != 0.3` floating-point error never
//! lands on Binance's filter checks.
//!
//! Filter precedence:
//! - Quantity rounding for **market** entries uses `MARKET_LOT_SIZE` when the
//!   symbol declares one, falling back to `LOT_SIZE`. Quantity rounding for
//!   **limit** entries always uses `LOT_SIZE`. Mirrors Python's
//!   `_quantity_filter(use_market_filter=…)`.
//! - Price rounding uses `PRICE_FILTER.tickSize`, floored toward zero
//!   (`price - price%tick`) to match Python.
//! - `min_notional` accepts either `MIN_NOTIONAL` or `NOTIONAL` filter type
//!   (Binance has used both names over time).
//!
//! The cache is loaded lazily: `ensure_loaded(symbol)` fetches *all* symbols
//! via `FuturesApi::get_exchange_info` on the first miss, since one HTTP
//! call returns the full list. Subsequent symbols hit the cache for free.
//! Tests construct a cache with `populate_from_exchange_info` over a
//! hand-written JSON fixture so they don't need an HTTP layer.

use std::collections::HashMap;
use std::str::FromStr;
use std::sync::{Arc, Mutex};

use rust_decimal::prelude::*;
use rust_decimal::{Decimal, RoundingStrategy};

use crate::auth_client::{symbol_for_api, FuturesApi};
use crate::error::{LiveError, Result};

/// Convert f64 → Decimal via the shortest round-trip string representation
/// (Rust's `Display` for f64 emits the shortest decimal literal that
/// round-trips back to the exact f64 bit pattern). Mirrors Python's
/// `Decimal(str(value))` — the only conversion path that doesn't carry
/// binary-representation noise (e.g. `0.1` truly becoming `0.1`, not
/// `0.10000000000000000555…`).
fn dec_from_f64(v: f64) -> Option<Decimal> {
    if !v.is_finite() {
        return None;
    }
    Decimal::from_str(&format!("{v}")).ok()
}

// ---------------------------------------------------------------------------
// Filter types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct PriceFilter {
    pub tick_size: Decimal,
    pub min_price: Option<Decimal>,
    pub max_price: Option<Decimal>,
}

#[derive(Debug, Clone)]
pub struct LotSize {
    pub step_size: Decimal,
    pub min_qty: Decimal,
    pub max_qty: Option<Decimal>,
}

#[derive(Debug, Clone)]
pub struct SymbolInfo {
    pub symbol: String,
    pub price_filter: Option<PriceFilter>,
    pub lot_size: Option<LotSize>,
    pub market_lot_size: Option<LotSize>,
    pub min_notional: Option<Decimal>,
}

impl SymbolInfo {
    /// Pick the appropriate quantity filter. `use_market_filter=true` (market
    /// entries / closes) prefers `MARKET_LOT_SIZE` when present, falling back
    /// to `LOT_SIZE`. Limit entries always use `LOT_SIZE`.
    pub fn quantity_filter(&self, use_market_filter: bool) -> Option<&LotSize> {
        if use_market_filter {
            if let Some(m) = &self.market_lot_size {
                return Some(m);
            }
        }
        self.lot_size.as_ref()
    }
}

// ---------------------------------------------------------------------------
// ExchangeInfoCache
// ---------------------------------------------------------------------------

pub struct ExchangeInfoCache {
    client: Option<Arc<dyn FuturesApi>>,
    by_symbol: Mutex<HashMap<String, SymbolInfo>>,
}

impl ExchangeInfoCache {
    /// Production constructor — lazy-load via `FuturesApi`.
    pub fn new(client: Arc<dyn FuturesApi>) -> Self {
        Self {
            client: Some(client),
            by_symbol: Mutex::new(HashMap::new()),
        }
    }

    /// Test constructor — pre-populated, no HTTP layer required.
    /// `ensure_loaded` becomes a no-op (looks up the cache directly; misses
    /// surface `UnknownSymbol`).
    pub fn from_static(symbols: HashMap<String, SymbolInfo>) -> Self {
        Self {
            client: None,
            by_symbol: Mutex::new(symbols),
        }
    }

    /// Load the symbol from Binance if not already cached.
    ///
    /// One call to `get_exchange_info()` populates *every* symbol on the
    /// futures venue, so subsequent symbol lookups are free.
    pub fn ensure_loaded(&self, symbol: &str) -> Result<()> {
        let api_symbol = symbol_for_api(symbol);
        if self.by_symbol.lock().unwrap().contains_key(&api_symbol) {
            return Ok(());
        }
        let Some(client) = self.client.as_ref() else {
            return Err(LiveError::UnknownSymbol(api_symbol));
        };
        let raw = client.get_exchange_info()?;
        self.populate_from_exchange_info(&raw)?;
        if !self.by_symbol.lock().unwrap().contains_key(&api_symbol) {
            return Err(LiveError::UnknownSymbol(api_symbol));
        }
        Ok(())
    }

    /// Parse a Binance-shaped `exchangeInfo` payload and replace the cache.
    /// Used directly by tests; production loads via `ensure_loaded`.
    pub fn populate_from_exchange_info(&self, raw: &serde_json::Value) -> Result<()> {
        let symbols = raw["symbols"]
            .as_array()
            .ok_or_else(|| LiveError::Parse("exchangeInfo.symbols missing".into()))?;
        let mut map = self.by_symbol.lock().unwrap();
        map.clear();
        for s in symbols {
            let info = parse_symbol(s)?;
            map.insert(info.symbol.clone(), info);
        }
        Ok(())
    }

    /// Return a clone of the cached entry for `symbol`. Returns
    /// `UnknownSymbol` when the lazy load did not pull this symbol.
    pub fn get(&self, symbol: &str) -> Result<SymbolInfo> {
        let api = symbol_for_api(symbol);
        self.by_symbol
            .lock()
            .unwrap()
            .get(&api)
            .cloned()
            .ok_or(LiveError::UnknownSymbol(api))
    }

    // -- Rounding -----------------------------------------------------------

    /// Round `qty` *down* to the symbol's step size (toward zero).
    /// Used for initial sizing of an entry and for closes (never oversell).
    pub fn round_quantity_down(
        &self,
        symbol: &str,
        qty: f64,
        use_market_filter: bool,
    ) -> Result<f64> {
        self.round_quantity_with_strategy(
            symbol,
            qty,
            use_market_filter,
            RoundingStrategy::ToZero,
        )
    }

    /// Round `qty` *up* to the symbol's step size (away from zero).
    /// Used after a min-quantity / min-notional bump so the bump survives the
    /// final step alignment instead of being re-truncated below the minimum.
    pub fn round_quantity_up(
        &self,
        symbol: &str,
        qty: f64,
        use_market_filter: bool,
    ) -> Result<f64> {
        self.round_quantity_with_strategy(
            symbol,
            qty,
            use_market_filter,
            RoundingStrategy::AwayFromZero,
        )
    }

    fn round_quantity_with_strategy(
        &self,
        symbol: &str,
        qty: f64,
        use_market_filter: bool,
        strategy: RoundingStrategy,
    ) -> Result<f64> {
        // Mirror Python: missing symbol / missing filter / non-positive step
        // all return the raw input. Non-positive qty rounds to 0.
        let info = self.get(symbol)?;
        let Some(filter) = info.quantity_filter(use_market_filter) else {
            return Ok(qty);
        };
        if filter.step_size <= Decimal::ZERO {
            return Ok(qty);
        }
        let Some(dec_qty) = dec_from_f64(qty) else {
            return Ok(qty);
        };
        if dec_qty <= Decimal::ZERO {
            return Ok(0.0);
        }
        // (qty / step).round_to_integral(strategy) * step
        let scaled = (dec_qty / filter.step_size).round_dp_with_strategy(0, strategy);
        let rounded = scaled * filter.step_size;
        if rounded <= Decimal::ZERO {
            return Ok(0.0);
        }
        Ok(rounded.to_f64().unwrap_or(qty))
    }

    /// Floor `price` to the nearest tickSize boundary (toward zero).
    /// Matches Python's `round(price - (price % tick), precision)` — tests
    /// pin both behaviors equivalent on the cases we care about.
    pub fn round_price(&self, symbol: &str, price: f64) -> Result<f64> {
        let info = self.get(symbol)?;
        let Some(filter) = info.price_filter else {
            return Ok(price);
        };
        if filter.tick_size <= Decimal::ZERO {
            return Ok(price);
        }
        let Some(dec_price) = dec_from_f64(price) else {
            return Ok(price);
        };
        let scaled = (dec_price / filter.tick_size).floor();
        let rounded = scaled * filter.tick_size;
        Ok(rounded.to_f64().unwrap_or(price))
    }

    /// Minimum order quantity for the chosen filter. Returns 0.0 when the
    /// symbol has no filter (Python parity — caller skips the bump).
    pub fn min_qty(&self, symbol: &str, use_market_filter: bool) -> Result<f64> {
        let info = self.get(symbol)?;
        Ok(info
            .quantity_filter(use_market_filter)
            .map(|f| f.min_qty.to_f64().unwrap_or(0.0))
            .unwrap_or(0.0))
    }

    /// Minimum notional in quote currency. Returns 0.0 when the symbol has no
    /// MIN_NOTIONAL/NOTIONAL filter declared.
    pub fn min_notional(&self, symbol: &str) -> Result<f64> {
        let info = self.get(symbol)?;
        Ok(info
            .min_notional
            .map(|d| d.to_f64().unwrap_or(0.0))
            .unwrap_or(0.0))
    }
}

// ---------------------------------------------------------------------------
// Parsing
// ---------------------------------------------------------------------------

fn parse_symbol(v: &serde_json::Value) -> Result<SymbolInfo> {
    let symbol = v["symbol"]
        .as_str()
        .ok_or_else(|| LiveError::Parse("symbol field missing".into()))?
        .to_string();
    let filters = v["filters"]
        .as_array()
        .ok_or_else(|| LiveError::Parse(format!("symbol {symbol} has no filters array")))?;

    let mut price_filter = None;
    let mut lot_size = None;
    let mut market_lot_size = None;
    let mut min_notional = None;

    for f in filters {
        let kind = f["filterType"].as_str().unwrap_or("");
        match kind {
            "PRICE_FILTER" => {
                price_filter = Some(PriceFilter {
                    tick_size: dec_field(f, "tickSize")?,
                    min_price: dec_field_opt(f, "minPrice"),
                    max_price: dec_field_opt(f, "maxPrice"),
                });
            }
            "LOT_SIZE" => {
                lot_size = Some(LotSize {
                    step_size: dec_field(f, "stepSize")?,
                    min_qty: dec_field(f, "minQty")?,
                    max_qty: dec_field_opt(f, "maxQty"),
                });
            }
            "MARKET_LOT_SIZE" => {
                market_lot_size = Some(LotSize {
                    step_size: dec_field(f, "stepSize")?,
                    min_qty: dec_field(f, "minQty")?,
                    max_qty: dec_field_opt(f, "maxQty"),
                });
            }
            "MIN_NOTIONAL" | "NOTIONAL" => {
                // Binance has used both `minNotional` and `notional` keys.
                let v = dec_field_opt(f, "minNotional").or_else(|| dec_field_opt(f, "notional"));
                if let Some(v) = v {
                    min_notional = Some(v);
                }
            }
            _ => {}
        }
    }

    Ok(SymbolInfo {
        symbol,
        price_filter,
        lot_size,
        market_lot_size,
        min_notional,
    })
}

fn dec_field(v: &serde_json::Value, key: &str) -> Result<Decimal> {
    let s = v[key]
        .as_str()
        .ok_or_else(|| LiveError::Parse(format!("filter field {key} missing or not string")))?;
    s.parse::<Decimal>()
        .map_err(|e| LiveError::Parse(format!("filter field {key}={s:?}: {e}")))
}

fn dec_field_opt(v: &serde_json::Value, key: &str) -> Option<Decimal> {
    v.get(key)
        .and_then(|x| x.as_str())
        .and_then(|s| s.parse::<Decimal>().ok())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    fn dec(s: &str) -> Decimal {
        Decimal::from_str(s).unwrap()
    }

    fn fixture() -> serde_json::Value {
        // Real-shape Binance USD-M exchangeInfo response, two symbols. BTCUSDT
        // has both LOT_SIZE and MARKET_LOT_SIZE so we can test the fallback;
        // ETHUSDT has only LOT_SIZE so use_market_filter falls through.
        serde_json::json!({
            "symbols": [
                {
                    "symbol": "BTCUSDT",
                    "filters": [
                        {"filterType": "PRICE_FILTER",     "tickSize": "0.10",   "minPrice": "0.10",   "maxPrice": "10000000"},
                        {"filterType": "LOT_SIZE",         "stepSize": "0.001",  "minQty": "0.001",   "maxQty": "1000"},
                        {"filterType": "MARKET_LOT_SIZE",  "stepSize": "0.01",   "minQty": "0.01",    "maxQty": "120"},
                        {"filterType": "MIN_NOTIONAL",     "notional": "5.0"}
                    ]
                },
                {
                    "symbol": "ETHUSDT",
                    "filters": [
                        {"filterType": "PRICE_FILTER",  "tickSize": "0.01",  "minPrice": "0.01"},
                        {"filterType": "LOT_SIZE",      "stepSize": "0.0001","minQty": "0.001"},
                        {"filterType": "NOTIONAL",      "minNotional": "10.0"}
                    ]
                }
            ]
        })
    }

    fn cache() -> ExchangeInfoCache {
        let c = ExchangeInfoCache::from_static(HashMap::new());
        c.populate_from_exchange_info(&fixture()).unwrap();
        c
    }

    #[test]
    fn parses_filters_into_decimal() {
        let c = cache();
        let btc = c.get("BTCUSDT").unwrap();
        assert_eq!(btc.symbol, "BTCUSDT");
        let pf = btc.price_filter.unwrap();
        assert_eq!(pf.tick_size, dec("0.10"));
        let lot = btc.lot_size.clone().unwrap();
        assert_eq!(lot.step_size, dec("0.001"));
        assert_eq!(lot.min_qty, dec("0.001"));
        let mlot = btc.market_lot_size.unwrap();
        assert_eq!(mlot.step_size, dec("0.01"));
        assert_eq!(mlot.min_qty, dec("0.01"));
        assert_eq!(btc.min_notional.unwrap(), dec("5.0"));
    }

    #[test]
    fn market_filter_takes_precedence_when_present() {
        let c = cache();
        // BTCUSDT has both LOT_SIZE (step 0.001) and MARKET_LOT_SIZE (step 0.01).
        // 0.0123 with market filter → 0.01 (rounded down)
        let mq = c.round_quantity_down("BTCUSDT", 0.0123, true).unwrap();
        assert!((mq - 0.01).abs() < 1e-12, "market round_down: got {mq}");
        // Same input with limit filter → 0.012
        let lq = c.round_quantity_down("BTCUSDT", 0.0123, false).unwrap();
        assert!((lq - 0.012).abs() < 1e-12, "limit round_down: got {lq}");
    }

    #[test]
    fn market_filter_falls_back_to_lot_size_when_absent() {
        let c = cache();
        // ETHUSDT has only LOT_SIZE — use_market_filter should fall through.
        let q = c.round_quantity_down("ETHUSDT", 0.12345678, true).unwrap();
        assert!((q - 0.1234).abs() < 1e-12, "got {q}");
    }

    #[test]
    fn round_down_versus_up_modes() {
        let c = cache();
        // BTCUSDT LOT_SIZE step = 0.001
        // 0.0019 down → 0.001
        // 0.0019 up   → 0.002
        let down = c.round_quantity_down("BTCUSDT", 0.0019, false).unwrap();
        let up = c.round_quantity_up("BTCUSDT", 0.0019, false).unwrap();
        assert!((down - 0.001).abs() < 1e-12, "down: got {down}");
        assert!((up - 0.002).abs() < 1e-12, "up: got {up}");
    }

    #[test]
    fn round_zero_or_negative_collapses_to_zero() {
        let c = cache();
        assert_eq!(c.round_quantity_down("BTCUSDT", 0.0, false).unwrap(), 0.0);
        assert_eq!(c.round_quantity_down("BTCUSDT", -1.0, false).unwrap(), 0.0);
    }

    #[test]
    fn round_below_step_floors_to_zero() {
        let c = cache();
        // 0.0001 with step 0.001 → 0
        let q = c.round_quantity_down("BTCUSDT", 0.0001, false).unwrap();
        assert_eq!(q, 0.0);
    }

    #[test]
    fn price_floors_toward_zero_at_tick() {
        let c = cache();
        // BTCUSDT tick = 0.10
        // 43250.7321 → floor((43250.7321 / 0.10)) * 0.10 = 432507 * 0.10 = 43250.70
        let p = c.round_price("BTCUSDT", 43250.7321).unwrap();
        assert!((p - 43250.7).abs() < 1e-9, "got {p}");
    }

    #[test]
    fn price_already_on_tick_is_idempotent() {
        let c = cache();
        // 43250.70 / 0.10 = 432507.0 exactly. floor = 432507. * 0.10 = 43250.7.
        let p = c.round_price("BTCUSDT", 43250.7).unwrap();
        assert!((p - 43250.7).abs() < 1e-9, "got {p}");
    }

    #[test]
    fn price_with_finer_tick() {
        let c = cache();
        // ETHUSDT tick = 0.01
        let p = c.round_price("ETHUSDT", 2614.367).unwrap();
        assert!((p - 2614.36).abs() < 1e-9, "got {p}");
    }

    #[test]
    fn min_qty_returned_per_filter_kind() {
        let c = cache();
        assert_eq!(c.min_qty("BTCUSDT", false).unwrap(), 0.001); // LOT_SIZE
        assert_eq!(c.min_qty("BTCUSDT", true).unwrap(), 0.01); // MARKET_LOT_SIZE
        assert_eq!(c.min_qty("ETHUSDT", true).unwrap(), 0.001); // falls back
    }

    #[test]
    fn min_notional_handles_both_keys() {
        let c = cache();
        assert_eq!(c.min_notional("BTCUSDT").unwrap(), 5.0); // "notional" key
        assert_eq!(c.min_notional("ETHUSDT").unwrap(), 10.0); // "minNotional" key
    }

    #[test]
    fn unknown_symbol_returns_typed_error() {
        let c = cache();
        let err = c.get("DOGEUSDT").unwrap_err();
        assert!(matches!(err, LiveError::UnknownSymbol(_)));
    }

    #[test]
    fn ensure_loaded_no_client_no_data_errors() {
        let c = ExchangeInfoCache::from_static(HashMap::new());
        let err = c.ensure_loaded("BTCUSDT").unwrap_err();
        assert!(matches!(err, LiveError::UnknownSymbol(_)));
    }
}
