//! Binance USD-M futures REST client.
//!
//! Mirrors `live/auth_client.py::BinanceFuturesClient`. The plan-v2 design
//! splits retry policy by *request shape* rather than by status code, so
//! the production gates set in §1 hold:
//!
//! - **Idempotency-safe** — `GET *`, `DELETE /fapi/v1/order`,
//!   `DELETE /fapi/v1/algoOrder`, `POST /fapi/v1/leverage`. Retried freely
//!   on throttle / 5xx / network. `cancel_order_safe` swallows `-2011`
//!   ("Unknown order sent") so re-cancels of already-canceled orders are
//!   harmless.
//! - **Idempotency-unsafe** — `POST /fapi/v1/order` and `POST
//!   /fapi/v1/algoOrder`. The caller generates a stable
//!   `newClientOrderId` / `clientAlgoId` *before* the first POST. On 5xx
//!   or network error we enter a recovery loop:
//!     1. Sleep with exponential backoff.
//!     2. Query Binance by client ID.
//!     3. If the order exists → return it (placement actually succeeded).
//!     4. If `-2013 "Order does not exist"` → re-POST with the *same*
//!        client ID (idempotent by construction).
//!     5. If query itself returns 5xx / network → still ambiguous, sleep
//!        and re-query — never re-POST until we have a definitive answer.
//!   Up to three attempts in total before surfacing the failure.
//!
//! Other plan-v2 invariants enforced here:
//!   - `recvWindow=5000` and a monotonic-anchored timestamp on every signed
//!     request (via `ServerTime`).
//!   - On `-1021` ("Timestamp for this request is outside of the
//!     recvWindow") force-resync server time and retry once.
//!   - Throttle codes `418` / `429` honour `Retry-After`, fall back to
//!     `15s → 30s → 60s … → 300s` exponential backoff. Loop forever (Python
//!     parity — better to wait it out than crash the engine).
//!   - `X-MBX-USED-WEIGHT-1m` headers feed the shared `RateLimiter`.
//!   - `X-MBX-ORDER-COUNT-{10S,1M}` headers feed `OrderCountTracker`, which
//!     `pre_check`s before every order POST.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use chrono::{DateTime, Utc};
use claude_trader_data::rate_limiter::RateLimiter;
use claude_trader_models::{
    dt_to_ms, ms_to_dt_opt, AccountTrade, ExchangeOrder, LiveConfig, OrderSide, OrderStatus,
    OrderType,
};
use hmac::{Hmac, Mac};
use sha2::Sha256;

use crate::error::{LiveError, Result};
use crate::order_count::OrderCountTracker;
use crate::time_sync::{ServerTime, ServerTimeFetcher, SystemClock};

type HmacSha256 = Hmac<Sha256>;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const RECV_WINDOW_MS: i64 = 5000;
const THROTTLE_BACKOFF_INITIAL_S: f64 = 15.0;
const THROTTLE_BACKOFF_MAX_S: f64 = 300.0;
const SERVER_ERROR_RETRY_MAX: u32 = 3;
const SERVER_ERROR_BACKOFF_INITIAL_S: f64 = 5.0;
const NETWORK_RETRY_MAX: u32 = 3;
const NETWORK_BACKOFF_INITIAL_S: f64 = 5.0;
const ORDER_RECOVERY_MAX_ATTEMPTS: u32 = 3;
const ORDER_RECOVERY_BACKOFF_INITIAL_S: f64 = 5.0;
const ORDER_RECOVERY_BACKOFF_MAX_S: f64 = 60.0;
const TIMESTAMP_REJECT_RETRY_MAX: u32 = 1;

const RATE_LIMIT_PER_MIN: u32 = 2400;

const RETRYABLE_THROTTLE_STATUSES: &[u16] = &[418, 429];

const HEADER_USED_WEIGHT_1M: &str = "X-MBX-USED-WEIGHT-1m";
const HEADER_RETRY_AFTER: &str = "Retry-After";

const PATH_TIME: &str = "/fapi/v1/time";
const PATH_ORDER: &str = "/fapi/v1/order";
const PATH_ALGO_ORDER: &str = "/fapi/v1/algoOrder";
const PATH_OPEN_ORDERS: &str = "/fapi/v1/openOrders";
const PATH_LEVERAGE: &str = "/fapi/v1/leverage";
const PATH_ACCOUNT: &str = "/fapi/v2/account";
const PATH_USER_TRADES: &str = "/fapi/v1/userTrades";
const PATH_POSITION_RISK: &str = "/fapi/v3/positionRisk";
const PATH_EXCHANGE_INFO: &str = "/fapi/v1/exchangeInfo";
const PATH_PREMIUM_INDEX: &str = "/fapi/v1/premiumIndex";

/// Binance error codes for "request was rejected before reaching the matching
/// engine". Plan v2 §1: when `submit_entry_order` sees these we mark the
/// position FAILED and skip the idempotent recovery loop.
pub const TERMINAL_4XX_CODES: &[i64] = &[
    -1013, // INVALID_PRICE / filter failure
    -1102, // MANDATORY_PARAM_EMPTY_OR_MALFORMED
    -1111, // BAD_PRECISION
    -1117, // INVALID_SIDE
    -2010, // NEW_ORDER_REJECTED — usually insufficient balance / margin
    -2019, // MARGIN_NOT_SUFFICIENT
    -4131, // LIMIT_ORDER_REJECT
    -4164, // MIN_NOTIONAL
];

// ---------------------------------------------------------------------------
// HTTP transport abstraction
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HttpMethod {
    Get,
    Post,
    Delete,
}

impl HttpMethod {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Get => "GET",
            Self::Post => "POST",
            Self::Delete => "DELETE",
        }
    }
}

/// What we hand to the transport layer. Pre-built so the transport doesn't
/// need to know anything about Binance signing.
pub struct HttpRequest {
    pub method: HttpMethod,
    pub url: String,
    pub headers: Vec<(String, String)>,
    /// Form-urlencoded body for POST/DELETE; `None` for GET.
    pub body: Option<String>,
}

/// Response carries the data the retry layer needs: status, headers, body.
/// Headers stored as `HashMap<String, String>` so test-time mocks can build
/// them without depending on `reqwest::header::HeaderMap`.
#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: String,
}

impl HttpResponse {
    pub fn header(&self, name: &str) -> Option<&str> {
        // Case-insensitive lookup — reqwest normalizes headers to lowercase
        // but tests may pass mixed-case.
        for (k, v) in &self.headers {
            if k.eq_ignore_ascii_case(name) {
                return Some(v);
            }
        }
        None
    }
}

pub trait HttpTransport: Send + Sync {
    fn execute(&self, req: HttpRequest) -> Result<HttpResponse>;
}

/// Production transport built on top of `reqwest::blocking::Client`.
pub struct ReqwestTransport {
    inner: reqwest::blocking::Client,
}

impl ReqwestTransport {
    pub fn new() -> Result<Self> {
        Ok(Self {
            inner: reqwest::blocking::Client::builder()
                .timeout(Duration::from_secs(15))
                .build()
                .map_err(|e| LiveError::Http(format!("client build: {e}")))?,
        })
    }
}

impl HttpTransport for ReqwestTransport {
    fn execute(&self, req: HttpRequest) -> Result<HttpResponse> {
        let mut builder = match req.method {
            HttpMethod::Get => self.inner.get(&req.url),
            HttpMethod::Post => self.inner.post(&req.url),
            HttpMethod::Delete => self.inner.delete(&req.url),
        };
        for (k, v) in &req.headers {
            builder = builder.header(k, v);
        }
        if let Some(body) = req.body {
            builder = builder
                .header("Content-Type", "application/x-www-form-urlencoded")
                .body(body);
        }
        let resp = builder
            .send()
            .map_err(|e| LiveError::Http(format!("send: {e}")))?;
        let status = resp.status().as_u16();
        let mut headers = HashMap::new();
        for (k, v) in resp.headers() {
            if let Ok(s) = v.to_str() {
                headers.insert(k.as_str().to_string(), s.to_string());
            }
        }
        let body = resp
            .text()
            .map_err(|e| LiveError::Http(format!("read body: {e}")))?;
        Ok(HttpResponse {
            status,
            headers,
            body,
        })
    }
}

// ---------------------------------------------------------------------------
// Sleep abstraction
// ---------------------------------------------------------------------------

pub trait Sleeper: Send + Sync {
    fn sleep(&self, dur: Duration);
}

pub struct ThreadSleeper;

impl Sleeper for ThreadSleeper {
    fn sleep(&self, dur: Duration) {
        std::thread::sleep(dur);
    }
}

// ---------------------------------------------------------------------------
// Server-time fetcher (HTTP impl; the trait lives in time_sync)
// ---------------------------------------------------------------------------

pub struct HttpServerTimeFetcher {
    base_url: String,
    transport: Arc<dyn HttpTransport>,
}

impl HttpServerTimeFetcher {
    pub fn new(base_url: String, transport: Arc<dyn HttpTransport>) -> Self {
        Self {
            base_url,
            transport,
        }
    }
}

impl ServerTimeFetcher for HttpServerTimeFetcher {
    fn fetch_ms(&self) -> Result<i64> {
        let url = format!("{}{}", self.base_url, PATH_TIME);
        let resp = self.transport.execute(HttpRequest {
            method: HttpMethod::Get,
            url,
            headers: vec![],
            body: None,
        })?;
        if resp.status != 200 {
            return Err(LiveError::Http(format!(
                "{} {}: {}",
                PATH_TIME, resp.status, resp.body
            )));
        }
        let v: serde_json::Value =
            serde_json::from_str(&resp.body).map_err(|e| LiveError::Parse(e.to_string()))?;
        v["serverTime"]
            .as_i64()
            .ok_or_else(|| LiveError::Parse("serverTime missing or not i64".into()))
    }
}

// ---------------------------------------------------------------------------
// FuturesApi trait
// ---------------------------------------------------------------------------

/// All Binance interactions the live runtime needs, behind a trait so the
/// tracker / executor / engine can be tested without HTTP. Production
/// implementation is `BinanceFuturesClient`.
pub trait FuturesApi: Send + Sync {
    fn server_now(&self) -> DateTime<Utc>;

    // -- Order placement ----------------------------------------------------
    // Caller-supplied client IDs are mandatory. The two-phase placement flow
    // (executor's `prepare_entry_position` → `submit_entry_order`) generates
    // and persists the ID before invoking these methods, so a 5xx or network
    // failure can be recovered idempotently.

    fn place_market_order(
        &self,
        symbol: &str,
        side: OrderSide,
        quantity: f64,
        position_side: &str,
        client_order_id: &str,
    ) -> Result<ExchangeOrder>;

    fn place_limit_order(
        &self,
        symbol: &str,
        side: OrderSide,
        quantity: f64,
        price: f64,
        position_side: &str,
        client_order_id: &str,
    ) -> Result<ExchangeOrder>;

    fn place_stop_market(
        &self,
        symbol: &str,
        side: OrderSide,
        stop_price: f64,
        position_side: &str,
        quantity: Option<f64>,
        client_algo_id: &str,
    ) -> Result<ExchangeOrder>;

    fn place_take_profit_market(
        &self,
        symbol: &str,
        side: OrderSide,
        stop_price: f64,
        position_side: &str,
        quantity: Option<f64>,
        client_algo_id: &str,
    ) -> Result<ExchangeOrder>;

    // -- Cancels ------------------------------------------------------------
    fn cancel_order(&self, symbol: &str, order_id: i64) -> Result<ExchangeOrder>;
    fn cancel_algo_order(&self, algo_id: i64) -> Result<()>;

    // -- Queries ------------------------------------------------------------
    fn get_order(&self, symbol: &str, order_id: i64) -> Result<ExchangeOrder>;

    /// Recovery hook for 5xx / network failures during `submit_entry_order`.
    /// `Ok(None)` means Binance returned `-2013 "Order does not exist"`.
    fn get_order_by_client_id(
        &self,
        symbol: &str,
        client_id: &str,
    ) -> Result<Option<ExchangeOrder>>;

    fn get_algo_order(&self, algo_id: i64) -> Result<ExchangeOrder>;

    /// Recovery hook for algo-order placements. `Ok(None)` means -2013.
    fn get_algo_order_by_client_id(
        &self,
        client_id: &str,
    ) -> Result<Option<ExchangeOrder>>;

    fn get_open_orders(&self, symbol: Option<&str>) -> Result<Vec<ExchangeOrder>>;

    // -- Account ------------------------------------------------------------
    fn get_position_info(&self, symbol: Option<&str>) -> Result<Vec<serde_json::Value>>;

    fn get_account_trades(
        &self,
        symbol: &str,
        start: Option<DateTime<Utc>>,
        end: Option<DateTime<Utc>>,
        order_id: Option<i64>,
        limit: usize,
    ) -> Result<Vec<AccountTrade>>;

    fn get_account_info(&self) -> Result<serde_json::Value>;
    fn get_available_balance(&self) -> Result<f64>;
    fn set_leverage(&self, symbol: &str, leverage: u32) -> Result<()>;

    // -- Market data (unsigned) --------------------------------------------
    fn get_exchange_info(&self) -> Result<serde_json::Value>;
    fn get_mark_price(&self, symbol: &str) -> Result<f64>;
}

// ---------------------------------------------------------------------------
// BinanceFuturesClient
// ---------------------------------------------------------------------------

pub struct BinanceFuturesClient {
    base_url: String,
    api_key: String,
    api_secret: Vec<u8>,
    transport: Arc<dyn HttpTransport>,
    sleeper: Arc<dyn Sleeper>,
    server_time: ServerTime,
    rate_limiter: Arc<RateLimiter>,
    order_count: OrderCountTracker,
}

impl BinanceFuturesClient {
    /// Production constructor: real HTTP, real sleep, production server-time.
    /// Eagerly syncs with Binance so connectivity errors surface at startup.
    pub fn new(config: LiveConfig) -> Result<Self> {
        let transport: Arc<dyn HttpTransport> = Arc::new(ReqwestTransport::new()?);
        let sleeper: Arc<dyn Sleeper> = Arc::new(ThreadSleeper);
        Self::with_dependencies(config, transport, sleeper)
    }

    /// Test/integration constructor: caller controls transport + sleep.
    pub fn with_dependencies(
        config: LiveConfig,
        transport: Arc<dyn HttpTransport>,
        sleeper: Arc<dyn Sleeper>,
    ) -> Result<Self> {
        let fetcher = Arc::new(HttpServerTimeFetcher::new(
            config.base_url.clone(),
            transport.clone(),
        ));
        let server_time = ServerTime::new(fetcher, Arc::new(SystemClock::new()));
        let rate_limiter = Arc::new(RateLimiter::new(RATE_LIMIT_PER_MIN));
        let order_count = OrderCountTracker::new();
        let client = Self {
            base_url: config.base_url.clone(),
            api_key: config.api_key.clone(),
            api_secret: config.api_secret.into_bytes(),
            transport,
            sleeper,
            server_time,
            rate_limiter,
            order_count,
        };
        // Eager sync. Failure logs to stderr (ServerTime swallows it) so the
        // engine can still start in offline-test environments.
        client.server_time.ensure_synced(true);
        // Load actual order-count limits from `exchangeInfo.rateLimits`.
        // Best-effort: defaults remain in place if the fetch or parse fails.
        if let Err(e) = client.refresh_order_count_limits() {
            eprintln!(
                "WARN: could not load order-count limits from exchangeInfo: {e}; \
                 falling back to documented defaults.",
            );
        }
        Ok(client)
    }

    /// Fetch `exchangeInfo` and update `OrderCountTracker` with the
    /// venue-reported `ORDERS` rate limits. Looks for entries with
    /// `rateLimitType == "ORDERS"` and intervals (10s, 1m).
    fn refresh_order_count_limits(&self) -> Result<()> {
        let info = self.get_exchange_info()?;
        let (limit_10s, limit_1m) = parse_order_rate_limits(&info)
            .ok_or_else(|| LiveError::Parse("exchangeInfo.rateLimits missing ORDERS entries".into()))?;
        self.order_count.set_limits(limit_10s, limit_1m);
        Ok(())
    }

    /// Test/inspection accessor: current order-count limits as `(10s, 1m)`.
    pub fn order_count_limits(&self) -> (u32, u32) {
        self.order_count.limits()
    }

    // ----- Signing primitives ---------------------------------------------

    fn sign(&self, message: &str) -> String {
        let mut mac =
            HmacSha256::new_from_slice(&self.api_secret).expect("HMAC accepts any key length");
        mac.update(message.as_bytes());
        hex::encode(mac.finalize().into_bytes())
    }

    /// Append `timestamp` and `recvWindow`, build the canonical query, sign
    /// it, append `signature`. Returns `(query_string, signed_params)` where
    /// `signed_params` mirrors what we'd put on the wire — identical to the
    /// query string the signature was computed over plus `signature=…`.
    fn build_signed_query(&self, params: &mut Vec<(String, String)>) -> String {
        self.server_time.ensure_synced(false);
        let ts = self.server_time.now_ms();
        params.push(("timestamp".to_string(), ts.to_string()));
        params.push(("recvWindow".to_string(), RECV_WINDOW_MS.to_string()));
        let unsigned = build_query(params);
        let signature = self.sign(&unsigned);
        format!("{unsigned}&signature={signature}")
    }

    fn build_unsigned_query(&self, params: &[(String, String)]) -> String {
        build_query(params)
    }

    // ----- Throttle / 5xx / network retry layer ---------------------------

    /// Family-1 retry for idempotent requests. The caller passes a closure
    /// that builds the `HttpRequest` fresh on every iteration, so signed
    /// requests can re-sign with a current timestamp + signature on each
    /// retry. Without this, a `-1021` rejection forces a server-time
    /// resync but the retried request still carries the stale timestamp
    /// and fails identically.
    ///
    /// Loop semantics:
    ///   - 2xx → return Ok
    ///   - 418/429 → sleep `Retry-After` (or backoff cap), retry forever
    ///   - 5xx → sleep `5 · 2ⁿ`, retry up to 3
    ///   - network error → sleep `5 · 2ⁿ`, retry up to 3
    ///   - `-1021` → force time-sync, retry once
    ///   - everything else → surface
    fn run_with_retry<F>(&self, weight: u32, is_order_endpoint: bool, mut build: F) -> Result<HttpResponse>
    where
        F: FnMut() -> HttpRequest,
    {
        let mut throttle_delay = THROTTLE_BACKOFF_INITIAL_S;
        let mut server_error_attempts = 0u32;
        let mut network_attempts = 0u32;
        let mut timestamp_retries = 0u32;
        loop {
            self.rate_limiter.acquire(weight);
            // Re-build the request on every iteration. For signed paths the
            // closure calls `build_signed_query` again, producing a fresh
            // timestamp + HMAC signature; for unsigned paths the closure
            // captures a pre-built URL.
            let req = build();
            let outcome = self.transport.execute(req);
            match outcome {
                Ok(resp) => {
                    self.update_rate_limiter(&resp);
                    if is_order_endpoint {
                        self.order_count.record_response(&resp.headers);
                    }
                    if resp.status >= 200 && resp.status < 300 {
                        return Ok(resp);
                    }
                    if RETRYABLE_THROTTLE_STATUSES.contains(&resp.status) {
                        let delay = parse_retry_after(resp.header(HEADER_RETRY_AFTER))
                            .unwrap_or(throttle_delay);
                        eprintln!(
                            "Binance throttled ({}); waiting {:.1}s.",
                            resp.status, delay
                        );
                        self.sleeper.sleep(Duration::from_secs_f64(delay));
                        throttle_delay = (throttle_delay * 2.0).min(THROTTLE_BACKOFF_MAX_S);
                        continue;
                    }
                    if (500..600).contains(&resp.status) {
                        server_error_attempts += 1;
                        if server_error_attempts <= SERVER_ERROR_RETRY_MAX {
                            let delay = SERVER_ERROR_BACKOFF_INITIAL_S
                                * (1u32 << (server_error_attempts - 1)) as f64;
                            eprintln!(
                                "Binance {} on idempotent request; retry {}/{} in {:.1}s",
                                resp.status,
                                server_error_attempts,
                                SERVER_ERROR_RETRY_MAX,
                                delay,
                            );
                            self.sleeper.sleep(Duration::from_secs_f64(delay));
                            continue;
                        }
                    }
                    let api_err = parse_api_error(resp.status, &resp.body);
                    if api_err.is_timestamp_skew()
                        && timestamp_retries < TIMESTAMP_REJECT_RETRY_MAX
                    {
                        timestamp_retries += 1;
                        eprintln!(
                            "Binance rejected timestamp; force-resyncing server time and retrying."
                        );
                        self.server_time.ensure_synced(true);
                        continue;
                    }
                    return Err(api_err);
                }
                Err(e) => {
                    network_attempts += 1;
                    if network_attempts <= NETWORK_RETRY_MAX {
                        let delay = NETWORK_BACKOFF_INITIAL_S
                            * (1u32 << (network_attempts - 1)) as f64;
                        eprintln!(
                            "Network error: {}; retry {}/{} in {:.1}s",
                            e, network_attempts, NETWORK_RETRY_MAX, delay,
                        );
                        self.sleeper.sleep(Duration::from_secs_f64(delay));
                        continue;
                    }
                    return Err(e);
                }
            }
        }
    }

    /// Family-2 retry: order placement with idempotent recovery via client ID.
    ///
    /// On 5xx / network: enter the recovery loop (sleep, query by client ID,
    /// only re-POST on -2013). On terminal-4xx allow-list: surface immediately
    /// so caller marks the position FAILED. On other 4xx: still error out but
    /// do not retry (caller's `check_fills` will reconcile next pass).
    fn execute_order_post(
        &self,
        path: &str,
        params: Vec<(String, String)>,
        weight: u32,
        client_id_field: &str,
        client_id_value: &str,
        symbol_for_query: Option<&str>,
        path_kind: OrderPathKind,
    ) -> Result<HttpResponse> {
        // Pre-flight: order-count throttle.
        if let Some(sleep) = self.order_count.pre_check() {
            eprintln!(
                "Pre-emptive throttle: order-count near limit, sleeping {:.1}s",
                sleep.as_secs_f64()
            );
            self.sleeper.sleep(sleep);
        }

        let mut attempts = 0u32;
        let mut backoff = ORDER_RECOVERY_BACKOFF_INITIAL_S;
        // Dedicated counter — NOT folded into `attempts`. Without this a
        // persistent -1021 response would loop forever because the previous
        // implementation decremented `attempts` on each timestamp retry.
        let mut timestamp_retries = 0u32;

        loop {
            attempts += 1;
            let mut params_for_attempt = params.clone();
            // The client ID is already in `params`; we leave it as-is on
            // every attempt so retries are signature-stable. Build a new
            // signed query each time because timestamp must move forward.
            let query = self.build_signed_query(&mut params_for_attempt);
            let url = format!("{}{}", self.base_url, path);

            let outcome = (|| -> Result<HttpResponse> {
                self.rate_limiter.acquire(weight);
                let resp = self.transport.execute(HttpRequest {
                    method: HttpMethod::Post,
                    url: url.clone(),
                    headers: vec![("X-MBX-APIKEY".into(), self.api_key.clone())],
                    body: Some(query),
                })?;
                self.update_rate_limiter(&resp);
                self.order_count.record_response(&resp.headers);
                Ok(resp)
            })();

            match outcome {
                Ok(resp) if resp.status >= 200 && resp.status < 300 => return Ok(resp),
                Ok(resp) if RETRYABLE_THROTTLE_STATUSES.contains(&resp.status) => {
                    let delay = parse_retry_after(resp.header(HEADER_RETRY_AFTER))
                        .unwrap_or(THROTTLE_BACKOFF_INITIAL_S);
                    eprintln!(
                        "Binance throttled ({}); waiting {:.1}s before retry.",
                        resp.status, delay,
                    );
                    self.sleeper.sleep(Duration::from_secs_f64(delay));
                    // Throttle is an "engine-rejected" outcome — the order
                    // never got there. Same client_order_id, same retry
                    // sequence, no recovery loop needed.
                    continue;
                }
                Ok(resp) if (500..600).contains(&resp.status) => {
                    eprintln!(
                        "Binance {} on order POST; entering recovery (attempt {}).",
                        resp.status, attempts
                    );
                    if let Some(found) = self.recover_order_by_client_id(
                        path_kind,
                        symbol_for_query,
                        client_id_value,
                    )? {
                        return Ok(found);
                    }
                    // -2013 path: order does not exist; sleep and re-POST.
                    if attempts >= ORDER_RECOVERY_MAX_ATTEMPTS {
                        return Err(LiveError::Http(format!(
                            "order POST failed after {} attempts (last {} {})",
                            attempts, resp.status, resp.body,
                        )));
                    }
                    self.sleeper.sleep(Duration::from_secs_f64(backoff));
                    backoff = (backoff * 2.0).min(ORDER_RECOVERY_BACKOFF_MAX_S);
                    let _ = client_id_field; // already in params
                    continue;
                }
                Ok(resp) => {
                    let api_err = parse_api_error(resp.status, &resp.body);
                    if api_err.is_timestamp_skew()
                        && timestamp_retries < TIMESTAMP_REJECT_RETRY_MAX
                    {
                        timestamp_retries += 1;
                        eprintln!(
                            "Binance rejected timestamp on order POST; force-resync + retry ({}/{}).",
                            timestamp_retries, TIMESTAMP_REJECT_RETRY_MAX,
                        );
                        self.server_time.ensure_synced(true);
                        // Don't increment `attempts` for the recovery-loop
                        // budget — the request never reached the matching
                        // engine. The dedicated `timestamp_retries` counter
                        // bounds this branch so a persistent skew can't loop
                        // forever.
                        attempts = attempts.saturating_sub(1);
                        continue;
                    }
                    return Err(api_err);
                }
                Err(LiveError::Http(msg)) => {
                    eprintln!(
                        "Network error on order POST: {}; entering recovery (attempt {}).",
                        msg, attempts
                    );
                    if let Some(found) = self.recover_order_by_client_id(
                        path_kind,
                        symbol_for_query,
                        client_id_value,
                    )? {
                        return Ok(found);
                    }
                    if attempts >= ORDER_RECOVERY_MAX_ATTEMPTS {
                        return Err(LiveError::Http(format!(
                            "order POST network failure after {} attempts: {}",
                            attempts, msg,
                        )));
                    }
                    self.sleeper.sleep(Duration::from_secs_f64(backoff));
                    backoff = (backoff * 2.0).min(ORDER_RECOVERY_BACKOFF_MAX_S);
                    continue;
                }
                Err(e) => return Err(e),
            }
        }
    }

    /// Query Binance for an order we may have placed. Returns `Ok(Some(_))`
    /// when found, `Ok(None)` on -2013 (definitively absent — caller may
    /// re-POST), `Err` on every other failure (caller defers).
    fn recover_order_by_client_id(
        &self,
        kind: OrderPathKind,
        symbol: Option<&str>,
        client_id: &str,
    ) -> Result<Option<HttpResponse>> {
        // Build the unsigned param list once. The closure inside
        // `run_with_retry` re-signs from this list every iteration so a
        // -1021 retry uses a fresh timestamp + signature.
        let (path, base_params) = match kind {
            OrderPathKind::Normal => {
                let symbol = symbol.expect("Normal-order recovery requires symbol");
                (
                    PATH_ORDER,
                    vec![
                        ("symbol".to_string(), symbol_for_api(symbol)),
                        ("origClientOrderId".to_string(), client_id.to_string()),
                    ],
                )
            }
            OrderPathKind::Algo => (
                PATH_ALGO_ORDER,
                vec![("clientAlgoId".to_string(), client_id.to_string())],
            ),
        };
        let resp = self.run_with_retry(1, false, || {
            let mut p = base_params.clone();
            let query = self.build_signed_query(&mut p);
            HttpRequest {
                method: HttpMethod::Get,
                url: format!("{}{}?{}", self.base_url, path, query),
                headers: vec![("X-MBX-APIKEY".into(), self.api_key.clone())],
                body: None,
            }
        });
        match resp {
            Ok(r) => Ok(Some(r)),
            Err(e) if e.is_unknown_order() => Ok(None),
            Err(e) => Err(e),
        }
    }

    // ----- Helper: weight sync from response ------------------------------

    fn update_rate_limiter(&self, resp: &HttpResponse) {
        if let Some(used) = resp
            .header(HEADER_USED_WEIGHT_1M)
            .and_then(|v| v.parse::<u32>().ok())
        {
            self.rate_limiter.sync_from_server(used);
        }
    }

    // ----- High-level wrappers used by FuturesApi methods ----------------

    fn signed_get(
        &self,
        path: &str,
        params: &[(String, String)],
        weight: u32,
    ) -> Result<serde_json::Value> {
        let resp = self.run_with_retry(weight, false, || {
            let mut p = params.to_vec();
            let query = self.build_signed_query(&mut p);
            HttpRequest {
                method: HttpMethod::Get,
                url: format!("{}{}?{}", self.base_url, path, query),
                headers: vec![("X-MBX-APIKEY".into(), self.api_key.clone())],
                body: None,
            }
        })?;
        parse_json(&resp.body)
    }

    fn unsigned_get(
        &self,
        path: &str,
        params: &[(String, String)],
        weight: u32,
    ) -> Result<serde_json::Value> {
        let query = self.build_unsigned_query(params);
        let url = if query.is_empty() {
            format!("{}{}", self.base_url, path)
        } else {
            format!("{}{}?{}", self.base_url, path, query)
        };
        let resp = self.run_with_retry(weight, false, || HttpRequest {
            method: HttpMethod::Get,
            url: url.clone(),
            headers: vec![],
            body: None,
        })?;
        parse_json(&resp.body)
    }

    fn signed_post_idempotent(
        &self,
        path: &str,
        params: &[(String, String)],
        weight: u32,
    ) -> Result<serde_json::Value> {
        let resp = self.run_with_retry(weight, false, || {
            let mut p = params.to_vec();
            let body = self.build_signed_query(&mut p);
            HttpRequest {
                method: HttpMethod::Post,
                url: format!("{}{}", self.base_url, path),
                headers: vec![("X-MBX-APIKEY".into(), self.api_key.clone())],
                body: Some(body),
            }
        })?;
        parse_json(&resp.body)
    }

    fn signed_delete_idempotent(
        &self,
        path: &str,
        params: &[(String, String)],
        weight: u32,
    ) -> Result<serde_json::Value> {
        let resp = self.run_with_retry(weight, false, || {
            let mut p = params.to_vec();
            let body = self.build_signed_query(&mut p);
            HttpRequest {
                method: HttpMethod::Delete,
                url: format!("{}{}", self.base_url, path),
                headers: vec![("X-MBX-APIKEY".into(), self.api_key.clone())],
                body: Some(body),
            }
        })?;
        parse_json(&resp.body)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OrderPathKind {
    Normal,
    Algo,
}

// ---------------------------------------------------------------------------
// FuturesApi impl
// ---------------------------------------------------------------------------

impl FuturesApi for BinanceFuturesClient {
    fn server_now(&self) -> DateTime<Utc> {
        self.server_time.now_utc()
    }

    fn place_market_order(
        &self,
        symbol: &str,
        side: OrderSide,
        quantity: f64,
        position_side: &str,
        client_order_id: &str,
    ) -> Result<ExchangeOrder> {
        let params = vec![
            ("symbol".into(), symbol_for_api(symbol)),
            ("side".into(), side.as_str().into()),
            ("type".into(), "MARKET".into()),
            ("quantity".into(), format_amount(quantity)),
            ("positionSide".into(), position_side.into()),
            ("newClientOrderId".into(), client_order_id.into()),
        ];
        let resp = self.execute_order_post(
            PATH_ORDER,
            params,
            1,
            "newClientOrderId",
            client_order_id,
            Some(symbol),
            OrderPathKind::Normal,
        )?;
        parse_normal_order(&parse_json(&resp.body)?, client_order_id)
    }

    fn place_limit_order(
        &self,
        symbol: &str,
        side: OrderSide,
        quantity: f64,
        price: f64,
        position_side: &str,
        client_order_id: &str,
    ) -> Result<ExchangeOrder> {
        let params = vec![
            ("symbol".into(), symbol_for_api(symbol)),
            ("side".into(), side.as_str().into()),
            ("type".into(), "LIMIT".into()),
            ("timeInForce".into(), "GTC".into()),
            ("quantity".into(), format_amount(quantity)),
            ("price".into(), format_amount(price)),
            ("positionSide".into(), position_side.into()),
            ("newClientOrderId".into(), client_order_id.into()),
        ];
        let resp = self.execute_order_post(
            PATH_ORDER,
            params,
            1,
            "newClientOrderId",
            client_order_id,
            Some(symbol),
            OrderPathKind::Normal,
        )?;
        parse_normal_order(&parse_json(&resp.body)?, client_order_id)
    }

    fn place_stop_market(
        &self,
        symbol: &str,
        side: OrderSide,
        stop_price: f64,
        position_side: &str,
        quantity: Option<f64>,
        client_algo_id: &str,
    ) -> Result<ExchangeOrder> {
        let mut params = vec![
            ("algoType".into(), "CONDITIONAL".into()),
            ("symbol".into(), symbol_for_api(symbol)),
            ("side".into(), side.as_str().into()),
            ("type".into(), "STOP_MARKET".into()),
            ("triggerPrice".into(), format_amount(stop_price)),
            ("positionSide".into(), position_side.into()),
            ("clientAlgoId".into(), client_algo_id.into()),
        ];
        match quantity {
            Some(q) => params.push(("quantity".into(), format_amount(q))),
            None => params.push(("closePosition".into(), "true".into())),
        }
        let resp = self.execute_order_post(
            PATH_ALGO_ORDER,
            params,
            1,
            "clientAlgoId",
            client_algo_id,
            None,
            OrderPathKind::Algo,
        )?;
        parse_algo_order(&parse_json(&resp.body)?, client_algo_id)
    }

    fn place_take_profit_market(
        &self,
        symbol: &str,
        side: OrderSide,
        stop_price: f64,
        position_side: &str,
        quantity: Option<f64>,
        client_algo_id: &str,
    ) -> Result<ExchangeOrder> {
        let mut params = vec![
            ("algoType".into(), "CONDITIONAL".into()),
            ("symbol".into(), symbol_for_api(symbol)),
            ("side".into(), side.as_str().into()),
            ("type".into(), "TAKE_PROFIT_MARKET".into()),
            ("triggerPrice".into(), format_amount(stop_price)),
            ("positionSide".into(), position_side.into()),
            ("clientAlgoId".into(), client_algo_id.into()),
        ];
        match quantity {
            Some(q) => params.push(("quantity".into(), format_amount(q))),
            None => params.push(("closePosition".into(), "true".into())),
        }
        let resp = self.execute_order_post(
            PATH_ALGO_ORDER,
            params,
            1,
            "clientAlgoId",
            client_algo_id,
            None,
            OrderPathKind::Algo,
        )?;
        parse_algo_order(&parse_json(&resp.body)?, client_algo_id)
    }

    fn cancel_order(&self, symbol: &str, order_id: i64) -> Result<ExchangeOrder> {
        let params = vec![
            ("symbol".into(), symbol_for_api(symbol)),
            ("orderId".into(), order_id.to_string()),
        ];
        let v = self.signed_delete_idempotent(PATH_ORDER, &params, 1)?;
        parse_normal_order(&v, "")
    }

    fn cancel_algo_order(&self, algo_id: i64) -> Result<()> {
        let params = vec![("algoId".into(), algo_id.to_string())];
        self.signed_delete_idempotent(PATH_ALGO_ORDER, &params, 1)?;
        Ok(())
    }

    fn get_order(&self, symbol: &str, order_id: i64) -> Result<ExchangeOrder> {
        let params = vec![
            ("symbol".into(), symbol_for_api(symbol)),
            ("orderId".into(), order_id.to_string()),
        ];
        let v = self.signed_get(PATH_ORDER, &params, 1)?;
        parse_normal_order(&v, "")
    }

    fn get_order_by_client_id(
        &self,
        symbol: &str,
        client_id: &str,
    ) -> Result<Option<ExchangeOrder>> {
        let params = vec![
            ("symbol".into(), symbol_for_api(symbol)),
            ("origClientOrderId".into(), client_id.into()),
        ];
        match self.signed_get(PATH_ORDER, &params, 1) {
            Ok(v) => Ok(Some(parse_normal_order(&v, client_id)?)),
            Err(e) if e.is_unknown_order() => Ok(None),
            Err(e) => Err(e),
        }
    }

    fn get_algo_order(&self, algo_id: i64) -> Result<ExchangeOrder> {
        let params = vec![("algoId".into(), algo_id.to_string())];
        let v = self.signed_get(PATH_ALGO_ORDER, &params, 1)?;
        parse_algo_order(&v, "")
    }

    fn get_algo_order_by_client_id(
        &self,
        client_id: &str,
    ) -> Result<Option<ExchangeOrder>> {
        let params = vec![("clientAlgoId".into(), client_id.into())];
        match self.signed_get(PATH_ALGO_ORDER, &params, 1) {
            Ok(v) => Ok(Some(parse_algo_order(&v, client_id)?)),
            Err(e) if e.is_unknown_order() => Ok(None),
            Err(e) => Err(e),
        }
    }

    fn get_open_orders(&self, symbol: Option<&str>) -> Result<Vec<ExchangeOrder>> {
        let mut params = Vec::new();
        if let Some(s) = symbol {
            params.push(("symbol".into(), symbol_for_api(s)));
        }
        let v = self.signed_get(PATH_OPEN_ORDERS, &params, 1)?;
        let arr = v
            .as_array()
            .ok_or_else(|| LiveError::Parse("openOrders not an array".into()))?;
        arr.iter().map(|x| parse_normal_order(x, "")).collect()
    }

    fn get_position_info(&self, symbol: Option<&str>) -> Result<Vec<serde_json::Value>> {
        let mut params = Vec::new();
        if let Some(s) = symbol {
            params.push(("symbol".into(), symbol_for_api(s)));
        }
        let v = self.signed_get(PATH_POSITION_RISK, &params, 5)?;
        let arr = v
            .as_array()
            .ok_or_else(|| LiveError::Parse("positionRisk not an array".into()))?;
        Ok(arr.clone())
    }

    fn get_account_trades(
        &self,
        symbol: &str,
        start: Option<DateTime<Utc>>,
        end: Option<DateTime<Utc>>,
        order_id: Option<i64>,
        limit: usize,
    ) -> Result<Vec<AccountTrade>> {
        let limit = limit.clamp(1, 100);
        let mut params = vec![
            ("symbol".into(), symbol_for_api(symbol)),
            ("limit".into(), limit.to_string()),
        ];
        if let Some(s) = start {
            params.push(("startTime".into(), dt_to_ms(s).to_string()));
        }
        if let Some(e) = end {
            params.push(("endTime".into(), dt_to_ms(e).to_string()));
        }
        if let Some(o) = order_id {
            params.push(("orderId".into(), o.to_string()));
        }
        let v = self.signed_get(PATH_USER_TRADES, &params, 5)?;
        let arr = v
            .as_array()
            .ok_or_else(|| LiveError::Parse("userTrades not an array".into()))?;
        arr.iter().map(parse_account_trade).collect()
    }

    fn get_account_info(&self) -> Result<serde_json::Value> {
        self.signed_get(PATH_ACCOUNT, &[], 5)
    }

    fn get_available_balance(&self) -> Result<f64> {
        let v = self.get_account_info()?;
        v["availableBalance"]
            .as_str()
            .and_then(|s| s.parse().ok())
            .ok_or_else(|| LiveError::Parse("availableBalance missing or unparseable".into()))
    }

    fn set_leverage(&self, symbol: &str, leverage: u32) -> Result<()> {
        let params = vec![
            ("symbol".into(), symbol_for_api(symbol)),
            ("leverage".into(), leverage.to_string()),
        ];
        self.signed_post_idempotent(PATH_LEVERAGE, &params, 1)?;
        Ok(())
    }

    fn get_exchange_info(&self) -> Result<serde_json::Value> {
        self.unsigned_get(PATH_EXCHANGE_INFO, &[], 1)
    }

    fn get_mark_price(&self, symbol: &str) -> Result<f64> {
        let params = vec![("symbol".into(), symbol_for_api(symbol))];
        let v = self.unsigned_get(PATH_PREMIUM_INDEX, &params, 1)?;
        v["markPrice"]
            .as_str()
            .and_then(|s| s.parse().ok())
            .ok_or_else(|| LiveError::Parse("markPrice missing or unparseable".into()))
    }
}

// ---------------------------------------------------------------------------
// Free functions: query building, parsing, encoding
// ---------------------------------------------------------------------------

pub fn symbol_for_api(ticker: &str) -> String {
    ticker.replace('/', "")
}

/// Extract the `ORDERS` rate limits from an `exchangeInfo` payload.
/// Returns `(limit_10s, limit_1m)` if both entries are present, else `None`.
pub fn parse_order_rate_limits(info: &serde_json::Value) -> Option<(u32, u32)> {
    let arr = info.get("rateLimits")?.as_array()?;
    let mut limit_10s: Option<u32> = None;
    let mut limit_1m: Option<u32> = None;
    for row in arr {
        let kind = row.get("rateLimitType").and_then(|v| v.as_str()).unwrap_or("");
        if kind != "ORDERS" {
            continue;
        }
        let interval = row.get("interval").and_then(|v| v.as_str()).unwrap_or("");
        let interval_num = row.get("intervalNum").and_then(|v| v.as_u64()).unwrap_or(0);
        let limit = row.get("limit").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
        if interval == "SECOND" && interval_num == 10 {
            limit_10s = Some(limit);
        } else if interval == "MINUTE" && interval_num == 1 {
            limit_1m = Some(limit);
        }
    }
    Some((limit_10s?, limit_1m?))
}

pub fn build_query(params: &[(String, String)]) -> String {
    params
        .iter()
        .map(|(k, v)| {
            format!(
                "{}={}",
                urlencoding::encode(k),
                urlencoding::encode(v),
            )
        })
        .collect::<Vec<_>>()
        .join("&")
}

fn format_amount(v: f64) -> String {
    // Match Python's `f"{v}"` formatting. Trim trailing zeros so we don't
    // emit `1.000000` (Binance's filters check the textual precision).
    let s = format!("{v}");
    s
}

fn parse_retry_after(raw: Option<&str>) -> Option<f64> {
    raw?.trim().parse::<f64>().ok().map(|v| v.max(0.0))
}

fn parse_json(body: &str) -> Result<serde_json::Value> {
    serde_json::from_str(body).map_err(|e| LiveError::Parse(e.to_string()))
}

fn parse_api_error(status: u16, body: &str) -> LiveError {
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(body) {
        if let Some(code) = v.get("code").and_then(|c| c.as_i64()) {
            let msg = v
                .get("msg")
                .and_then(|m| m.as_str())
                .unwrap_or("")
                .to_string();
            return LiveError::Api { code, msg };
        }
    }
    LiveError::Http(format!("HTTP {status}: {body}"))
}

const ALGO_STATUS_NEW: &str = "NEW";
const ALGO_STATUS_TRIGGERED: &str = "TRIGGERED";
const ALGO_STATUS_CANCELED: &[&str] = &["CANCELED", "CANCELLED"];
const ALGO_STATUS_EXPIRED: &str = "EXPIRED";
const ALGO_STATUS_REJECTED: &str = "REJECTED";

pub fn parse_normal_order(v: &serde_json::Value, client_id: &str) -> Result<ExchangeOrder> {
    let order_id = v["orderId"]
        .as_i64()
        .ok_or_else(|| LiveError::Parse("orderId missing".into()))?;
    let symbol = v["symbol"].as_str().unwrap_or("").to_string();
    let side = match v["side"].as_str().unwrap_or("BUY") {
        "SELL" => OrderSide::Sell,
        _ => OrderSide::Buy,
    };
    // Python prefers origType (the order's logical type, e.g. STOP_MARKET)
    // over type (which can show up as MARKET after a stop fires).
    let type_str = v
        .get("origType")
        .and_then(|s| s.as_str())
        .or_else(|| v.get("type").and_then(|s| s.as_str()))
        .unwrap_or("MARKET");
    let order_type = parse_order_type(type_str);
    let quantity = parse_optional_f64(v.get("origQty")).unwrap_or(0.0);
    let price = parse_optional_f64(v.get("price")).unwrap_or(0.0);
    let stop_price = parse_optional_f64(v.get("stopPrice")).unwrap_or(0.0);
    let status = parse_normal_status(v["status"].as_str().unwrap_or("NEW"))?;
    let filled_qty = parse_optional_f64(v.get("executedQty")).unwrap_or(0.0);
    let avg_fill_price = parse_optional_f64(v.get("avgPrice")).unwrap_or(0.0);
    let created_at = v.get("time").and_then(|t| t.as_i64()).and_then(ms_to_dt_opt);
    let updated_at = v
        .get("updateTime")
        .and_then(|t| t.as_i64())
        .and_then(ms_to_dt_opt);
    // Server-supplied client ID takes priority (matches what we sent), else
    // fall back to the caller-provided value (used by query-by-client-id).
    let server_cid = v
        .get("clientOrderId")
        .and_then(|s| s.as_str())
        .map(|s| s.to_string())
        .unwrap_or_default();
    let client_order_id = if !server_cid.is_empty() {
        server_cid
    } else {
        client_id.to_string()
    };
    Ok(ExchangeOrder {
        order_id,
        symbol,
        side,
        order_type,
        quantity,
        price,
        stop_price,
        status,
        filled_qty,
        avg_fill_price,
        created_at,
        updated_at,
        algo_id: 0,
        client_order_id,
    })
}

pub fn parse_algo_order(v: &serde_json::Value, client_id: &str) -> Result<ExchangeOrder> {
    let algo_status = v
        .get("algoStatus")
        .and_then(|s| s.as_str())
        .unwrap_or(ALGO_STATUS_NEW);
    let status = if algo_status == ALGO_STATUS_NEW {
        OrderStatus::New
    } else if algo_status == ALGO_STATUS_TRIGGERED {
        OrderStatus::Filled
    } else if ALGO_STATUS_CANCELED.contains(&algo_status) {
        OrderStatus::Canceled
    } else if algo_status == ALGO_STATUS_EXPIRED {
        OrderStatus::Expired
    } else if algo_status == ALGO_STATUS_REJECTED {
        OrderStatus::Rejected
    } else {
        OrderStatus::New
    };
    let type_str = v
        .get("orderType")
        .and_then(|s| s.as_str())
        .or_else(|| v.get("type").and_then(|s| s.as_str()))
        .unwrap_or("STOP_MARKET");
    let order_type = parse_order_type(type_str);
    let quantity = parse_optional_f64(v.get("quantity")).unwrap_or(0.0);
    let symbol = v["symbol"].as_str().unwrap_or("").to_string();
    let side = match v["side"].as_str().unwrap_or("BUY") {
        "SELL" => OrderSide::Sell,
        _ => OrderSide::Buy,
    };
    let price = parse_optional_f64(v.get("price")).unwrap_or(0.0);
    let stop_price = parse_optional_f64(v.get("triggerPrice")).unwrap_or(0.0);
    let avg_fill_price = parse_optional_f64(v.get("actualPrice")).unwrap_or(0.0);
    let filled_qty = if matches!(status, OrderStatus::Filled) {
        quantity
    } else {
        0.0
    };
    let created_at = v
        .get("createTime")
        .and_then(|t| t.as_i64())
        .and_then(ms_to_dt_opt);
    let updated_at = v
        .get("updateTime")
        .and_then(|t| t.as_i64())
        .and_then(ms_to_dt_opt);
    let order_id = v
        .get("actualOrderId")
        .and_then(|n| n.as_i64())
        .unwrap_or(0);
    let algo_id = v
        .get("algoId")
        .and_then(|n| n.as_i64())
        .ok_or_else(|| LiveError::Parse("algoId missing".into()))?;
    let server_cid = v
        .get("clientAlgoId")
        .and_then(|s| s.as_str())
        .map(|s| s.to_string())
        .unwrap_or_default();
    let client_order_id = if !server_cid.is_empty() {
        server_cid
    } else {
        client_id.to_string()
    };
    Ok(ExchangeOrder {
        order_id,
        symbol,
        side,
        order_type,
        quantity,
        price,
        stop_price,
        status,
        filled_qty,
        avg_fill_price,
        created_at,
        updated_at,
        algo_id,
        client_order_id,
    })
}

pub fn parse_account_trade(v: &serde_json::Value) -> Result<AccountTrade> {
    let trade_id = v
        .get("id")
        .and_then(|n| n.as_i64())
        .unwrap_or(0);
    let order_id = v
        .get("orderId")
        .and_then(|n| n.as_i64())
        .unwrap_or(0);
    let symbol = v["symbol"].as_str().unwrap_or("").to_string();
    let side = match v["side"].as_str().unwrap_or("BUY") {
        "SELL" => OrderSide::Sell,
        _ => OrderSide::Buy,
    };
    let price = parse_optional_f64(v.get("price")).unwrap_or(0.0);
    let quantity = parse_optional_f64(v.get("qty"))
        .or_else(|| parse_optional_f64(v.get("quantity")))
        .unwrap_or(0.0);
    let time_ms = v
        .get("time")
        .and_then(|t| t.as_i64())
        .ok_or_else(|| LiveError::Parse("trade time missing".into()))?;
    let time = ms_to_dt_opt(time_ms)
        .ok_or_else(|| LiveError::Parse(format!("trade time ms={time_ms} out of range")))?;
    let realized_pnl = parse_optional_f64(v.get("realizedPnl")).unwrap_or(0.0);
    let commission = parse_optional_f64(v.get("commission")).unwrap_or(0.0);
    let commission_asset = v
        .get("commissionAsset")
        .and_then(|s| s.as_str())
        .unwrap_or("")
        .to_string();
    let position_side = v
        .get("positionSide")
        .and_then(|s| s.as_str())
        .unwrap_or("BOTH")
        .to_string();
    Ok(AccountTrade {
        trade_id,
        order_id,
        symbol,
        side,
        price,
        quantity,
        time,
        realized_pnl,
        commission,
        commission_asset,
        position_side,
    })
}

fn parse_optional_f64(v: Option<&serde_json::Value>) -> Option<f64> {
    let v = v?;
    if let Some(s) = v.as_str() {
        return s.parse().ok();
    }
    v.as_f64()
}

fn parse_order_type(s: &str) -> OrderType {
    match s {
        "LIMIT" => OrderType::Limit,
        "STOP_MARKET" => OrderType::StopMarket,
        "TAKE_PROFIT_MARKET" => OrderType::TakeProfitMarket,
        _ => OrderType::Market,
    }
}

fn parse_normal_status(s: &str) -> Result<OrderStatus> {
    Ok(match s {
        "NEW" | "PARTIALLY_FILLED" => OrderStatus::New,
        "FILLED" => OrderStatus::Filled,
        "CANCELED" | "CANCELLED" => OrderStatus::Canceled,
        "EXPIRED" => OrderStatus::Expired,
        "REJECTED" => OrderStatus::Rejected,
        other => {
            return Err(LiveError::Parse(format!("unknown order status: {other}")))
        }
    })
}

// ---------------------------------------------------------------------------
// Tests — signing primitives, parsers, and helper logic.
// HTTP-integration tests live in `tests/auth_client_http.rs`.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Golden case from Binance USD-M signing docs.
    /// Verifying our HMAC and query encoding produce the documented signature
    /// guards against regressions in canonical-encoding order or HMAC wiring.
    #[test]
    fn hmac_signature_matches_binance_docs() {
        let secret = b"NhqPtmdSJYdKjVHjA7PZj4Mge3R5YNiP1e3UZjInClVN65XAbvqqM6A7H5fATj0j";
        let query = "symbol=LTCBTC&side=BUY&type=LIMIT&timeInForce=GTC&quantity=1&price=0.1&recvWindow=5000&timestamp=1499827319559";
        let mut mac = HmacSha256::new_from_slice(secret).unwrap();
        mac.update(query.as_bytes());
        let sig = hex::encode(mac.finalize().into_bytes());
        assert_eq!(
            sig,
            "c8db56825ae71d6d79447849e617115f4a920fa2acdcab2b053c4b2838bd6b71"
        );
    }

    #[test]
    fn build_query_preserves_order_and_encodes() {
        let params = vec![
            ("symbol".to_string(), "BTC/USDT".to_string()),
            ("side".to_string(), "BUY".to_string()),
        ];
        // We don't strip the `/` ourselves at this layer — `symbol_for_api`
        // already converted it. Demonstrate the encoder will percent-encode
        // it if a value sneaks through.
        let q = build_query(&params);
        assert_eq!(q, "symbol=BTC%2FUSDT&side=BUY");
    }

    #[test]
    fn symbol_for_api_strips_slash() {
        assert_eq!(symbol_for_api("BTC/USDT"), "BTCUSDT");
        assert_eq!(symbol_for_api("BTCUSDT"), "BTCUSDT");
    }

    #[test]
    fn parse_normal_order_handles_filled_market() {
        let raw = serde_json::json!({
            "orderId": 12345,
            "symbol": "BTCUSDT",
            "side": "BUY",
            "type": "MARKET",
            "origType": "MARKET",
            "origQty": "0.038",
            "executedQty": "0.038",
            "avgPrice": "43000.50",
            "price": "0",
            "stopPrice": "0",
            "status": "FILLED",
            "time": 1_700_000_000_000_i64,
            "updateTime": 1_700_000_000_500_i64,
            "clientOrderId": "abc123",
        });
        let parsed = parse_normal_order(&raw, "fallback").unwrap();
        assert_eq!(parsed.order_id, 12345);
        assert_eq!(parsed.symbol, "BTCUSDT");
        assert_eq!(parsed.side, OrderSide::Buy);
        assert_eq!(parsed.order_type, OrderType::Market);
        assert_eq!(parsed.quantity, 0.038);
        assert_eq!(parsed.filled_qty, 0.038);
        assert_eq!(parsed.avg_fill_price, 43000.5);
        assert_eq!(parsed.status, OrderStatus::Filled);
        assert_eq!(parsed.client_order_id, "abc123");
        assert_eq!(parsed.algo_id, 0);
    }

    #[test]
    fn parse_normal_order_partial_fill_treated_as_new() {
        let raw = serde_json::json!({
            "orderId": 1,
            "symbol": "BTCUSDT",
            "side": "BUY",
            "type": "LIMIT",
            "origType": "LIMIT",
            "origQty": "0.1",
            "executedQty": "0.05",
            "avgPrice": "43000",
            "price": "43000",
            "stopPrice": "0",
            "status": "PARTIALLY_FILLED",
        });
        let parsed = parse_normal_order(&raw, "").unwrap();
        // Live tracker treats PARTIALLY_FILLED as still working (matches Python).
        assert_eq!(parsed.status, OrderStatus::New);
    }

    #[test]
    fn parse_normal_order_falls_back_to_caller_client_id() {
        let raw = serde_json::json!({
            "orderId": 1,
            "symbol": "BTCUSDT",
            "side": "BUY",
            "type": "MARKET",
            "origQty": "0",
            "status": "NEW",
        });
        let parsed = parse_normal_order(&raw, "caller-supplied").unwrap();
        assert_eq!(parsed.client_order_id, "caller-supplied");
    }

    #[test]
    fn parse_algo_order_maps_triggered_to_filled() {
        let raw = serde_json::json!({
            "algoId": 999,
            "actualOrderId": 12345,
            "symbol": "BTCUSDT",
            "side": "SELL",
            "orderType": "TAKE_PROFIT_MARKET",
            "type": "TAKE_PROFIT_MARKET",
            "triggerPrice": "45000",
            "actualPrice": "45000",
            "quantity": "0.038",
            "algoStatus": "TRIGGERED",
            "createTime": 1_700_000_000_000_i64,
            "updateTime": 1_700_000_001_000_i64,
            "clientAlgoId": "tp-uuid",
        });
        let parsed = parse_algo_order(&raw, "").unwrap();
        assert_eq!(parsed.algo_id, 999);
        assert_eq!(parsed.order_id, 12345);
        assert_eq!(parsed.status, OrderStatus::Filled);
        assert_eq!(parsed.order_type, OrderType::TakeProfitMarket);
        assert_eq!(parsed.stop_price, 45000.0);
        assert_eq!(parsed.avg_fill_price, 45000.0);
        assert_eq!(parsed.filled_qty, 0.038);
        assert_eq!(parsed.client_order_id, "tp-uuid");
    }

    #[test]
    fn parse_algo_order_canceled_variants() {
        for status in &["CANCELED", "CANCELLED"] {
            let raw = serde_json::json!({
                "algoId": 1,
                "symbol": "BTCUSDT",
                "side": "SELL",
                "orderType": "STOP_MARKET",
                "triggerPrice": "0",
                "quantity": "0.1",
                "algoStatus": status,
            });
            let parsed = parse_algo_order(&raw, "").unwrap();
            assert_eq!(parsed.status, OrderStatus::Canceled, "for {status}");
        }
    }

    #[test]
    fn parse_api_error_extracts_code_and_msg() {
        let body = r#"{"code":-1021,"msg":"Timestamp for this request is outside of the recvWindow."}"#;
        let err = parse_api_error(400, body);
        assert!(matches!(
            err,
            LiveError::Api { code: -1021, .. }
        ));
        assert!(err.is_timestamp_skew());
    }

    #[test]
    fn parse_api_error_falls_back_to_http_when_no_code() {
        let body = "<html>not json</html>";
        let err = parse_api_error(503, body);
        assert!(matches!(err, LiveError::Http(_)));
    }

    #[test]
    fn parse_retry_after_handles_seconds() {
        assert_eq!(parse_retry_after(Some("12")), Some(12.0));
        assert_eq!(parse_retry_after(Some("0.5")), Some(0.5));
        assert_eq!(parse_retry_after(Some("not-a-number")), None);
        assert_eq!(parse_retry_after(None), None);
        assert_eq!(parse_retry_after(Some("-3")), Some(0.0)); // clamped
    }

    #[test]
    fn terminal_4xx_codes_table_is_sane() {
        // Every terminal code is negative (Binance convention) and the list
        // contains the codes we explicitly enumerated in plan v2.
        for &c in TERMINAL_4XX_CODES {
            assert!(c < 0, "code {c} should be negative");
        }
        assert!(TERMINAL_4XX_CODES.contains(&-1013));
        assert!(TERMINAL_4XX_CODES.contains(&-2010));
        assert!(TERMINAL_4XX_CODES.contains(&-4131));
    }

    #[test]
    fn parse_normal_status_rejects_unknown() {
        assert!(matches!(parse_normal_status("BOGUS"), Err(_)));
    }

    #[test]
    fn parse_order_rate_limits_extracts_orders_entries() {
        let info = serde_json::json!({
            "rateLimits": [
                {"rateLimitType": "REQUEST_WEIGHT", "interval": "MINUTE", "intervalNum": 1, "limit": 2400},
                {"rateLimitType": "ORDERS", "interval": "SECOND", "intervalNum": 10, "limit": 500},
                {"rateLimitType": "ORDERS", "interval": "MINUTE", "intervalNum": 1, "limit": 2000}
            ]
        });
        let (l10, l1m) = parse_order_rate_limits(&info).unwrap();
        assert_eq!(l10, 500);
        assert_eq!(l1m, 2000);
    }

    #[test]
    fn parse_order_rate_limits_returns_none_when_orders_missing() {
        let info = serde_json::json!({
            "rateLimits": [
                {"rateLimitType": "REQUEST_WEIGHT", "interval": "MINUTE", "intervalNum": 1, "limit": 2400}
            ]
        });
        assert!(parse_order_rate_limits(&info).is_none());
    }

    #[test]
    fn parse_account_trade_round_trip() {
        let raw = serde_json::json!({
            "id": 100,
            "orderId": 8732145901_i64,
            "symbol": "ETHUSDT",
            "side": "SELL",
            "price": "2745.36",
            "qty": "0.038",
            "time": 1_700_000_000_500_i64,
            "realizedPnl": "5.12",
            "commission": "0.05",
            "commissionAsset": "USDT",
            "positionSide": "LONG",
        });
        let t = parse_account_trade(&raw).unwrap();
        assert_eq!(t.trade_id, 100);
        assert_eq!(t.order_id, 8732145901);
        assert_eq!(t.symbol, "ETHUSDT");
        assert_eq!(t.side, OrderSide::Sell);
        assert_eq!(t.price, 2745.36);
        assert_eq!(t.quantity, 0.038);
        assert_eq!(t.realized_pnl, 5.12);
        assert_eq!(t.commission, 0.05);
        assert_eq!(t.commission_asset, "USDT");
        assert_eq!(t.position_side, "LONG");
    }
}
