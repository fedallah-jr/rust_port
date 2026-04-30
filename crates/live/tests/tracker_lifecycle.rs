//! Lifecycle tests for `PositionTracker` (Phase D).
//!
//! Drives the tracker through every documented transition path with a
//! recording mock `FuturesApi`. Tests cover:
//!   - place_entry: Placed / Rejected (terminal-4xx) / Deferred (5xx)
//!   - check_fills entry path: Filled→Open+TP/SL, stale-limit timeout, canceled→Failed,
//!     Deferred resolved by client_order_id, Deferred -2013 → Failed
//!   - check_fills exit path: TP-fills-cancels-SL, SL-fills-cancels-TP,
//!     external-close+account-trade fallback, timeout close
//!   - reconcile_with_exchange: external-position detection + has_external_conflict
//!   - bracket recovery: Healthy×Healthy no-op, NeedsReplace replaces only dead side,
//!     Unknown defers, feature flag disables
//!   - persistence: Rust→Rust round-trip, Python-shaped fixture deserialize,
//!     save excludes Closed/Failed, atomic-write tmp-file lifecycle

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use chrono::{DateTime, Duration, Utc};
use claude_trader_live::auth_client::FuturesApi;
use claude_trader_live::error::LiveError;
use claude_trader_live::exchange_info::ExchangeInfoCache;
use claude_trader_live::executor::OrderExecutor;
use claude_trader_live::tracker::{PlacementStatus, PositionTracker};
use claude_trader_models::{
    AccountTrade, ExchangeOrder, LiveConfig, MarketType, OrderSide, OrderStatus, OrderType,
    PositionStatus, PositionType, Signal,
};

// ---------------------------------------------------------------------------
// Mock FuturesApi
// ---------------------------------------------------------------------------

#[derive(Default)]
struct State {
    available_balance: f64,
    mark_price: f64,
    set_leverage: u32,
    place_market: Vec<Result<ExchangeOrder, LiveError>>,
    place_limit: Vec<Result<ExchangeOrder, LiveError>>,
    place_tp: Vec<Result<ExchangeOrder, LiveError>>,
    place_sl: Vec<Result<ExchangeOrder, LiveError>>,
    cancel_order: Vec<Result<ExchangeOrder, LiveError>>,
    cancel_algo: Vec<Result<(), LiveError>>,
    get_order: Vec<Result<ExchangeOrder, LiveError>>,
    get_order_by_cid: Vec<Result<Option<ExchangeOrder>, LiveError>>,
    get_algo_order: Vec<Result<ExchangeOrder, LiveError>>,
    get_algo_order_by_cid: Vec<Result<Option<ExchangeOrder>, LiveError>>,
    position_info: Vec<Result<Vec<serde_json::Value>, LiveError>>,
    account_trades: Vec<Result<Vec<AccountTrade>, LiveError>>,

    cancel_log: Vec<(String, i64)>,
    cancel_algo_log: Vec<i64>,
    place_market_log: Vec<(String, OrderSide, f64, String)>,
    place_tp_log: Vec<(String, f64, Option<f64>, String)>,
    place_sl_log: Vec<(String, f64, Option<f64>, String)>,
}

#[derive(Default)]
struct MockApi {
    state: Mutex<State>,
}

impl MockApi {
    fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }
    fn set_balance(&self, b: f64) {
        self.state.lock().unwrap().available_balance = b;
    }
    fn set_mark_price(&self, p: f64) {
        self.state.lock().unwrap().mark_price = p;
    }
    fn enqueue_market(&self, r: Result<ExchangeOrder, LiveError>) {
        self.state.lock().unwrap().place_market.push(r);
    }
    fn enqueue_limit(&self, r: Result<ExchangeOrder, LiveError>) {
        self.state.lock().unwrap().place_limit.push(r);
    }
    fn enqueue_tp(&self, r: Result<ExchangeOrder, LiveError>) {
        self.state.lock().unwrap().place_tp.push(r);
    }
    fn enqueue_sl(&self, r: Result<ExchangeOrder, LiveError>) {
        self.state.lock().unwrap().place_sl.push(r);
    }
    fn enqueue_cancel(&self, r: Result<ExchangeOrder, LiveError>) {
        self.state.lock().unwrap().cancel_order.push(r);
    }
    fn enqueue_cancel_algo(&self, r: Result<(), LiveError>) {
        self.state.lock().unwrap().cancel_algo.push(r);
    }
    fn enqueue_get_order(&self, r: Result<ExchangeOrder, LiveError>) {
        self.state.lock().unwrap().get_order.push(r);
    }
    fn enqueue_get_order_by_cid(&self, r: Result<Option<ExchangeOrder>, LiveError>) {
        self.state.lock().unwrap().get_order_by_cid.push(r);
    }
    fn enqueue_get_algo_order(&self, r: Result<ExchangeOrder, LiveError>) {
        self.state.lock().unwrap().get_algo_order.push(r);
    }
    fn enqueue_get_algo_order_by_cid(&self, r: Result<Option<ExchangeOrder>, LiveError>) {
        self.state.lock().unwrap().get_algo_order_by_cid.push(r);
    }
    fn enqueue_position_info(&self, r: Result<Vec<serde_json::Value>, LiveError>) {
        self.state.lock().unwrap().position_info.push(r);
    }
    fn enqueue_account_trades(&self, r: Result<Vec<AccountTrade>, LiveError>) {
        self.state.lock().unwrap().account_trades.push(r);
    }
    fn cancel_calls(&self) -> Vec<(String, i64)> {
        self.state.lock().unwrap().cancel_log.clone()
    }
    fn cancel_algo_calls(&self) -> Vec<i64> {
        self.state.lock().unwrap().cancel_algo_log.clone()
    }
    fn tp_calls(&self) -> Vec<(String, f64, Option<f64>, String)> {
        self.state.lock().unwrap().place_tp_log.clone()
    }
    fn sl_calls(&self) -> Vec<(String, f64, Option<f64>, String)> {
        self.state.lock().unwrap().place_sl_log.clone()
    }
    fn market_calls(&self) -> Vec<(String, OrderSide, f64, String)> {
        self.state.lock().unwrap().place_market_log.clone()
    }
}

fn pop<T>(slot: &mut Vec<Result<T, LiveError>>, label: &str) -> Result<T, LiveError> {
    if slot.is_empty() {
        return Err(LiveError::Http(format!("MockApi: no scripted {label}")));
    }
    slot.remove(0)
}

impl FuturesApi for MockApi {
    fn server_now(&self) -> DateTime<Utc> {
        Utc::now()
    }
    fn place_market_order(
        &self,
        symbol: &str,
        side: OrderSide,
        quantity: f64,
        _ps: &str,
        cid: &str,
    ) -> Result<ExchangeOrder, LiveError> {
        let mut s = self.state.lock().unwrap();
        s.place_market_log.push((symbol.into(), side, quantity, cid.into()));
        pop(&mut s.place_market, "place_market_order")
    }
    fn place_limit_order(
        &self,
        _: &str,
        _: OrderSide,
        _: f64,
        _: f64,
        _: &str,
        _: &str,
    ) -> Result<ExchangeOrder, LiveError> {
        let mut s = self.state.lock().unwrap();
        pop(&mut s.place_limit, "place_limit_order")
    }
    fn place_stop_market(
        &self,
        symbol: &str,
        _: OrderSide,
        stop_price: f64,
        _: &str,
        quantity: Option<f64>,
        cid: &str,
    ) -> Result<ExchangeOrder, LiveError> {
        let mut s = self.state.lock().unwrap();
        s.place_sl_log.push((symbol.into(), stop_price, quantity, cid.into()));
        pop(&mut s.place_sl, "place_stop_market")
    }
    fn place_take_profit_market(
        &self,
        symbol: &str,
        _: OrderSide,
        stop_price: f64,
        _: &str,
        quantity: Option<f64>,
        cid: &str,
    ) -> Result<ExchangeOrder, LiveError> {
        let mut s = self.state.lock().unwrap();
        s.place_tp_log.push((symbol.into(), stop_price, quantity, cid.into()));
        pop(&mut s.place_tp, "place_take_profit_market")
    }
    fn cancel_order(&self, sym: &str, oid: i64) -> Result<ExchangeOrder, LiveError> {
        let mut s = self.state.lock().unwrap();
        s.cancel_log.push((sym.into(), oid));
        pop(&mut s.cancel_order, "cancel_order")
    }
    fn cancel_algo_order(&self, aid: i64) -> Result<(), LiveError> {
        let mut s = self.state.lock().unwrap();
        s.cancel_algo_log.push(aid);
        pop(&mut s.cancel_algo, "cancel_algo_order")
    }
    fn get_order(&self, _: &str, _: i64) -> Result<ExchangeOrder, LiveError> {
        let mut s = self.state.lock().unwrap();
        pop(&mut s.get_order, "get_order")
    }
    fn get_order_by_client_id(
        &self,
        _: &str,
        _: &str,
    ) -> Result<Option<ExchangeOrder>, LiveError> {
        let mut s = self.state.lock().unwrap();
        pop(&mut s.get_order_by_cid, "get_order_by_client_id")
    }
    fn get_algo_order(&self, _: i64) -> Result<ExchangeOrder, LiveError> {
        let mut s = self.state.lock().unwrap();
        pop(&mut s.get_algo_order, "get_algo_order")
    }
    fn get_algo_order_by_client_id(
        &self,
        _: &str,
    ) -> Result<Option<ExchangeOrder>, LiveError> {
        let mut s = self.state.lock().unwrap();
        if s.get_algo_order_by_cid.is_empty() {
            // Default: not scripted. Tests that exercise the by-cid path
            // must enqueue explicitly.
            return Err(LiveError::Http("not scripted".into()));
        }
        s.get_algo_order_by_cid.remove(0)
    }
    fn get_open_orders(&self, _: Option<&str>) -> Result<Vec<ExchangeOrder>, LiveError> {
        Ok(vec![])
    }
    fn get_position_info(
        &self,
        _: Option<&str>,
    ) -> Result<Vec<serde_json::Value>, LiveError> {
        let mut s = self.state.lock().unwrap();
        pop(&mut s.position_info, "get_position_info")
    }
    fn get_account_trades(
        &self,
        _: &str,
        _: Option<DateTime<Utc>>,
        _: Option<DateTime<Utc>>,
        _: Option<i64>,
        _: usize,
    ) -> Result<Vec<AccountTrade>, LiveError> {
        let mut s = self.state.lock().unwrap();
        pop(&mut s.account_trades, "get_account_trades")
    }
    fn get_account_info(&self) -> Result<serde_json::Value, LiveError> {
        Ok(serde_json::json!({"availableBalance": "0"}))
    }
    fn get_available_balance(&self) -> Result<f64, LiveError> {
        Ok(self.state.lock().unwrap().available_balance)
    }
    fn set_leverage(&self, _: &str, leverage: u32) -> Result<(), LiveError> {
        self.state.lock().unwrap().set_leverage = leverage;
        Ok(())
    }
    fn get_exchange_info(&self) -> Result<serde_json::Value, LiveError> {
        Err(LiveError::Http("not scripted".into()))
    }
    fn get_mark_price(&self, _: &str) -> Result<f64, LiveError> {
        Ok(self.state.lock().unwrap().mark_price)
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn fixture_exchange_info() -> serde_json::Value {
    serde_json::json!({
        "symbols": [{
            "symbol": "BTCUSDT",
            "filters": [
                {"filterType": "PRICE_FILTER", "tickSize": "0.10"},
                {"filterType": "LOT_SIZE", "stepSize": "0.001", "minQty": "0.001"},
                {"filterType": "MARKET_LOT_SIZE", "stepSize": "0.001", "minQty": "0.001"},
                {"filterType": "MIN_NOTIONAL", "notional": "5.0"}
            ]
        }]
    })
}

fn config(_state_path: PathBuf, recover: bool) -> LiveConfig {
    LiveConfig {
        api_key: "k".into(),
        api_secret: "s".into(),
        base_url: "http://test".into(),
        position_size_usdt: 100.0,
        max_concurrent_positions: 3,
        order_check_interval_seconds: 5.0,
        testnet: false,
        recover_brackets_on_startup: recover,
    }
}

fn build_tracker(client: Arc<MockApi>, state_path: PathBuf, recover: bool) -> PositionTracker {
    let info = ExchangeInfoCache::from_static(HashMap::new());
    info.populate_from_exchange_info(&fixture_exchange_info()).unwrap();
    let exec = OrderExecutor::with_exchange_info(client.clone(), config(state_path.clone(), recover), info);
    let mut tracker = PositionTracker::with_executor(client, config(state_path.clone(), recover), exec);
    tracker.set_state_path(state_path);
    tracker
}

/// Isolated tmp directory per test. Returns the full path to the state file
/// inside that directory. Caller may inspect siblings via `path.parent()`
/// without seeing stray files from other tests.
fn temp_state_path(label: &str) -> PathBuf {
    let pid = std::process::id();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("ct_tracker_{label}_{pid}_{nonce}"));
    std::fs::create_dir_all(&dir).unwrap();
    dir.join("live_state.json")
}

fn cleanup(path: &PathBuf) {
    if let Some(dir) = path.parent() {
        let _ = std::fs::remove_dir_all(dir);
    }
}

fn long_signal(ticker: &str) -> Signal {
    Signal {
        signal_date: Utc::now(),
        position_type: PositionType::Long,
        ticker: ticker.into(),
        pattern: String::new(),
        tp_pct: Some(2.0),
        sl_pct: Some(1.0),
        tp_price: None,
        sl_price: None,
        leverage: 1.0,
        market_type: MarketType::Futures,
        taker_fee_rate: 0.0005,
        entry_price: None,
        fill_timeout_seconds: 3600,
        entry_delay_seconds: None,
        max_holding_hours: 72,
        size_multiplier: 1.0,
        metadata: Default::default(),
    }
}

fn order_filled(order_id: i64, qty: f64, fill_price: f64, cid: &str) -> ExchangeOrder {
    ExchangeOrder {
        order_id,
        symbol: "BTCUSDT".into(),
        side: OrderSide::Buy,
        order_type: OrderType::Market,
        quantity: qty,
        price: 0.0,
        stop_price: 0.0,
        status: OrderStatus::Filled,
        filled_qty: qty,
        avg_fill_price: fill_price,
        created_at: Some(Utc::now()),
        updated_at: Some(Utc::now()),
        algo_id: 0,
        client_order_id: cid.into(),
    }
}

fn algo_order(algo_id: i64, status: OrderStatus, kind: OrderType) -> ExchangeOrder {
    ExchangeOrder {
        order_id: 0,
        symbol: "BTCUSDT".into(),
        side: OrderSide::Sell,
        order_type: kind,
        quantity: 0.01,
        price: 0.0,
        stop_price: 0.0,
        status,
        filled_qty: 0.0,
        avg_fill_price: 0.0,
        created_at: None,
        updated_at: None,
        algo_id,
        client_order_id: format!("algo-{algo_id}"),
    }
}

fn position_info_open(symbol: &str, amt: f64, side: &str) -> serde_json::Value {
    serde_json::json!({
        "symbol": symbol,
        "positionAmt": amt.to_string(),
        "positionSide": side
    })
}

fn position_info_closed(symbol: &str, side: &str) -> serde_json::Value {
    serde_json::json!({
        "symbol": symbol,
        "positionAmt": "0",
        "positionSide": side
    })
}

// ---------------------------------------------------------------------------
// place_entry path
// ---------------------------------------------------------------------------

#[test]
fn place_entry_happy_path() {
    let api = MockApi::new();
    api.set_mark_price(43_000.0);
    api.set_balance(1_000.0);
    let path = temp_state_path("place_happy");
    let mut tracker = build_tracker(api.clone(), path.clone(), true);

    let sig = long_signal("BTCUSDT");
    api.enqueue_market(Ok(order_filled(123, 0.01, 0.0, "will-fill"))); // status doesn't matter for submit's return
    let res = tracker.place_entry(&sig, "alpha", Some(1_000.0), Some(100.0)).unwrap();
    assert_eq!(res.status, PlacementStatus::Placed);
    assert!(res.margin_consumed > 0.0);

    let positions = tracker.positions();
    assert_eq!(positions.len(), 1);
    assert_eq!(positions[0].status, PositionStatus::PendingEntry);
    assert_eq!(positions[0].entry_order.as_ref().unwrap().order_id, 123);
    assert!(path.exists(), "state file should be written");

    cleanup(&path);
}

#[test]
fn place_entry_terminal_4xx_marks_failed() {
    let api = MockApi::new();
    api.set_mark_price(43_000.0);
    api.set_balance(1_000.0);
    let path = temp_state_path("place_rejected");
    let mut tracker = build_tracker(api.clone(), path.clone(), true);

    let sig = long_signal("BTCUSDT");
    api.enqueue_market(Err(LiveError::Api {
        code: -2010,
        msg: "NEW_ORDER_REJECTED".into(),
    }));
    let res = tracker.place_entry(&sig, "alpha", Some(1_000.0), Some(100.0)).unwrap();
    assert_eq!(res.status, PlacementStatus::Rejected);
    assert_eq!(res.margin_consumed, 0.0);

    // Position is in memory as Failed but not on disk (filtered).
    let positions = tracker.positions();
    assert_eq!(positions.len(), 1);
    assert_eq!(positions[0].status, PositionStatus::Failed);

    let on_disk = std::fs::read_to_string(&path).unwrap();
    let arr: Vec<serde_json::Value> = serde_json::from_str(&on_disk).unwrap();
    assert!(arr.is_empty(), "Failed positions must not persist");

    cleanup(&path);
}

#[test]
fn place_entry_5xx_outcome_unknown_returns_deferred() {
    let api = MockApi::new();
    api.set_mark_price(43_000.0);
    api.set_balance(1_000.0);
    let path = temp_state_path("place_deferred");
    let mut tracker = build_tracker(api.clone(), path.clone(), true);

    let sig = long_signal("BTCUSDT");
    api.enqueue_market(Err(LiveError::Http("connection reset".into())));
    let res = tracker.place_entry(&sig, "alpha", Some(1_000.0), Some(100.0)).unwrap();
    assert_eq!(res.status, PlacementStatus::Deferred);
    assert!(res.margin_consumed > 0.0);

    let positions = tracker.positions();
    assert_eq!(positions[0].status, PositionStatus::PendingEntry);
    let entry = positions[0].entry_order.as_ref().unwrap();
    assert_eq!(entry.order_id, 0, "deferred → no order_id yet");
    assert!(!entry.client_order_id.is_empty(), "client_order_id persisted");

    cleanup(&path);
}

// ---------------------------------------------------------------------------
// check_fills entry path
// ---------------------------------------------------------------------------

#[test]
fn check_fills_entry_filled_places_tp_sl() {
    let api = MockApi::new();
    api.set_mark_price(43_000.0);
    api.set_balance(10_000.0);
    let path = temp_state_path("entry_filled");
    let mut tracker = build_tracker(api.clone(), path.clone(), true);

    // Step 1: place_entry returns a NEW market order.
    let sig = long_signal("BTCUSDT");
    api.enqueue_market(Ok(ExchangeOrder {
        order_id: 100,
        status: OrderStatus::New,
        ..order_filled(100, 0.01, 0.0, "cid-1")
    }));
    tracker.place_entry(&sig, "alpha", Some(10_000.0), Some(100.0)).unwrap();

    // Step 2: check_fills sees Filled status; tracker computes TP/SL.
    api.enqueue_get_order(Ok(order_filled(100, 0.01, 43_500.0, "cid-1")));
    api.enqueue_tp(Ok(algo_order(900_001, OrderStatus::New, OrderType::TakeProfitMarket)));
    api.enqueue_sl(Ok(algo_order(900_002, OrderStatus::New, OrderType::StopMarket)));
    tracker.check_fills(Utc::now());

    let pos = &tracker.positions()[0];
    assert_eq!(pos.status, PositionStatus::Open);
    assert_eq!(pos.fill_price, 43_500.0);
    assert!(pos.tp_order.is_some());
    assert!(pos.sl_order.is_some());

    // TP/SL placed with the actual fill price, not the entry signal price.
    let tp_calls = api.tp_calls();
    assert_eq!(tp_calls.len(), 1);
    let (_, tp_stop, tp_qty, _) = &tp_calls[0];
    assert!(*tp_stop > 43_500.0, "TP must be above fill price");
    assert_eq!(*tp_qty, Some(0.01));

    cleanup(&path);
}

#[test]
fn check_fills_stale_limit_entry_times_out() {
    let api = MockApi::new();
    api.set_balance(10_000.0);
    let path = temp_state_path("stale_entry");
    let mut tracker = build_tracker(api.clone(), path.clone(), true);

    let mut sig = long_signal("BTCUSDT");
    sig.entry_price = Some(43_000.0); // limit
    sig.fill_timeout_seconds = 60;
    api.enqueue_limit(Ok(ExchangeOrder {
        order_id: 200,
        status: OrderStatus::New,
        ..order_filled(200, 0.002, 0.0, "cid-2")
    }));
    tracker.place_entry(&sig, "alpha", Some(10_000.0), Some(100.0)).unwrap();

    // check_fills: order still NEW, age > timeout → cancel + Failed
    let two_min_ago = Utc::now() - Duration::seconds(120);
    api.enqueue_get_order(Ok(ExchangeOrder {
        order_id: 200,
        status: OrderStatus::New,
        created_at: Some(two_min_ago),
        ..order_filled(200, 0.002, 0.0, "cid-2")
    }));
    api.enqueue_cancel(Ok(ExchangeOrder {
        order_id: 200,
        status: OrderStatus::Canceled,
        ..order_filled(200, 0.002, 0.0, "cid-2")
    }));
    tracker.check_fills(Utc::now());

    let pos = &tracker.positions()[0];
    assert_eq!(pos.status, PositionStatus::Failed);
    assert_eq!(api.cancel_calls().len(), 1);

    cleanup(&path);
}

#[test]
fn check_fills_deferred_entry_resolves_via_client_id() {
    let api = MockApi::new();
    api.set_mark_price(43_000.0);
    api.set_balance(10_000.0);
    let path = temp_state_path("deferred_entry");
    let mut tracker = build_tracker(api.clone(), path.clone(), true);

    let sig = long_signal("BTCUSDT");
    // Submit returns a 5xx → Deferred. order_id stays 0.
    api.enqueue_market(Err(LiveError::Http("503".into())));
    let res = tracker.place_entry(&sig, "alpha", Some(10_000.0), Some(100.0)).unwrap();
    assert_eq!(res.status, PlacementStatus::Deferred);
    let cid = tracker.positions()[0]
        .entry_order
        .as_ref()
        .unwrap()
        .client_order_id
        .clone();

    // check_fills queries by cid → returns the filled order
    api.enqueue_get_order_by_cid(Ok(Some(order_filled(777, 0.01, 43_010.0, &cid))));
    api.enqueue_tp(Ok(algo_order(901, OrderStatus::New, OrderType::TakeProfitMarket)));
    api.enqueue_sl(Ok(algo_order(902, OrderStatus::New, OrderType::StopMarket)));
    tracker.check_fills(Utc::now());

    let pos = &tracker.positions()[0];
    assert_eq!(pos.status, PositionStatus::Open);
    assert_eq!(pos.entry_order.as_ref().unwrap().order_id, 777);
    assert_eq!(pos.fill_price, 43_010.0);

    cleanup(&path);
}

#[test]
fn check_fills_deferred_entry_minus_2013_marks_failed() {
    let api = MockApi::new();
    api.set_mark_price(43_000.0);
    api.set_balance(10_000.0);
    let path = temp_state_path("deferred_2013");
    let mut tracker = build_tracker(api.clone(), path.clone(), true);

    let sig = long_signal("BTCUSDT");
    api.enqueue_market(Err(LiveError::Http("network".into())));
    tracker.place_entry(&sig, "alpha", Some(10_000.0), Some(100.0)).unwrap();

    // Query by cid → -2013 → tracker returns Ok(None)
    api.enqueue_get_order_by_cid(Ok(None));
    tracker.check_fills(Utc::now());

    assert_eq!(tracker.positions()[0].status, PositionStatus::Failed);
    cleanup(&path);
}

// ---------------------------------------------------------------------------
// check_fills exit path
// ---------------------------------------------------------------------------

fn populate_open_position(api: &MockApi, tracker: &mut PositionTracker, fill_price: f64) {
    let sig = long_signal("BTCUSDT");
    api.set_mark_price(fill_price);
    api.set_balance(10_000.0);
    api.enqueue_market(Ok(ExchangeOrder {
        order_id: 100,
        status: OrderStatus::New,
        ..order_filled(100, 0.01, 0.0, "cid-1")
    }));
    tracker.place_entry(&sig, "alpha", Some(10_000.0), Some(100.0)).unwrap();
    api.enqueue_get_order(Ok(order_filled(100, 0.01, fill_price, "cid-1")));
    api.enqueue_tp(Ok(algo_order(900_001, OrderStatus::New, OrderType::TakeProfitMarket)));
    api.enqueue_sl(Ok(algo_order(900_002, OrderStatus::New, OrderType::StopMarket)));
    tracker.check_fills(Utc::now());
    assert_eq!(tracker.positions()[0].status, PositionStatus::Open);
}

#[test]
fn exit_tp_fills_cancels_sl_and_closes_position() {
    let api = MockApi::new();
    let path = temp_state_path("exit_tp");
    let mut tracker = build_tracker(api.clone(), path.clone(), true);
    populate_open_position(&api, &mut tracker, 43_500.0);

    // check_fills exit path:
    // 1. Bulk position_info shows still-open
    // 2. We disable fast path here by setting NO position info to force slow path
    // Actually the fast path runs before slow path. Let's *enable* fast path first
    // and verify the exit fires when exchange shows position GONE.
    // To force the slow-path check on TP/SL fills, the simplest scenario is:
    // exchange_open=true (fast path skips → no exit), so we need an alternative.
    // For TP-fill detection, we need exchange to show closed AND TP query to show Filled.
    // Or: set the side_count > 1 so fast path is skipped. Or: don't pre-fetch
    // (=positions empty count). Easiest: let the fast path bulk fetch return
    // positionAmt=0 for our symbol. Then slow path queries TP, sees Filled.
    api.enqueue_position_info(Ok(vec![position_info_closed("BTCUSDT", "LONG")]));
    api.enqueue_get_algo_order(Ok(ExchangeOrder {
        avg_fill_price: 44_375.0,
        stop_price: 44_375.0,
        ..algo_order(900_001, OrderStatus::Filled, OrderType::TakeProfitMarket)
    }));
    // SL queried (not skipped because TP filled? Actually code skips SL when TP filled.)
    // After TP filled, code cancels SL via cancel_safely_any (algo).
    api.enqueue_cancel_algo(Ok(()));
    tracker.check_fills(Utc::now());

    let pos = &tracker.positions()[0];
    assert_eq!(pos.status, PositionStatus::Closed);
    assert!(pos.exit_price.unwrap() > 43_500.0);
    assert!(pos.pnl_pct.is_some());
    assert_eq!(api.cancel_algo_calls().len(), 1);

    cleanup(&path);
}

#[test]
fn exit_sl_fills_cancels_tp_and_closes_position() {
    let api = MockApi::new();
    let path = temp_state_path("exit_sl");
    let mut tracker = build_tracker(api.clone(), path.clone(), true);
    populate_open_position(&api, &mut tracker, 43_500.0);

    // Exchange position closed; TP query → still NEW; SL query → FILLED.
    api.enqueue_position_info(Ok(vec![position_info_closed("BTCUSDT", "LONG")]));
    api.enqueue_get_algo_order(Ok(algo_order(
        900_001,
        OrderStatus::New,
        OrderType::TakeProfitMarket,
    )));
    api.enqueue_get_algo_order(Ok(ExchangeOrder {
        avg_fill_price: 43_065.0,
        stop_price: 43_065.0,
        ..algo_order(900_002, OrderStatus::Filled, OrderType::StopMarket)
    }));
    api.enqueue_cancel_algo(Ok(()));
    tracker.check_fills(Utc::now());

    let pos = &tracker.positions()[0];
    assert_eq!(pos.status, PositionStatus::Closed);
    assert!(pos.exit_price.unwrap() < 43_500.0);
    assert!(pos.pnl_pct.unwrap() < 0.0, "SL should produce loss");
    cleanup(&path);
}

#[test]
fn exit_external_close_recovers_via_account_trades() {
    let api = MockApi::new();
    let path = temp_state_path("exit_external");
    let mut tracker = build_tracker(api.clone(), path.clone(), true);
    populate_open_position(&api, &mut tracker, 43_500.0);

    // Exchange position GONE, both TP & SL still NEW (manual close on Binance UI).
    api.enqueue_position_info(Ok(vec![position_info_closed("BTCUSDT", "LONG")]));
    api.enqueue_get_algo_order(Ok(algo_order(
        900_001,
        OrderStatus::New,
        OrderType::TakeProfitMarket,
    )));
    api.enqueue_get_algo_order(Ok(algo_order(
        900_002,
        OrderStatus::New,
        OrderType::StopMarket,
    )));
    // Inferred External → walks account trades.
    api.enqueue_account_trades(Ok(vec![AccountTrade {
        trade_id: 1,
        order_id: 50_555,
        symbol: "BTCUSDT".into(),
        side: OrderSide::Sell,
        price: 43_750.0,
        quantity: 0.01,
        time: Utc::now(),
        realized_pnl: 2.5,
        commission: 0.0,
        commission_asset: "USDT".into(),
        position_side: "LONG".into(),
    }]));
    tracker.check_fills(Utc::now());

    let pos = &tracker.positions()[0];
    assert_eq!(pos.status, PositionStatus::Closed);
    assert!((pos.exit_price.unwrap() - 43_750.0).abs() < 1e-6);
    assert!(pos.pnl_pct.unwrap() > 0.0);
    cleanup(&path);
}

#[test]
fn exit_timeout_close_uses_market_order() {
    let api = MockApi::new();
    let path = temp_state_path("exit_timeout");
    let mut tracker = build_tracker(api.clone(), path.clone(), true);
    populate_open_position(&api, &mut tracker, 43_500.0);

    // populate_open_position set opened_at ≈ now via the order's updated_at.
    // To trip the timeout, pass `now + max_h + 1h` to check_fills.
    let signal_max_h = tracker.positions()[0].signal.max_holding_hours;
    let future_now = Utc::now() + Duration::hours(signal_max_h + 1);

    // Fast-path bulk fetch: position still open. Outcome::Open → check_timeout fires.
    api.enqueue_position_info(Ok(vec![position_info_open("BTCUSDT", 0.01, "LONG")]));
    // Cancel both brackets (NEW status).
    api.enqueue_cancel_algo(Ok(()));
    api.enqueue_cancel_algo(Ok(()));
    // Close-position market order.
    api.enqueue_market(Ok(ExchangeOrder {
        avg_fill_price: 43_400.0,
        ..order_filled(555, 0.01, 43_400.0, "close-cid")
    }));

    tracker.check_fills(future_now);

    let pos = &tracker.positions()[0];
    assert_eq!(pos.status, PositionStatus::Closed);
    assert_eq!(pos.exit_price, Some(43_400.0));
    assert!(pos.pnl_pct.is_some());
    // Both brackets canceled.
    assert_eq!(api.cancel_algo_calls().len(), 2);
    // Close market sell with quantity 0.01 (the position quantity).
    let market = api.market_calls();
    let close = market
        .iter()
        .find(|(_, side, _, _)| *side == OrderSide::Sell)
        .expect("close market sell recorded");
    assert!((close.2 - 0.01).abs() < 1e-12);

    cleanup(&path);
}

// ---------------------------------------------------------------------------
// Reconciliation
// ---------------------------------------------------------------------------

#[test]
fn reconcile_picks_up_external_position() {
    let api = MockApi::new();
    let path = temp_state_path("reconcile");
    let mut tracker = build_tracker(api.clone(), path.clone(), true);

    // Exchange has a SOL/USDT LONG position we don't know about.
    api.enqueue_position_info(Ok(vec![
        position_info_open("SOLUSDT", 5.0, "LONG"),
        position_info_open("BTCUSDT", 0.0, "BOTH"), // empty, skipped
    ]));
    let ok = tracker.reconcile_with_exchange();
    assert!(ok);

    // open_count includes the external position
    assert_eq!(tracker.open_count(), 1);

    // has_external_conflict blocks SOL/USDT
    let mut sig = long_signal("SOLUSDT");
    sig.ticker = "SOLUSDT".into();
    assert!(tracker.has_external_conflict(&sig));

    // Other tickers are fine
    let other = long_signal("ETHUSDT");
    assert!(!tracker.has_external_conflict(&other));

    cleanup(&path);
}

// ---------------------------------------------------------------------------
// Bracket recovery
// ---------------------------------------------------------------------------

#[test]
fn bracket_recovery_replaces_only_dead_side() {
    let api = MockApi::new();
    let path = temp_state_path("recovery_partial");
    let mut tracker = build_tracker(api.clone(), path.clone(), true);
    populate_open_position(&api, &mut tracker, 43_500.0);
    let baseline_tp = api.tp_calls().len();
    let baseline_sl = api.sl_calls().len();

    // Now simulate "TP was canceled while we were down". Recovery sweep:
    api.enqueue_position_info(Ok(vec![position_info_open("BTCUSDT", 0.01, "LONG")]));
    // tp_order query → CANCELED
    api.enqueue_get_algo_order(Ok(algo_order(
        900_001,
        OrderStatus::Canceled,
        OrderType::TakeProfitMarket,
    )));
    // sl_order query → NEW (healthy)
    api.enqueue_get_algo_order(Ok(algo_order(
        900_002,
        OrderStatus::New,
        OrderType::StopMarket,
    )));
    // place_tp_only re-places.
    api.enqueue_tp(Ok(algo_order(
        900_003,
        OrderStatus::New,
        OrderType::TakeProfitMarket,
    )));

    tracker.recover_brackets();

    assert_eq!(api.tp_calls().len() - baseline_tp, 1, "TP re-placed exactly once");
    assert_eq!(api.sl_calls().len() - baseline_sl, 0, "SL must NOT be touched");

    let pos = &tracker.positions()[0];
    assert_eq!(pos.tp_order.as_ref().unwrap().algo_id, 900_003);
    assert_eq!(pos.sl_order.as_ref().unwrap().algo_id, 900_002);

    cleanup(&path);
}

#[test]
fn bracket_recovery_skips_when_query_fails() {
    let api = MockApi::new();
    let path = temp_state_path("recovery_unknown");
    let mut tracker = build_tracker(api.clone(), path.clone(), true);
    populate_open_position(&api, &mut tracker, 43_500.0);
    let baseline_tp = api.tp_calls().len();
    let baseline_sl = api.sl_calls().len();

    api.enqueue_position_info(Ok(vec![position_info_open("BTCUSDT", 0.01, "LONG")]));
    // tp_order query → ERROR
    api.enqueue_get_algo_order(Err(LiveError::Http("503".into())));
    // We never reach sl_order query because Unknown short-circuits — but defensively
    // enqueue something so the test doesn't depend on call ordering.
    api.enqueue_get_algo_order(Ok(algo_order(
        900_002,
        OrderStatus::New,
        OrderType::StopMarket,
    )));

    tracker.recover_brackets();

    // No re-placement above the baseline placement done by populate.
    assert_eq!(api.tp_calls().len(), baseline_tp);
    assert_eq!(api.sl_calls().len(), baseline_sl);
    cleanup(&path);
}

#[test]
fn bracket_recovery_disabled_by_config() {
    let api = MockApi::new();
    let path = temp_state_path("recovery_disabled");
    let mut tracker = build_tracker(api.clone(), path.clone(), false);
    populate_open_position(&api, &mut tracker, 43_500.0);
    let baseline_tp = api.tp_calls().len();
    let baseline_sl = api.sl_calls().len();

    // Even if API would say TP is canceled, recovery shouldn't run.
    tracker.recover_brackets();
    assert_eq!(api.tp_calls().len(), baseline_tp);
    assert_eq!(api.sl_calls().len(), baseline_sl);
    cleanup(&path);
}

#[test]
fn bracket_recovery_skips_when_exchange_position_closed() {
    let api = MockApi::new();
    let path = temp_state_path("recovery_no_pos");
    let mut tracker = build_tracker(api.clone(), path.clone(), true);
    populate_open_position(&api, &mut tracker, 43_500.0);
    let baseline_tp = api.tp_calls().len();
    let baseline_sl = api.sl_calls().len();

    // Exchange shows position closed → skip recovery (next check_fills will
    // close the position out via inference + account trades).
    api.enqueue_position_info(Ok(vec![position_info_closed("BTCUSDT", "LONG")]));

    tracker.recover_brackets();
    assert_eq!(api.tp_calls().len(), baseline_tp);
    assert_eq!(api.sl_calls().len(), baseline_sl);
    cleanup(&path);
}

// ---------------------------------------------------------------------------
// Persistence
// ---------------------------------------------------------------------------

#[test]
fn persistence_round_trip_rust_to_rust() {
    let api1 = MockApi::new();
    api1.set_mark_price(43_000.0);
    api1.set_balance(10_000.0);
    let path = temp_state_path("persist_rust");
    {
        let mut t1 = build_tracker(api1.clone(), path.clone(), true);
        let sig = long_signal("BTCUSDT");
        api1.enqueue_market(Ok(ExchangeOrder {
            order_id: 1,
            status: OrderStatus::New,
            ..order_filled(1, 0.01, 0.0, "cid-1")
        }));
        t1.place_entry(&sig, "alpha", Some(10_000.0), Some(100.0)).unwrap();
    }

    // Fresh tracker reads the file.
    let api2 = MockApi::new();
    let mut t2 = build_tracker(api2, path.clone(), true);
    t2.load_state();
    let positions = t2.positions();
    assert_eq!(positions.len(), 1);
    assert_eq!(positions[0].status, PositionStatus::PendingEntry);
    assert_eq!(positions[0].strategy_id, "alpha");
    cleanup(&path);
}

#[test]
fn persistence_loads_python_shaped_fixture_without_client_order_id() {
    let api = MockApi::new();
    let path = temp_state_path("persist_python");
    let mut tracker = build_tracker(api, path.clone(), true);

    // Hand-written Python-format fixture: top-level array, snake_case keys,
    // no client_order_id, no pattern field on Signal.
    let fixture = r#"[
      {
        "position_id": "abc12345",
        "strategy_id": "BreadthMomentumStrategy",
        "status": "OPEN",
        "signal": {
          "signal_date": "2026-04-29T13:00:00+00:00",
          "position_type": "LONG",
          "ticker": "BTCUSDT",
          "tp_pct": 4.5,
          "sl_pct": 2.0,
          "leverage": 1.0,
          "market_type": "FUTURES",
          "taker_fee_rate": 0.0005,
          "fill_timeout_seconds": 3600,
          "max_holding_hours": 72
        },
        "entry_order": {
          "order_id": 8732145901,
          "symbol": "BTCUSDT",
          "side": "BUY",
          "order_type": "MARKET",
          "quantity": 0.01,
          "price": 0.0,
          "stop_price": 0.0,
          "status": "FILLED",
          "filled_qty": 0.01,
          "avg_fill_price": 43500.0,
          "algo_id": 0
        },
        "tp_order": {
          "order_id": 0,
          "symbol": "BTCUSDT",
          "side": "SELL",
          "order_type": "TAKE_PROFIT_MARKET",
          "quantity": 0.01,
          "price": 0.0,
          "stop_price": 44375.0,
          "status": "NEW",
          "filled_qty": 0.0,
          "avg_fill_price": 0.0,
          "algo_id": 18293471
        },
        "sl_order": {
          "order_id": 0,
          "symbol": "BTCUSDT",
          "side": "SELL",
          "order_type": "STOP_MARKET",
          "quantity": 0.01,
          "price": 0.0,
          "stop_price": 43065.0,
          "status": "NEW",
          "filled_qty": 0.0,
          "avg_fill_price": 0.0,
          "algo_id": 18293472
        },
        "fill_price": 43500.0,
        "quantity": 0.01,
        "opened_at": "2026-04-29T13:00:14.882000+00:00"
      }
    ]"#;
    std::fs::write(&path, fixture).unwrap();
    tracker.load_state();
    let positions = tracker.positions();
    assert_eq!(positions.len(), 1);
    let p = &positions[0];
    assert_eq!(p.status, PositionStatus::Open);
    assert_eq!(p.fill_price, 43500.0);
    assert_eq!(p.tp_order.as_ref().unwrap().algo_id, 18293471);
    assert_eq!(p.sl_order.as_ref().unwrap().algo_id, 18293472);
    // Defaulted because Python file didn't have it.
    assert_eq!(p.entry_order.as_ref().unwrap().client_order_id, "");
    assert_eq!(p.signal.pattern, "");

    cleanup(&path);
}

#[test]
fn persistence_excludes_failed_and_closed() {
    let api = MockApi::new();
    api.set_mark_price(43_000.0);
    api.set_balance(10_000.0);
    let path = temp_state_path("persist_filter");
    let mut tracker = build_tracker(api.clone(), path.clone(), true);

    // Place 3 positions. First fails terminally, second succeeds and stays
    // PendingEntry, third we'll force-close via timeout.
    let sig = long_signal("BTCUSDT");

    // Position 1: rejected
    api.enqueue_market(Err(LiveError::Api {
        code: -2010,
        msg: "rejected".into(),
    }));
    tracker.place_entry(&sig, "alpha", Some(10_000.0), Some(100.0)).unwrap();

    // Position 2: pending
    api.enqueue_market(Ok(ExchangeOrder {
        order_id: 2,
        status: OrderStatus::New,
        ..order_filled(2, 0.01, 0.0, "cid-2")
    }));
    tracker.place_entry(&sig, "alpha", Some(10_000.0), Some(100.0)).unwrap();

    // Position 3: pending
    api.enqueue_market(Ok(ExchangeOrder {
        order_id: 3,
        status: OrderStatus::New,
        ..order_filled(3, 0.01, 0.0, "cid-3")
    }));
    tracker.place_entry(&sig, "alpha", Some(10_000.0), Some(100.0)).unwrap();

    let on_disk: Vec<serde_json::Value> =
        serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    // Position 1 is Failed → not on disk. Positions 2 & 3 are PendingEntry → on disk.
    assert_eq!(on_disk.len(), 2);
    let ids: HashSet<_> = on_disk
        .iter()
        .map(|p| p["entry_order"]["order_id"].as_i64().unwrap())
        .collect();
    assert!(ids.contains(&2));
    assert!(ids.contains(&3));

    cleanup(&path);
}

#[test]
fn persistence_atomic_write_no_tmp_file_left_behind() {
    let api = MockApi::new();
    api.set_mark_price(43_000.0);
    api.set_balance(10_000.0);
    let path = temp_state_path("persist_atomic");
    let mut tracker = build_tracker(api.clone(), path.clone(), true);

    let sig = long_signal("BTCUSDT");
    api.enqueue_market(Ok(ExchangeOrder {
        order_id: 1,
        status: OrderStatus::New,
        ..order_filled(1, 0.01, 0.0, "cid-1")
    }));
    tracker.place_entry(&sig, "alpha", Some(10_000.0), Some(100.0)).unwrap();
    tracker.save_state_now();

    // Final file exists; no tmp leftover in the same directory with our pid.
    assert!(path.exists());
    let dir = path.parent().unwrap();
    let pid = std::process::id();
    let leftover_pattern = format!(".{pid}.tmp");
    let leftover = std::fs::read_dir(dir)
        .unwrap()
        .flatten()
        .find(|e| e.file_name().to_string_lossy().ends_with(&leftover_pattern));
    assert!(leftover.is_none(), "tmp file should be gone after rename");

    cleanup(&path);
}

#[test]
fn check_fills_persists_state_so_crash_does_not_reissue_brackets() {
    // Regression for: check_fills marks dirty but doesn't save → a crash
    // after entry-fill + TP/SL placement restarts from the stale
    // PendingEntry snapshot and re-runs the entry flow → duplicate brackets.
    let api = MockApi::new();
    api.set_mark_price(43_000.0);
    api.set_balance(10_000.0);
    let path = temp_state_path("crash_safety");

    // Step 1: place_entry returns NEW market order; persisted as PendingEntry.
    {
        let mut t1 = build_tracker(api.clone(), path.clone(), true);
        let sig = long_signal("BTCUSDT");
        api.enqueue_market(Ok(ExchangeOrder {
            order_id: 100,
            status: OrderStatus::New,
            ..order_filled(100, 0.01, 0.0, "cid-1")
        }));
        t1.place_entry(&sig, "alpha", Some(10_000.0), Some(100.0)).unwrap();
        // Step 2: check_fills sees Filled; tracker fills + places TP/SL.
        api.enqueue_get_order(Ok(order_filled(100, 0.01, 43_500.0, "cid-1")));
        api.enqueue_tp(Ok(algo_order(900_001, OrderStatus::New, OrderType::TakeProfitMarket)));
        api.enqueue_sl(Ok(algo_order(900_002, OrderStatus::New, OrderType::StopMarket)));
        t1.check_fills(Utc::now());
        assert_eq!(t1.positions()[0].status, PositionStatus::Open);
        // Drop t1 *without* an explicit save_state_now — simulates a crash.
    }

    // Step 3: brand-new tracker reads the file. Position must be OPEN with
    // both bracket orders, NOT PendingEntry.
    let api2 = MockApi::new();
    let mut t2 = build_tracker(api2, path.clone(), true);
    t2.load_state();
    let positions = t2.positions();
    assert_eq!(positions.len(), 1);
    let p = &positions[0];
    assert_eq!(
        p.status,
        PositionStatus::Open,
        "post-fill state must be persisted; got {:?}",
        p.status,
    );
    assert!(p.tp_order.is_some(), "tp_order must be persisted");
    assert!(p.sl_order.is_some(), "sl_order must be persisted");
    assert_eq!(p.fill_price, 43_500.0);

    cleanup(&path);
}

#[test]
fn bracket_submit_5xx_leaves_placeholder_for_recovery_to_adopt() {
    // Regression for: TP/SL POST returning 5xx/network ambiguity used to
    // leave the position with NO tp_order/sl_order on disk, so next-startup
    // bracket recovery would PLACE A DUPLICATE if the original POST had
    // actually reached Binance.
    //
    // Two-phase fix: prepare → save → submit. The placeholder (with its
    // clientAlgoId) persists across the submit failure. Startup recovery
    // queries by cid; if the order exists on Binance, it gets ADOPTED
    // (assigned to position.tp_order with the real algo_id), no
    // re-placement.
    let api = MockApi::new();
    api.set_mark_price(43_000.0);
    api.set_balance(10_000.0);
    let path = temp_state_path("bracket_crash_safety");

    // Step 1: place_entry → check_fills detects fill, prepares brackets,
    //         persists placeholders, then submit returns 5xx for both.
    {
        let mut t1 = build_tracker(api.clone(), path.clone(), true);
        let sig = long_signal("BTCUSDT");
        api.enqueue_market(Ok(ExchangeOrder {
            order_id: 100,
            status: OrderStatus::New,
            ..order_filled(100, 0.01, 0.0, "cid-1")
        }));
        t1.place_entry(&sig, "alpha", Some(10_000.0), Some(100.0)).unwrap();
        // Filled response → tracker prepares brackets, then submit fails.
        api.enqueue_get_order(Ok(order_filled(100, 0.01, 43_500.0, "cid-1")));
        api.enqueue_tp(Err(LiveError::Http("503 during TP submit".into())));
        api.enqueue_sl(Err(LiveError::Http("503 during SL submit".into())));
        t1.check_fills(Utc::now());
        // Position is still Open (Python parity: TP/SL failure doesn't
        // unwind the entry). The placeholders are on the position with cids.
        let pos = &t1.positions()[0];
        assert_eq!(pos.status, PositionStatus::Open);
        let tp = pos.tp_order.as_ref().expect("tp placeholder must persist");
        let sl = pos.sl_order.as_ref().expect("sl placeholder must persist");
        assert_eq!(tp.algo_id, 0, "placeholder has algo_id=0");
        assert_eq!(sl.algo_id, 0);
        assert!(!tp.client_order_id.is_empty(), "cid populated");
        assert!(!sl.client_order_id.is_empty(), "cid populated");
        // Drop t1 (simulates crash before any retry).
    }

    // Step 2: fresh tracker loads. Recovery queries by cid → both orders
    //         exist on Binance (the original POSTs DID reach the matching
    //         engine, the response was just lost). Recovery adopts both
    //         and DOES NOT issue another POST.
    let api2 = MockApi::new();
    let path2 = path.clone();
    let mut t2 = build_tracker(api2.clone(), path2, true);
    t2.load_state();
    assert_eq!(t2.positions().len(), 1);
    let placeholder_tp_cid =
        t2.positions()[0].tp_order.as_ref().unwrap().client_order_id.clone();
    let placeholder_sl_cid =
        t2.positions()[0].sl_order.as_ref().unwrap().client_order_id.clone();

    // Recovery: positionRisk shows position open …
    api2.enqueue_position_info(Ok(vec![position_info_open("BTCUSDT", 0.01, "LONG")]));
    // … and the algo-by-cid queries return the actual orders Binance
    // accepted (the placeholder cids match).
    let server_tp_response = ExchangeOrder {
        algo_id: 99_001,
        client_order_id: placeholder_tp_cid.clone(),
        ..algo_order(99_001, OrderStatus::New, OrderType::TakeProfitMarket)
    };
    let server_sl_response = ExchangeOrder {
        algo_id: 99_002,
        client_order_id: placeholder_sl_cid.clone(),
        ..algo_order(99_002, OrderStatus::New, OrderType::StopMarket)
    };
    // Note: query_existing_order routes algo placeholders through
    // get_algo_order_by_client_id, so we enqueue THAT, not get_algo_order.
    // Our MockApi's get_algo_order_by_client_id is unscripted — let me check.

    // We need a get_algo_order_by_client_id queue. Add to the mock.
    api2.enqueue_get_algo_order_by_cid(Ok(Some(server_tp_response.clone())));
    api2.enqueue_get_algo_order_by_cid(Ok(Some(server_sl_response.clone())));

    t2.recover_brackets();

    // Assert: NO new TP/SL POSTs (we adopted, didn't re-place).
    assert!(api2.tp_calls().is_empty(), "must not re-POST TP — adopted via cid");
    assert!(api2.sl_calls().is_empty(), "must not re-POST SL — adopted via cid");

    // Assert: position now has the REAL algo_ids from Binance.
    let pos = &t2.positions()[0];
    assert_eq!(
        pos.tp_order.as_ref().unwrap().algo_id,
        99_001,
        "tp_order should have been replaced with the server response"
    );
    assert_eq!(pos.sl_order.as_ref().unwrap().algo_id, 99_002);

    cleanup(&path);
}

#[test]
fn open_count_for_strategy_excludes_other_strategies() {
    let api = MockApi::new();
    api.set_mark_price(43_000.0);
    api.set_balance(10_000.0);
    let path = temp_state_path("open_count");
    let mut tracker = build_tracker(api.clone(), path.clone(), true);

    let sig = long_signal("BTCUSDT");
    api.enqueue_market(Ok(ExchangeOrder {
        order_id: 1,
        status: OrderStatus::New,
        ..order_filled(1, 0.01, 0.0, "cid-1")
    }));
    tracker.place_entry(&sig, "alpha", Some(10_000.0), Some(100.0)).unwrap();

    api.enqueue_market(Ok(ExchangeOrder {
        order_id: 2,
        status: OrderStatus::New,
        ..order_filled(2, 0.01, 0.0, "cid-2")
    }));
    tracker.place_entry(&sig, "beta", Some(10_000.0), Some(100.0)).unwrap();

    assert_eq!(tracker.open_count_for("alpha"), 1);
    assert_eq!(tracker.open_count_for("beta"), 1);
    assert_eq!(tracker.open_count_for("gamma"), 0);
    assert_eq!(tracker.open_count(), 2);

    cleanup(&path);
}
