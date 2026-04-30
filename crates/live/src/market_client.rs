//! `LiveMarketClient` — the unsigned market-data surface strategies use.
//!
//! Mirrors `live/auth_client.LiveMarketClient`. Distinct from `FuturesApi`
//! (the signed/account surface owned by the engine + executor + tracker)
//! because:
//!   - market data is unsigned — no clock sync, no client-id idempotency
//!   - strategies should NOT be able to place orders or query account state
//!   - cache stance differs: `FuturesApi` reads exchangeInfo through the
//!     account-side caches; live strategies must observe fresh klines on
//!     every poll, so `BinanceMarketClient` is built with `BinanceClient::no_cache()`
//!     and a regression test pins that wiring (see `cache_enabled()`).
//!
//! Implementations:
//!   - `NullMarketClient` — zero-network fallback for the demo runner and
//!     tests that don't need real market data.
//!   - `BinanceMarketClient` — production client wrapping
//!     `claude_trader_data::BinanceClient::no_cache(MarketType::Futures)`.
//!     Provides both `fetch_klines` and `fetch_funding_rates`; the latter
//!     is required by strategies whose `required_context` declares
//!     `Funding(symbol)` keys (e.g. opus46).

use chrono::{DateTime, Utc};
use claude_trader_models::{Candle, FundingRate};

use crate::error::{LiveError, Result};

pub trait LiveMarketClient: Send + Sync {
    fn fetch_klines(
        &self,
        symbol: &str,
        interval: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<Candle>>;

    /// Fetch the historical funding rate series for a USD-M futures symbol
    /// over `[start, end)`. Default implementation returns an empty series
    /// — strategies that don't need funding (or test stubs) keep working.
    /// Production implementations override to call the futures funding-rate
    /// endpoint (Binance: `/fapi/v1/fundingRate`).
    fn fetch_funding_rates(
        &self,
        _symbol: &str,
        _start: DateTime<Utc>,
        _end: DateTime<Utc>,
    ) -> Result<Vec<FundingRate>> {
        Ok(Vec::new())
    }
}

/// No-op market client. Used by:
///   - the example runner's demo strategy that doesn't need klines
///   - tests that don't exercise market-data fetching
pub struct NullMarketClient;

impl LiveMarketClient for NullMarketClient {
    fn fetch_klines(
        &self,
        _: &str,
        _: &str,
        _: DateTime<Utc>,
        _: DateTime<Utc>,
    ) -> Result<Vec<Candle>> {
        Ok(Vec::new())
    }
}

/// Production market client wrapping `claude_trader_data::BinanceClient`
/// in **no-cache** mode. Every `fetch_klines` call hits HTTP, matching the
/// Python `LiveMarketClient` cache-bypass invariant. Without this, a
/// strategy polling for fresh candles can be served stale data from the
/// cache layers.
pub struct BinanceMarketClient {
    inner: claude_trader_data::BinanceClient,
}

impl BinanceMarketClient {
    pub fn new() -> Self {
        Self {
            inner: claude_trader_data::BinanceClient::no_cache(
                claude_trader_models::MarketType::Futures,
            ),
        }
    }

    /// Construct a market client that hits `base_url` instead of the
    /// production fapi endpoint. Live runners pass `LiveConfig.base_url`
    /// here so testnet (or any operator-supplied URL) is honored on the
    /// market-data side too — without this, a `BINANCE_TESTNET=1` run
    /// places orders on demo-fapi while making decisions from production
    /// klines and funding.
    ///
    /// Empty `base_url` falls back to the production default.
    pub fn with_base_url(base_url: &str) -> Self {
        Self {
            inner: claude_trader_data::BinanceClient::no_cache_with_base_url(
                claude_trader_models::MarketType::Futures,
                base_url,
            ),
        }
    }

    /// True iff the underlying `BinanceClient` consults memory + disk
    /// caches before HTTP. Live polling demands `false` so strategies
    /// always observe fresh klines; this accessor exists so a regression
    /// test can pin the wiring.
    pub fn cache_enabled(&self) -> bool {
        self.inner.cache_enabled()
    }

    /// Read-only accessor on the resolved base URL — exposed so a
    /// regression test can pin the testnet/custom wiring without making
    /// network calls.
    pub fn base_url(&self) -> &str {
        self.inner.base_url()
    }
}

impl Default for BinanceMarketClient {
    fn default() -> Self {
        Self::new()
    }
}

impl LiveMarketClient for BinanceMarketClient {
    fn fetch_klines(
        &self,
        symbol: &str,
        interval: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<Candle>> {
        match self.inner.fetch_klines(symbol, interval, start, end) {
            Ok(fetch) => Ok(fetch.into_rows()),
            Err(e) => Err(LiveError::Http(format!("fetch_klines: {e}"))),
        }
    }

    fn fetch_funding_rates(
        &self,
        symbol: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<FundingRate>> {
        self.inner
            .fetch_funding_rates(symbol, start, end)
            .map_err(|e| LiveError::Http(format!("fetch_funding_rates: {e}")))
    }
}
