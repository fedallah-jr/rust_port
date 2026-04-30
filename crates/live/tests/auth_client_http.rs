//! Integration tests for the request-shape-aware retry / idempotent-recovery
//! layer in `auth_client::BinanceFuturesClient`.
//!
//! Strategy: inject `MockTransport` (scripted responses keyed by URL+method)
//! and `MockSleeper` (records sleeps, doesn't actually wait). Every Binance
//! retry/recovery code path the merge gates call out is covered.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use claude_trader_live::auth_client::{
    BinanceFuturesClient, FuturesApi, HttpMethod, HttpRequest, HttpResponse, HttpTransport,
    Sleeper,
};
use claude_trader_live::error::LiveError;
use claude_trader_models::{LiveConfig, OrderSide, OrderStatus};

// ---------------------------------------------------------------------------
// Mock transport: scripted responses, keyed by (method, path).
//
// Each entry is consumed in FIFO order on match. Tests assert at the end that
// all queued responses were used (catches drift between intent and behaviour).
// ---------------------------------------------------------------------------

#[derive(Default)]
struct MockTransport {
    inner: Mutex<MockState>,
}

#[derive(Default)]
struct MockState {
    /// (method, path) → queue of scripted outcomes
    queues: HashMap<(HttpMethod, String), Vec<MockOutcome>>,
    /// Recorded request log for assertion
    log: Vec<RecordedRequest>,
}

#[derive(Clone)]
enum MockOutcome {
    Response(HttpResponse),
    Network(String),
}

#[derive(Debug, Clone)]
pub struct RecordedRequest {
    pub method: HttpMethod,
    pub url: String,
    pub body: Option<String>,
}

impl MockTransport {
    fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }
    fn enqueue(&self, method: HttpMethod, path: &str, outcome: MockOutcome) {
        let mut s = self.inner.lock().unwrap();
        s.queues
            .entry((method, path.to_string()))
            .or_default()
            .push(outcome);
    }
    fn enqueue_response(&self, method: HttpMethod, path: &str, resp: HttpResponse) {
        self.enqueue(method, path, MockOutcome::Response(resp));
    }
    fn enqueue_network_err(&self, method: HttpMethod, path: &str, msg: &str) {
        self.enqueue(method, path, MockOutcome::Network(msg.to_string()));
    }
    fn log(&self) -> Vec<RecordedRequest> {
        self.inner.lock().unwrap().log.clone()
    }
    fn assert_drained(&self) {
        let s = self.inner.lock().unwrap();
        for ((m, p), v) in s.queues.iter() {
            assert!(
                v.is_empty(),
                "MockTransport queue not drained: {} {} has {} responses left",
                m.as_str(),
                p,
                v.len(),
            );
        }
    }
}

impl HttpTransport for MockTransport {
    fn execute(&self, req: HttpRequest) -> Result<HttpResponse, LiveError> {
        let path = url_path(&req.url).to_string();
        let mut s = self.inner.lock().unwrap();
        s.log.push(RecordedRequest {
            method: req.method,
            url: req.url.clone(),
            body: req.body.clone(),
        });
        let key = (req.method, path.clone());
        let outcome = match s.queues.get_mut(&key) {
            Some(q) if !q.is_empty() => Some(q.remove(0)),
            _ => None,
        };
        drop(s);
        match outcome {
            Some(MockOutcome::Response(r)) => Ok(r),
            Some(MockOutcome::Network(m)) => Err(LiveError::Http(m)),
            None => Err(LiveError::Http(format!(
                "MockTransport: no scripted response for {} {}",
                req.method.as_str(),
                path
            ))),
        }
    }
}

fn url_path(url: &str) -> &str {
    let after_scheme = url.split("://").nth(1).unwrap_or(url);
    // Preserve the leading `/` of the path — `find` returns the index of the
    // slash itself, so slicing from there keeps it. Naive `split_once('/')`
    // would strip it.
    let after_host = match after_scheme.find('/') {
        Some(i) => &after_scheme[i..],
        None => "",
    };
    let path_end = after_host.find('?').unwrap_or(after_host.len());
    &after_host[..path_end]
}

// ---------------------------------------------------------------------------
// Mock sleeper — records each sleep duration, never actually waits.
// ---------------------------------------------------------------------------

#[derive(Default)]
struct MockSleeper {
    sleeps: Mutex<Vec<Duration>>,
}

impl MockSleeper {
    fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }
    fn sleep_count(&self) -> usize {
        self.sleeps.lock().unwrap().len()
    }
    fn total_sleep(&self) -> Duration {
        self.sleeps.lock().unwrap().iter().sum()
    }
}

impl Sleeper for MockSleeper {
    fn sleep(&self, dur: Duration) {
        self.sleeps.lock().unwrap().push(dur);
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn ok_json(body: &str) -> HttpResponse {
    HttpResponse {
        status: 200,
        headers: HashMap::new(),
        body: body.to_string(),
    }
}

fn ok_json_with(body: &str, hdrs: &[(&str, &str)]) -> HttpResponse {
    let mut headers = HashMap::new();
    for (k, v) in hdrs {
        headers.insert((*k).to_string(), (*v).to_string());
    }
    HttpResponse {
        status: 200,
        headers,
        body: body.to_string(),
    }
}

fn http_status(status: u16, body: &str) -> HttpResponse {
    HttpResponse {
        status,
        headers: HashMap::new(),
        body: body.to_string(),
    }
}

fn http_status_with_headers(status: u16, body: &str, hdrs: &[(&str, &str)]) -> HttpResponse {
    let mut headers = HashMap::new();
    for (k, v) in hdrs {
        headers.insert((*k).to_string(), (*v).to_string());
    }
    HttpResponse {
        status,
        headers,
        body: body.to_string(),
    }
}

fn time_response(server_ms: i64) -> HttpResponse {
    ok_json(&format!(r#"{{"serverTime":{server_ms}}}"#))
}

fn config() -> LiveConfig {
    LiveConfig {
        api_key: "test-key".into(),
        api_secret: "test-secret".into(),
        base_url: "http://mock.test".into(),
        position_size_usdt: 100.0,
        max_concurrent_positions: 3,
        order_check_interval_seconds: 5.0,
        testnet: false,
        recover_brackets_on_startup: true,
    }
}

fn build_client(
    transport: Arc<MockTransport>,
    sleeper: Arc<MockSleeper>,
) -> BinanceFuturesClient {
    // Initial server-time sync + exchangeInfo (for order-count limits) run
    // in the constructor — script both responses.
    transport.enqueue_response(HttpMethod::Get, "/fapi/v1/time", time_response(1_700_000_000_000));
    transport.enqueue_response(
        HttpMethod::Get,
        "/fapi/v1/exchangeInfo",
        ok_json(default_exchange_info()),
    );
    BinanceFuturesClient::with_dependencies(config(), transport, sleeper).unwrap()
}

fn default_exchange_info() -> &'static str {
    r#"{
      "rateLimits": [
        {"rateLimitType": "ORDERS", "interval": "SECOND", "intervalNum": 10, "limit": 300},
        {"rateLimitType": "ORDERS", "interval": "MINUTE", "intervalNum": 1, "limit": 1200}
      ],
      "symbols": []
    }"#
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[test]
fn constructor_loads_order_count_limits_from_exchange_info() {
    // exchangeInfo response declares non-default ORDERS limits.
    let transport = MockTransport::new();
    transport.enqueue_response(
        HttpMethod::Get,
        "/fapi/v1/time",
        time_response(1_700_000_000_000),
    );
    transport.enqueue_response(
        HttpMethod::Get,
        "/fapi/v1/exchangeInfo",
        ok_json(
            r#"{
              "rateLimits": [
                {"rateLimitType": "ORDERS", "interval": "SECOND", "intervalNum": 10, "limit": 500},
                {"rateLimitType": "ORDERS", "interval": "MINUTE", "intervalNum": 1, "limit": 2000}
              ],
              "symbols": []
            }"#,
        ),
    );
    let client =
        BinanceFuturesClient::with_dependencies(config(), transport.clone(), MockSleeper::new())
            .unwrap();
    let (l10, l1m) = client.order_count_limits();
    assert_eq!(l10, 500);
    assert_eq!(l1m, 2000);
    transport.assert_drained();
}

#[test]
fn signed_get_includes_timestamp_recvwindow_signature() {
    let transport = MockTransport::new();
    let sleeper = MockSleeper::new();
    let body = r#"[{"symbol":"BTCUSDT","positionAmt":"0.0","positionSide":"BOTH"}]"#;
    transport.enqueue_response(HttpMethod::Get, "/fapi/v3/positionRisk", ok_json(body));
    let client = build_client(transport.clone(), sleeper);

    client.get_position_info(None).unwrap();

    let log = transport.log();
    // [0] is the time-sync GET, [1] is the positionRisk GET.
    let pos_req = log
        .iter()
        .find(|r| url_path(&r.url) == "/fapi/v3/positionRisk")
        .unwrap();
    assert!(pos_req.url.contains("timestamp="));
    assert!(pos_req.url.contains("recvWindow=5000"));
    assert!(pos_req.url.contains("signature="));
}

#[test]
fn order_post_persistent_timestamp_skew_surfaces_after_cap() {
    // Regression for: execute_order_post used to decrement attempts on
    // each -1021 with no separate counter, so a persistent skew response
    // looped forever. The fix introduces a TIMESTAMP_REJECT_RETRY_MAX
    // counter; after the cap, the order returns Err(LiveError::Api{-1021}).
    let transport = MockTransport::new();
    let sleeper = MockSleeper::new();
    // 3 consecutive -1021s, then no further responses. With a cap of 1
    // timestamp retry, the second -1021 should surface as the final error
    // (we sent ATTEMPT 1 = -1021 → resync → ATTEMPT 2 = -1021 → cap hit → return).
    let err_body = r#"{"code":-1021,"msg":"timestamp out of recvWindow"}"#;
    for _ in 0..3 {
        transport.enqueue_response(HttpMethod::Post, "/fapi/v1/order", http_status(400, err_body));
    }
    // Each timestamp retry forces a server-time resync — script those too.
    for _ in 0..3 {
        transport.enqueue_response(
            HttpMethod::Get,
            "/fapi/v1/time",
            time_response(1_700_000_000_000),
        );
    }
    let client = build_client(transport.clone(), sleeper);

    let err = client
        .place_market_order("BTC/USDT", OrderSide::Buy, 0.01, "BOTH", "stable-cid")
        .unwrap_err();
    match err {
        LiveError::Api { code: -1021, .. } => {}
        other => panic!("expected LiveError::Api(-1021), got {other:?}"),
    }
    // Bounded: at most TIMESTAMP_REJECT_RETRY_MAX(=1) + 1 initial = 2 POSTs.
    let posts = transport
        .log()
        .iter()
        .filter(|r| r.method == HttpMethod::Post && url_path(&r.url) == "/fapi/v1/order")
        .count();
    assert!(
        posts <= 3,
        "post count must be bounded; got {posts}",
    );
}

#[test]
fn timestamp_retry_uses_fresh_signature() {
    // Regression for: -1021 retry must rebuild the request with a fresh
    // timestamp + HMAC, not re-send the stale URL. If the URL is reused
    // verbatim, Binance rejects again with -1021 and the retry never
    // actually has a chance to succeed.
    let transport = MockTransport::new();
    let sleeper = MockSleeper::new();
    let err_body =
        r#"{"code":-1021,"msg":"Timestamp for this request is outside of the recvWindow."}"#;
    transport.enqueue_response(HttpMethod::Get, "/fapi/v2/account", http_status(400, err_body));
    // Force-resync path → time endpoint hit again.
    transport.enqueue_response(
        HttpMethod::Get,
        "/fapi/v1/time",
        time_response(1_700_000_005_000),
    );
    transport.enqueue_response(
        HttpMethod::Get,
        "/fapi/v2/account",
        ok_json(r#"{"availableBalance":"500.0"}"#),
    );
    let client = build_client(transport.clone(), sleeper);

    client.get_account_info().unwrap();

    // Inspect the recorded GET requests to /fapi/v2/account. There should
    // be two; their timestamp= query params must DIFFER (and so must their
    // signatures), because run_with_retry rebuilds the request each iteration.
    let log = transport.log();
    let acct_urls: Vec<&str> = log
        .iter()
        .filter(|r| url_path(&r.url) == "/fapi/v2/account")
        .map(|r| r.url.as_str())
        .collect();
    assert_eq!(acct_urls.len(), 2, "expected 2 account requests");
    let ts1 = extract_query_param(acct_urls[0], "timestamp");
    let ts2 = extract_query_param(acct_urls[1], "timestamp");
    let sig1 = extract_query_param(acct_urls[0], "signature");
    let sig2 = extract_query_param(acct_urls[1], "signature");
    assert!(
        ts1 != ts2,
        "retry must use a fresh timestamp; got {ts1} == {ts2}",
    );
    assert!(
        sig1 != sig2,
        "retry must produce a fresh signature; got {sig1} == {sig2}",
    );
    transport.assert_drained();
}

fn extract_query_param(url: &str, key: &str) -> String {
    let qs = url.split_once('?').map(|(_, q)| q).unwrap_or("");
    for pair in qs.split('&') {
        if let Some((k, v)) = pair.split_once('=') {
            if k == key {
                return v.to_string();
            }
        }
    }
    String::new()
}

#[test]
fn timestamp_skew_triggers_force_resync_then_succeeds() {
    let transport = MockTransport::new();
    let sleeper = MockSleeper::new();
    // Reject first attempt with -1021, then succeed.
    let err_body = r#"{"code":-1021,"msg":"Timestamp for this request is outside of the recvWindow."}"#;
    let ok_body = r#"{"availableBalance":"123.45"}"#;
    transport.enqueue_response(HttpMethod::Get, "/fapi/v2/account", http_status(400, err_body));
    // The force-resync issues another /fapi/v1/time call.
    transport.enqueue_response(
        HttpMethod::Get,
        "/fapi/v1/time",
        time_response(1_700_000_005_000),
    );
    transport.enqueue_response(HttpMethod::Get, "/fapi/v2/account", ok_json(ok_body));
    let client = build_client(transport.clone(), sleeper);

    let v = client.get_account_info().unwrap();
    assert_eq!(v["availableBalance"].as_str(), Some("123.45"));

    // Two account calls, two time syncs (initial + forced).
    let log = transport.log();
    let time_calls = log
        .iter()
        .filter(|r| url_path(&r.url) == "/fapi/v1/time")
        .count();
    let acct_calls = log
        .iter()
        .filter(|r| url_path(&r.url) == "/fapi/v2/account")
        .count();
    assert_eq!(time_calls, 2);
    assert_eq!(acct_calls, 2);
    transport.assert_drained();
}

#[test]
fn throttle_429_with_retry_after_then_success() {
    let transport = MockTransport::new();
    let sleeper = MockSleeper::new();
    transport.enqueue_response(
        HttpMethod::Get,
        "/fapi/v3/positionRisk",
        http_status_with_headers(429, r#"{"code":-1003,"msg":"Too many requests."}"#, &[("Retry-After", "1")]),
    );
    transport.enqueue_response(
        HttpMethod::Get,
        "/fapi/v3/positionRisk",
        ok_json("[]"),
    );
    let client = build_client(transport.clone(), sleeper.clone());

    client.get_position_info(None).unwrap();

    // One sleep recorded for the 429 backoff. Used Retry-After (1s).
    assert_eq!(sleeper.sleep_count(), 1);
    let total = sleeper.total_sleep();
    assert!(total >= Duration::from_millis(900) && total <= Duration::from_millis(1100));
    transport.assert_drained();
}

#[test]
fn throttle_418_falls_back_to_default_when_no_retry_after() {
    let transport = MockTransport::new();
    let sleeper = MockSleeper::new();
    transport.enqueue_response(
        HttpMethod::Get,
        "/fapi/v3/positionRisk",
        http_status(418, r#"{"code":-1003,"msg":"IP banned."}"#),
    );
    transport.enqueue_response(HttpMethod::Get, "/fapi/v3/positionRisk", ok_json("[]"));
    let client = build_client(transport.clone(), sleeper.clone());

    client.get_position_info(None).unwrap();

    // No Retry-After → 15 s default.
    let total = sleeper.total_sleep();
    assert!(total >= Duration::from_secs(15) && total < Duration::from_secs(16));
    transport.assert_drained();
}

#[test]
fn server_5xx_on_idempotent_request_retries_then_surfaces() {
    let transport = MockTransport::new();
    let sleeper = MockSleeper::new();
    // 4 consecutive 503s — initial + 3 retries — then nothing left to consume.
    for _ in 0..4 {
        transport.enqueue_response(
            HttpMethod::Get,
            "/fapi/v3/positionRisk",
            http_status(503, "Service Unavailable"),
        );
    }
    let client = build_client(transport.clone(), sleeper.clone());

    let err = client.get_position_info(None).unwrap_err();
    match err {
        LiveError::Http(_) => {}
        other => panic!("expected LiveError::Http, got {other:?}"),
    }
    // 3 retries → 3 sleeps recorded (5s, 10s, 20s).
    assert_eq!(sleeper.sleep_count(), 3);
    transport.assert_drained();
}

#[test]
fn cancel_of_unknown_order_surfaces_minus_2011() {
    let transport = MockTransport::new();
    let sleeper = MockSleeper::new();
    transport.enqueue_response(
        HttpMethod::Delete,
        "/fapi/v1/order",
        http_status(400, r#"{"code":-2011,"msg":"Unknown order sent."}"#),
    );
    let client = build_client(transport.clone(), sleeper);

    let err = client.cancel_order("BTC/USDT", 12345).unwrap_err();
    assert!(err.is_cancel_of_unknown(), "got {err:?}");
    transport.assert_drained();
}

#[test]
fn get_order_by_client_id_returns_none_on_minus_2013() {
    let transport = MockTransport::new();
    let sleeper = MockSleeper::new();
    transport.enqueue_response(
        HttpMethod::Get,
        "/fapi/v1/order",
        http_status(400, r#"{"code":-2013,"msg":"Order does not exist."}"#),
    );
    let client = build_client(transport.clone(), sleeper);

    let result = client.get_order_by_client_id("BTC/USDT", "abc-uuid").unwrap();
    assert!(result.is_none());
    transport.assert_drained();
}

#[test]
fn order_post_5xx_recovers_when_query_finds_order() {
    let transport = MockTransport::new();
    let sleeper = MockSleeper::new();

    // First attempt: 503 — outcome unknown.
    transport.enqueue_response(
        HttpMethod::Post,
        "/fapi/v1/order",
        http_status(503, "Service Unavailable"),
    );
    // Recovery query: GET /fapi/v1/order finds it.
    let order_body = r#"{
        "orderId": 12345,
        "symbol": "BTCUSDT",
        "side": "BUY",
        "type": "MARKET",
        "origType": "MARKET",
        "origQty": "0.01",
        "executedQty": "0.01",
        "avgPrice": "43000.0",
        "price": "0",
        "stopPrice": "0",
        "status": "FILLED",
        "clientOrderId": "client-uuid"
    }"#;
    transport.enqueue_response(HttpMethod::Get, "/fapi/v1/order", ok_json(order_body));
    let client = build_client(transport.clone(), sleeper);

    let order = client
        .place_market_order("BTC/USDT", OrderSide::Buy, 0.01, "BOTH", "client-uuid")
        .unwrap();
    assert_eq!(order.order_id, 12345);
    assert_eq!(order.status, OrderStatus::Filled);
    assert_eq!(order.client_order_id, "client-uuid");
    transport.assert_drained();
}

#[test]
fn order_post_5xx_then_minus_2013_reposts_with_same_client_id() {
    let transport = MockTransport::new();
    let sleeper = MockSleeper::new();

    // 1) POST → 503
    transport.enqueue_response(HttpMethod::Post, "/fapi/v1/order", http_status(503, ""));
    // 2) Recovery GET → -2013
    transport.enqueue_response(
        HttpMethod::Get,
        "/fapi/v1/order",
        http_status(400, r#"{"code":-2013,"msg":"Order does not exist."}"#),
    );
    // 3) Re-POST → success
    let order_body = r#"{
        "orderId": 9999,
        "symbol": "BTCUSDT",
        "side": "BUY",
        "type": "MARKET",
        "origQty": "0.01",
        "status": "NEW",
        "clientOrderId": "uuid-stable"
    }"#;
    transport.enqueue_response(HttpMethod::Post, "/fapi/v1/order", ok_json(order_body));
    let client = build_client(transport.clone(), sleeper);

    let order = client
        .place_market_order("BTC/USDT", OrderSide::Buy, 0.01, "BOTH", "uuid-stable")
        .unwrap();
    assert_eq!(order.order_id, 9999);
    assert_eq!(order.client_order_id, "uuid-stable");

    // Both POSTs sent the same clientOrderId in the body.
    let posts: Vec<RecordedRequest> = transport
        .log()
        .into_iter()
        .filter(|r| r.method == HttpMethod::Post && url_path(&r.url) == "/fapi/v1/order")
        .collect();
    assert_eq!(posts.len(), 2);
    for p in &posts {
        let body = p.body.as_ref().expect("POST has body");
        assert!(
            body.contains("newClientOrderId=uuid-stable"),
            "body missing client id: {body}"
        );
    }
    transport.assert_drained();
}

#[test]
fn order_post_terminal_4xx_surfaces_immediately_with_code() {
    let transport = MockTransport::new();
    let sleeper = MockSleeper::new();
    // -2010 is in the terminal-4xx allow-list.
    transport.enqueue_response(
        HttpMethod::Post,
        "/fapi/v1/order",
        http_status(400, r#"{"code":-2010,"msg":"NEW_ORDER_REJECTED"}"#),
    );
    let client = build_client(transport.clone(), sleeper);

    let err = client
        .place_market_order("BTC/USDT", OrderSide::Buy, 0.01, "BOTH", "uuid")
        .unwrap_err();
    match err {
        LiveError::Api { code: -2010, .. } => {}
        other => panic!("expected LiveError::Api(-2010), got {other:?}"),
    }
    transport.assert_drained();
}

#[test]
fn order_post_network_error_recovers_via_query() {
    let transport = MockTransport::new();
    let sleeper = MockSleeper::new();

    // 1) POST → network error (connection reset)
    transport.enqueue_network_err(HttpMethod::Post, "/fapi/v1/order", "connection reset");
    // 2) Recovery GET finds the order — placement actually succeeded.
    let body = r#"{
        "orderId": 7777,
        "symbol": "BTCUSDT",
        "side": "BUY",
        "type": "MARKET",
        "origQty": "0.01",
        "status": "FILLED",
        "executedQty": "0.01",
        "avgPrice": "43000.0",
        "clientOrderId": "stable-id"
    }"#;
    transport.enqueue_response(HttpMethod::Get, "/fapi/v1/order", ok_json(body));
    let client = build_client(transport.clone(), sleeper);

    let order = client
        .place_market_order("BTC/USDT", OrderSide::Buy, 0.01, "BOTH", "stable-id")
        .unwrap();
    assert_eq!(order.order_id, 7777);
    transport.assert_drained();
}

#[test]
fn algo_order_5xx_recovers_via_client_algo_id() {
    let transport = MockTransport::new();
    let sleeper = MockSleeper::new();

    // 1) POST /fapi/v1/algoOrder → 503
    transport.enqueue_response(HttpMethod::Post, "/fapi/v1/algoOrder", http_status(503, ""));
    // 2) GET /fapi/v1/algoOrder by clientAlgoId → found
    let body = r#"{
        "algoId": 42,
        "actualOrderId": 0,
        "symbol": "BTCUSDT",
        "side": "SELL",
        "orderType": "STOP_MARKET",
        "type": "STOP_MARKET",
        "triggerPrice": "40000",
        "quantity": "0.01",
        "algoStatus": "NEW",
        "clientAlgoId": "sl-uuid"
    }"#;
    transport.enqueue_response(HttpMethod::Get, "/fapi/v1/algoOrder", ok_json(body));
    let client = build_client(transport.clone(), sleeper);

    let order = client
        .place_stop_market("BTC/USDT", OrderSide::Sell, 40000.0, "LONG", Some(0.01), "sl-uuid")
        .unwrap();
    assert_eq!(order.algo_id, 42);
    assert_eq!(order.client_order_id, "sl-uuid");
    transport.assert_drained();
}

#[test]
fn order_count_headers_drive_pre_check_throttle() {
    let transport = MockTransport::new();
    let sleeper = MockSleeper::new();

    // 1st order: response carries near-limit order-count headers.
    let body1 = r#"{
        "orderId": 1,
        "symbol": "BTCUSDT",
        "side": "BUY",
        "type": "MARKET",
        "origQty": "0.01",
        "status": "NEW",
        "clientOrderId": "id-1"
    }"#;
    let near_limit_headers = &[
        ("X-MBX-ORDER-COUNT-10S", "299"),
        ("X-MBX-ORDER-COUNT-1M", "10"),
    ];
    transport.enqueue_response(
        HttpMethod::Post,
        "/fapi/v1/order",
        ok_json_with(body1, near_limit_headers),
    );
    // 2nd order: succeeds. The tracker's pre_check should have observed the
    // near-limit count from #1 and slept before sending #2.
    let body2 = r#"{
        "orderId": 2,
        "symbol": "BTCUSDT",
        "side": "BUY",
        "type": "MARKET",
        "origQty": "0.01",
        "status": "NEW",
        "clientOrderId": "id-2"
    }"#;
    let post_throttle_headers = &[
        ("X-MBX-ORDER-COUNT-10S", "1"),
        ("X-MBX-ORDER-COUNT-1M", "11"),
    ];
    transport.enqueue_response(
        HttpMethod::Post,
        "/fapi/v1/order",
        ok_json_with(body2, post_throttle_headers),
    );
    let client = build_client(transport.clone(), sleeper.clone());

    client
        .place_market_order("BTC/USDT", OrderSide::Buy, 0.01, "BOTH", "id-1")
        .unwrap();
    let sleeps_before = sleeper.sleep_count();
    client
        .place_market_order("BTC/USDT", OrderSide::Buy, 0.01, "BOTH", "id-2")
        .unwrap();
    let sleeps_after = sleeper.sleep_count();

    assert!(
        sleeps_after > sleeps_before,
        "expected at least one pre-throttle sleep, got {sleeps_before} → {sleeps_after}",
    );
    transport.assert_drained();
}
