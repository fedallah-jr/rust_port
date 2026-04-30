//! Regression test: the production `BinanceMarketClient` must bypass the
//! data crate's memory + disk caches so strategies always see fresh klines.
//!
//! Spins up a tiny_http mock server, points a real `BinanceMarketClient`
//! at it via `with_base_url`, calls `fetch_klines` twice for the same
//! parameters, and asserts that BOTH calls reached the server. With
//! caching enabled, the second call would be served from memory and never
//! issue an HTTP request — corrupting any strategy that polls for current
//! market state.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;

use chrono::{Duration as ChronoDuration, TimeZone, Utc};
use claude_trader_live::market_client::{BinanceMarketClient, LiveMarketClient};
use tiny_http::{Method, Response, Server};

#[test]
fn no_cache_market_client_hits_http_every_time() {
    // 1) Stand up a tiny_http server that counts requests.
    let server = Arc::new(Server::http("127.0.0.1:0").unwrap());
    let addr = server.server_addr().to_string();
    let counter = Arc::new(AtomicUsize::new(0));

    let server_clone = server.clone();
    let counter_clone = counter.clone();
    let handle = thread::spawn(move || {
        // Serve up to 8 requests then exit. Two fetch_klines calls may
        // each issue ≥1 HTTP request (pagination terminates immediately
        // on an empty array, so usually exactly one per call).
        for _ in 0..8 {
            let req = match server_clone.recv_timeout(std::time::Duration::from_secs(3)) {
                Ok(Some(r)) => r,
                _ => return,
            };
            counter_clone.fetch_add(1, Ordering::SeqCst);
            assert_eq!(req.method(), &Method::Get);
            // Path sanity: live polling routes through /fapi/v1/klines.
            assert!(
                req.url().starts_with("/fapi/v1/klines"),
                "expected /fapi/v1/klines path, got {}",
                req.url(),
            );
            // Empty array → fetch_klines completes without paginating.
            let resp = Response::from_string("[]")
                .with_header(
                    tiny_http::Header::from_bytes("content-type", "application/json").unwrap(),
                );
            let _ = req.respond(resp);
        }
    });

    // 2) Build a real BinanceMarketClient pointed at the mock and exercise
    //    the production code path. If cache_enabled drifted to true, the
    //    second fetch_klines would be served from memory and the counter
    //    would stay at 1 — the assert below catches the regression.
    let market = BinanceMarketClient::with_base_url(&format!("http://{addr}"));
    let start = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
    let end = Utc.with_ymd_and_hms(2024, 1, 2, 0, 0, 0).unwrap();
    market.fetch_klines("BTCUSDT", "1h", start, end).unwrap();
    market.fetch_klines("BTCUSDT", "1h", start, end).unwrap();

    drop(server); // unblock the recv loop
    let _ = handle.join();

    let n = counter.load(Ordering::SeqCst);
    assert!(
        n >= 2,
        "expected ≥2 HTTP requests for two fetch_klines calls, got {n}"
    );
}

/// Production wiring guard. If someone accidentally re-points
/// `BinanceMarketClient::new()` at the cached `BinanceClient::futures()`
/// constructor (rather than `no_cache`), strategies start seeing stale
/// klines silently. This test is the canary.
#[test]
fn production_binance_market_client_disables_cache() {
    let client = BinanceMarketClient::new();
    assert!(
        !client.cache_enabled(),
        "BinanceMarketClient must wire the no-cache BinanceClient; \
         got cache_enabled=true → strategies will see stale klines",
    );
}

/// Smoke test that the trait method resolves on the production type.
#[test]
fn binance_market_client_implements_live_market_trait() {
    let client = BinanceMarketClient::new();
    let _: &dyn LiveMarketClient = &client;
}

/// Verify the no-cache flag is wired correctly on the underlying client.
#[test]
fn data_client_no_cache_flag_disables_caching() {
    let cached = claude_trader_data::BinanceClient::futures();
    let no_cache = claude_trader_data::BinanceClient::no_cache(
        claude_trader_models::MarketType::Futures,
    );
    assert!(cached.cache_enabled());
    assert!(!no_cache.cache_enabled());
    let _ = ChronoDuration::seconds(1); // silence unused-import warning
    let _ = Utc::now();
}

/// Default `BinanceMarketClient::new()` must hit production fapi.
/// Default-construction is what every test stub and the previous-version
/// runners used, so any drift here is a silent regression.
#[test]
fn default_market_client_targets_production_fapi() {
    let client = BinanceMarketClient::new();
    assert_eq!(
        client.base_url(),
        "https://fapi.binance.com",
        "default BinanceMarketClient must target production fapi"
    );
}

/// Testnet wiring guard: `BinanceMarketClient::with_base_url(testnet_url)`
/// must route to demo-fapi rather than silently falling back to production.
/// Without this, a `BINANCE_TESTNET=1` run signs orders to demo-fapi but
/// pulls klines/funding from production — split-brain testnet validation.
#[test]
fn with_base_url_routes_market_data_to_testnet() {
    let demo = "https://demo-fapi.binance.com";
    let client = BinanceMarketClient::with_base_url(demo);
    assert_eq!(client.base_url(), demo);
    assert!(
        !client.cache_enabled(),
        "with_base_url must preserve the no-cache invariant"
    );
}

/// Custom base URL with a trailing slash gets normalized so the path
/// templates (which carry leading `/`) don't double-slash.
#[test]
fn with_base_url_strips_trailing_slash() {
    let client = BinanceMarketClient::with_base_url("https://example.com/");
    assert_eq!(client.base_url(), "https://example.com");
}

/// Empty base URL falls back to the production default rather than
/// producing a URL like `/fapi/v1/...` that would resolve to localhost.
#[test]
fn with_base_url_empty_string_falls_back_to_default() {
    let client = BinanceMarketClient::with_base_url("");
    assert_eq!(client.base_url(), "https://fapi.binance.com");
}
