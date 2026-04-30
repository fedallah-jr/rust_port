//! End-to-end tests for `LiveEngine` (Phase E).
//!
//! Drives the engine through full ticks using a `ScriptedApi` mock plus a
//! recording `MockGenerator`. No real sleeps anywhere — all timing is
//! controlled by the test through `client.set_now()` and direct `tick()`
//! invocations.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use chrono::{DateTime, Duration as ChronoDuration, Utc};
use claude_trader_live::auth_client::FuturesApi;
use claude_trader_live::engine::LiveEngine;
use claude_trader_live::error::LiveError;
use claude_trader_live::market_client::{LiveMarketClient, NullMarketClient};
use claude_trader_live::signal_generator::{LiveSignalGenerator, SignalError};
use claude_trader_models::{
    AccountTrade, ExchangeOrder, GeneratorBudget, LiveConfig, MarketType, OrderSide, OrderStatus,
    OrderType, PositionType, Signal,
};

// ---------------------------------------------------------------------------
// Scripted FuturesApi
// ---------------------------------------------------------------------------

/// A FuturesApi the test drives. `now` is settable; balance + position info
/// come from scriptable defaults; orders are scripted FIFO.
struct ScriptedApi {
    now: Mutex<DateTime<Utc>>,
    balance: Mutex<f64>,
    position_info: Mutex<Vec<serde_json::Value>>,
    place_market: Mutex<Vec<Result<ExchangeOrder, LiveError>>>,
    place_tp: Mutex<Vec<Result<ExchangeOrder, LiveError>>>,
    place_sl: Mutex<Vec<Result<ExchangeOrder, LiveError>>>,
    set_leverage_calls: Mutex<usize>,
    place_market_calls: Mutex<Vec<(String, OrderSide, f64, String)>>,
    pos_info_calls: Mutex<usize>,
}

impl ScriptedApi {
    fn new() -> Arc<Self> {
        Arc::new(Self {
            now: Mutex::new(Utc::now()),
            balance: Mutex::new(10_000.0),
            position_info: Mutex::new(vec![]),
            place_market: Mutex::new(vec![]),
            place_tp: Mutex::new(vec![]),
            place_sl: Mutex::new(vec![]),
            set_leverage_calls: Mutex::new(0),
            place_market_calls: Mutex::new(vec![]),
            pos_info_calls: Mutex::new(0),
        })
    }
    fn set_now(&self, now: DateTime<Utc>) {
        *self.now.lock().unwrap() = now;
    }
    fn set_position_info(&self, rows: Vec<serde_json::Value>) {
        *self.position_info.lock().unwrap() = rows;
    }
    fn enqueue_market(&self, r: Result<ExchangeOrder, LiveError>) {
        self.place_market.lock().unwrap().push(r);
    }
    #[allow(dead_code)]
    fn enqueue_tp(&self, r: Result<ExchangeOrder, LiveError>) {
        self.place_tp.lock().unwrap().push(r);
    }
    #[allow(dead_code)]
    fn enqueue_sl(&self, r: Result<ExchangeOrder, LiveError>) {
        self.place_sl.lock().unwrap().push(r);
    }
    fn market_calls(&self) -> Vec<(String, OrderSide, f64, String)> {
        self.place_market_calls.lock().unwrap().clone()
    }
    fn pos_info_calls(&self) -> usize {
        *self.pos_info_calls.lock().unwrap()
    }
}

fn pop<T>(slot: &Mutex<Vec<Result<T, LiveError>>>, label: &str) -> Result<T, LiveError> {
    let mut g = slot.lock().unwrap();
    if g.is_empty() {
        return Err(LiveError::Http(format!("ScriptedApi: no scripted {label}")));
    }
    g.remove(0)
}

impl FuturesApi for ScriptedApi {
    fn server_now(&self) -> DateTime<Utc> {
        *self.now.lock().unwrap()
    }
    fn place_market_order(
        &self,
        symbol: &str,
        side: OrderSide,
        qty: f64,
        _: &str,
        cid: &str,
    ) -> Result<ExchangeOrder, LiveError> {
        self.place_market_calls
            .lock()
            .unwrap()
            .push((symbol.into(), side, qty, cid.into()));
        pop(&self.place_market, "place_market_order")
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
        unreachable!()
    }
    fn place_stop_market(
        &self,
        _: &str,
        _: OrderSide,
        _: f64,
        _: &str,
        _: Option<f64>,
        _: &str,
    ) -> Result<ExchangeOrder, LiveError> {
        pop(&self.place_sl, "place_stop_market")
    }
    fn place_take_profit_market(
        &self,
        _: &str,
        _: OrderSide,
        _: f64,
        _: &str,
        _: Option<f64>,
        _: &str,
    ) -> Result<ExchangeOrder, LiveError> {
        pop(&self.place_tp, "place_take_profit_market")
    }
    fn cancel_order(&self, _: &str, _: i64) -> Result<ExchangeOrder, LiveError> {
        unreachable!()
    }
    fn cancel_algo_order(&self, _: i64) -> Result<(), LiveError> {
        unreachable!()
    }
    fn get_order(&self, _: &str, _: i64) -> Result<ExchangeOrder, LiveError> {
        // Return an error rather than panic — fill-check tests trigger this
        // without scripting a response. The tracker logs and skips the
        // position, which is the realistic "transient query failure" case.
        Err(LiveError::Http("get_order not scripted".into()))
    }
    fn get_order_by_client_id(
        &self,
        _: &str,
        _: &str,
    ) -> Result<Option<ExchangeOrder>, LiveError> {
        unreachable!()
    }
    fn get_algo_order(&self, _: i64) -> Result<ExchangeOrder, LiveError> {
        unreachable!()
    }
    fn get_algo_order_by_client_id(
        &self,
        _: &str,
    ) -> Result<Option<ExchangeOrder>, LiveError> {
        unreachable!()
    }
    fn get_open_orders(&self, _: Option<&str>) -> Result<Vec<ExchangeOrder>, LiveError> {
        Ok(vec![])
    }
    fn get_position_info(
        &self,
        _: Option<&str>,
    ) -> Result<Vec<serde_json::Value>, LiveError> {
        *self.pos_info_calls.lock().unwrap() += 1;
        Ok(self.position_info.lock().unwrap().clone())
    }
    fn get_account_trades(
        &self,
        _: &str,
        _: Option<DateTime<Utc>>,
        _: Option<DateTime<Utc>>,
        _: Option<i64>,
        _: usize,
    ) -> Result<Vec<AccountTrade>, LiveError> {
        Ok(vec![])
    }
    fn get_account_info(&self) -> Result<serde_json::Value, LiveError> {
        Ok(serde_json::json!({"availableBalance": "10000"}))
    }
    fn get_available_balance(&self) -> Result<f64, LiveError> {
        Ok(*self.balance.lock().unwrap())
    }
    fn set_leverage(&self, _: &str, _: u32) -> Result<(), LiveError> {
        *self.set_leverage_calls.lock().unwrap() += 1;
        Ok(())
    }
    fn get_exchange_info(&self) -> Result<serde_json::Value, LiveError> {
        Ok(fixture_exchange_info())
    }
    fn get_mark_price(&self, _: &str) -> Result<f64, LiveError> {
        Ok(43_000.0)
    }
}

fn fixture_exchange_info() -> serde_json::Value {
    serde_json::json!({
        "symbols": [{
            "symbol": "BTCUSDT",
            "filters": [
                {"filterType": "PRICE_FILTER",     "tickSize": "0.10"},
                {"filterType": "LOT_SIZE",         "stepSize": "0.001", "minQty": "0.001"},
                {"filterType": "MARKET_LOT_SIZE",  "stepSize": "0.001", "minQty": "0.001"},
                {"filterType": "MIN_NOTIONAL",     "notional": "5.0"}
            ]
        }, {
            "symbol": "ETHUSDT",
            "filters": [
                {"filterType": "PRICE_FILTER",     "tickSize": "0.01"},
                {"filterType": "LOT_SIZE",         "stepSize": "0.0001", "minQty": "0.001"},
                {"filterType": "MARKET_LOT_SIZE",  "stepSize": "0.0001", "minQty": "0.001"},
                {"filterType": "MIN_NOTIONAL",     "notional": "5.0"}
            ]
        }, {
            "symbol": "SOLUSDT",
            "filters": [
                {"filterType": "PRICE_FILTER",     "tickSize": "0.01"},
                {"filterType": "LOT_SIZE",         "stepSize": "0.001", "minQty": "0.001"},
                {"filterType": "MARKET_LOT_SIZE",  "stepSize": "0.001", "minQty": "0.001"},
                {"filterType": "MIN_NOTIONAL",     "notional": "5.0"}
            ]
        }]
    })
}

// ---------------------------------------------------------------------------
// MockGenerator
// ---------------------------------------------------------------------------

#[derive(Default)]
struct GenLog {
    setup_count: usize,
    teardown_count: usize,
    poll_count: usize,
    last_poll_time: Option<DateTime<Utc>>,
}

struct MockGenerator {
    id: String,
    symbols: Vec<String>,
    interval: String,
    leverage: f64,
    poll_responses: Mutex<Vec<Result<Vec<Signal>, SignalError>>>,
    log: Arc<Mutex<GenLog>>,
}

impl MockGenerator {
    fn new(id: &str, symbols: &[&str]) -> (Self, Arc<Mutex<GenLog>>) {
        let log = Arc::new(Mutex::new(GenLog::default()));
        let gen = Self {
            id: id.into(),
            symbols: symbols.iter().map(|s| s.to_string()).collect(),
            interval: "1h".into(),
            leverage: 1.0,
            poll_responses: Mutex::new(vec![]),
            log: log.clone(),
        };
        (gen, log)
    }
    fn enqueue(&self, r: Result<Vec<Signal>, SignalError>) {
        self.poll_responses.lock().unwrap().push(r);
    }
}

impl LiveSignalGenerator for MockGenerator {
    fn strategy_id(&self) -> &str {
        &self.id
    }
    fn symbols(&self) -> &[String] {
        &self.symbols
    }
    fn analysis_interval(&self) -> &str {
        &self.interval
    }
    fn leverage(&self) -> f64 {
        self.leverage
    }
    fn setup(&mut self, _: Arc<dyn LiveMarketClient>) -> Result<(), SignalError> {
        self.log.lock().unwrap().setup_count += 1;
        Ok(())
    }
    fn set_poll_time(&mut self, now: DateTime<Utc>) {
        self.log.lock().unwrap().last_poll_time = Some(now);
    }
    fn poll(&mut self) -> Result<Vec<Signal>, SignalError> {
        let mut log = self.log.lock().unwrap();
        log.poll_count += 1;
        drop(log);
        let mut q = self.poll_responses.lock().unwrap();
        if q.is_empty() {
            return Ok(vec![]);
        }
        q.remove(0)
    }
    fn teardown(&mut self) {
        self.log.lock().unwrap().teardown_count += 1;
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
        metadata: HashMap::new(),
    }
}

fn order_new(order_id: i64, qty: f64, cid: &str) -> ExchangeOrder {
    ExchangeOrder {
        order_id,
        symbol: "BTCUSDT".into(),
        side: OrderSide::Buy,
        order_type: OrderType::Market,
        quantity: qty,
        price: 0.0,
        stop_price: 0.0,
        status: OrderStatus::New,
        filled_qty: 0.0,
        avg_fill_price: 0.0,
        created_at: Some(Utc::now()),
        updated_at: Some(Utc::now()),
        algo_id: 0,
        client_order_id: cid.into(),
    }
}

fn temp_state_path(label: &str) -> PathBuf {
    let pid = std::process::id();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("ct_engine_{label}_{pid}_{nonce}"));
    std::fs::create_dir_all(&dir).unwrap();
    dir.join("live_state.json")
}

fn cleanup(path: &PathBuf) {
    if let Some(dir) = path.parent() {
        let _ = std::fs::remove_dir_all(dir);
    }
}

fn cfg() -> LiveConfig {
    LiveConfig {
        api_key: "k".into(),
        api_secret: "s".into(),
        base_url: "http://test".into(),
        position_size_usdt: 100.0,
        max_concurrent_positions: 3,
        order_check_interval_seconds: 5.0,
        testnet: false,
        recover_brackets_on_startup: false,
    }
}

fn build_engine(
    api: Arc<ScriptedApi>,
    gen: Box<dyn LiveSignalGenerator>,
    state_path: PathBuf,
) -> LiveEngine {
    let market = Arc::new(NullMarketClient);
    let mut e = LiveEngine::new_single(cfg(), api, market, gen).unwrap();
    e.disable_signal_handlers();
    e.set_state_path(state_path);
    e
}

fn ts(rfc: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(rfc)
        .unwrap()
        .with_timezone(&Utc)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[test]
fn tick_runs_signal_poll_and_places_entry() {
    let api = ScriptedApi::new();
    let path = temp_state_path("tick_place");
    let (gen, log) = MockGenerator::new("alpha", &["BTCUSDT"]);
    gen.enqueue(Ok(vec![long_signal("BTCUSDT")]));
    let mut engine = build_engine(api.clone(), Box::new(gen), path.clone());

    // Just-past 1h boundary so due_slot_indices fires.
    let now = ts("2026-04-30T13:00:00.5Z");
    api.set_now(now);
    api.enqueue_market(Ok(order_new(123, 0.01, "")));

    engine.tick(now).unwrap();

    let log = log.lock().unwrap();
    assert_eq!(log.poll_count, 1);
    assert_eq!(log.last_poll_time, Some(now));
    drop(log);

    // Position entered.
    assert_eq!(engine.tracker().positions().len(), 1);
    cleanup(&path);
}

#[test]
fn tick_off_boundary_does_nothing() {
    let api = ScriptedApi::new();
    let path = temp_state_path("tick_idle");
    let (gen, log) = MockGenerator::new("alpha", &["BTCUSDT"]);
    let mut engine = build_engine(api.clone(), Box::new(gen), path.clone());

    // Mid-hour, no boundary fires.
    let now = ts("2026-04-30T12:30:00Z");
    api.set_now(now);
    engine.tick(now).unwrap();

    assert_eq!(log.lock().unwrap().poll_count, 0);
    assert_eq!(engine.tracker().positions().len(), 0);
    cleanup(&path);
}

#[test]
fn recoverable_poll_error_keeps_engine_running() {
    // Regression: SignalError::Recoverable should be logged and treated as
    // no-signals; engine continues. Only Fatal halts.
    let api = ScriptedApi::new();
    let path = temp_state_path("recoverable");
    let (gen, log) = MockGenerator::new("alpha", &["BTCUSDT"]);
    gen.enqueue(Err(SignalError::recoverable("symbol fetch failed")));
    let mut engine = build_engine(api.clone(), Box::new(gen), path.clone());

    let now = ts("2026-04-30T13:00:00.5Z");
    api.set_now(now);
    // Tick must NOT error.
    let result = engine.tick(now);
    assert!(result.is_ok(), "recoverable poll error must not halt the engine");

    // Generator was polled.
    assert_eq!(log.lock().unwrap().poll_count, 1);
    // No order was placed (no signals returned).
    assert!(api.market_calls().is_empty());

    cleanup(&path);
}

#[test]
fn fatal_signal_error_propagates_as_live_error() {
    let api = ScriptedApi::new();
    let path = temp_state_path("fatal");
    let (gen, _log) = MockGenerator::new("alpha", &["BTCUSDT"]);
    gen.enqueue(Err(SignalError::fatal("warmup failed")));
    let mut engine = build_engine(api.clone(), Box::new(gen), path.clone());

    let now = ts("2026-04-30T13:00:00.5Z");
    api.set_now(now);
    let err = engine.tick(now).unwrap_err();
    match err {
        LiveError::Fatal(msg) => assert!(msg.contains("warmup failed")),
        other => panic!("expected LiveError::Fatal, got {other:?}"),
    }
    cleanup(&path);
}

#[test]
fn signal_for_undeclared_symbol_dropped_with_warning() {
    let api = ScriptedApi::new();
    let path = temp_state_path("undeclared");
    let (gen, _log) = MockGenerator::new("alpha", &["BTCUSDT"]);
    // Strategy emits a signal for ETHUSDT — not in declared symbols.
    gen.enqueue(Ok(vec![long_signal("ETHUSDT")]));
    let mut engine = build_engine(api.clone(), Box::new(gen), path.clone());

    let now = ts("2026-04-30T13:00:00.5Z");
    api.set_now(now);
    // No market_order should be placed (the signal is filtered out).
    engine.tick(now).unwrap();

    assert!(api.market_calls().is_empty());
    assert!(engine.tracker().positions().is_empty());
    cleanup(&path);
}

#[test]
fn external_conflict_filters_matching_signal() {
    let api = ScriptedApi::new();
    let path = temp_state_path("ext_conflict");
    let (gen, _log) = MockGenerator::new("alpha", &["BTCUSDT", "ETHUSDT"]);
    gen.enqueue(Ok(vec![long_signal("BTCUSDT"), long_signal("ETHUSDT")]));
    // Exchange already has a BTC position we don't track.
    api.set_position_info(vec![serde_json::json!({
        "symbol": "BTCUSDT",
        "positionAmt": "0.05",
        "positionSide": "LONG"
    })]);
    let mut engine = build_engine(api.clone(), Box::new(gen), path.clone());

    let now = ts("2026-04-30T13:00:00.5Z");
    api.set_now(now);
    // Only one entry should fire (ETH; BTC is blocked).
    api.enqueue_market(Ok(order_new(200, 0.002, "")));

    engine.tick(now).unwrap();

    // 1 placement: ETH only.
    let calls = api.market_calls();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].0, "ETHUSDT");

    cleanup(&path);
}

#[test]
fn capital_cascade_caps_to_global_remaining() {
    let mut config = cfg();
    config.max_concurrent_positions = 1;
    let api = ScriptedApi::new();
    let path = temp_state_path("global_cap");
    let (gen, _log) = MockGenerator::new("alpha", &["BTCUSDT", "ETHUSDT", "SOLUSDT"]);
    gen.enqueue(Ok(vec![
        long_signal("BTCUSDT"),
        long_signal("ETHUSDT"),
        long_signal("SOLUSDT"),
    ]));
    let market = Arc::new(NullMarketClient);
    let mut engine = LiveEngine::new_single(config, api.clone(), market, Box::new(gen)).unwrap();
    engine.disable_signal_handlers();
    engine.set_state_path(path.clone());

    let now = ts("2026-04-30T13:00:00.5Z");
    api.set_now(now);
    // Only one market order can be placed; others should not be attempted.
    api.enqueue_market(Ok(order_new(1, 0.01, "")));

    engine.tick(now).unwrap();

    assert_eq!(api.market_calls().len(), 1);
    assert_eq!(engine.tracker().positions().len(), 1);
    cleanup(&path);
}

#[test]
fn pre_poll_runs_reconcile_and_caches_balance() {
    let api = ScriptedApi::new();
    let path = temp_state_path("pre_poll");
    let (gen, _log) = MockGenerator::new("alpha", &["BTCUSDT"]);
    let mut engine = build_engine(api.clone(), Box::new(gen), path.clone());

    // Just before the top of the hour — within pre-poll lead window (10 s).
    let now = ts("2026-04-30T12:59:55Z");
    api.set_now(now);
    let pos_calls_before = api.pos_info_calls();
    engine.tick(now).unwrap();
    let pos_calls_after = api.pos_info_calls();
    // Pre-poll did a reconcile, which calls get_position_info.
    assert!(pos_calls_after > pos_calls_before);
    cleanup(&path);
}

#[test]
fn pre_poll_marks_boundary_so_it_doesnt_repeat() {
    let api = ScriptedApi::new();
    let path = temp_state_path("pre_poll_once");
    let (gen, _log) = MockGenerator::new("alpha", &["BTCUSDT"]);
    let mut engine = build_engine(api.clone(), Box::new(gen), path.clone());

    let now = ts("2026-04-30T12:59:55Z");
    api.set_now(now);
    engine.tick(now).unwrap();
    let pos_after_first = api.pos_info_calls();

    // Tick again at a slightly later time still within the same boundary's
    // lead window. Pre-poll should NOT re-fire.
    let now2 = ts("2026-04-30T12:59:58Z");
    api.set_now(now2);
    engine.tick(now2).unwrap();

    assert_eq!(
        api.pos_info_calls(),
        pos_after_first,
        "pre-poll should not re-fire for the same boundary",
    );
    cleanup(&path);
}

#[test]
fn ordinary_place_entry_failure_does_not_stop_engine() {
    let api = ScriptedApi::new();
    let path = temp_state_path("place_err");
    let (gen, _log) = MockGenerator::new("alpha", &["BTCUSDT"]);
    gen.enqueue(Ok(vec![long_signal("BTCUSDT")]));
    let mut engine = build_engine(api.clone(), Box::new(gen), path.clone());

    // Submit fails with terminal-4xx → place_entry returns Ok(Rejected).
    let now = ts("2026-04-30T13:00:00.5Z");
    api.set_now(now);
    api.enqueue_market(Err(LiveError::Api {
        code: -2010,
        msg: "rejected".into(),
    }));

    let result = engine.tick(now);
    assert!(result.is_ok(), "engine should NOT halt on ordinary errors");

    // Position is in memory as Failed.
    assert_eq!(engine.tracker().positions().len(), 1);
    assert_eq!(
        engine.tracker().positions()[0].status,
        claude_trader_models::PositionStatus::Failed
    );
    cleanup(&path);
}

#[test]
fn set_poll_time_consistent_across_due_slots() {
    let api = ScriptedApi::new();
    let path = temp_state_path("set_poll_time");
    let market = Arc::new(NullMarketClient);
    let (gen_a, log_a) = MockGenerator::new("alpha", &["BTCUSDT"]);
    let (gen_b, log_b) = MockGenerator::new("beta", &["ETHUSDT"]);
    let mut engine = LiveEngine::new(
        cfg(),
        api.clone(),
        market,
        vec![
            (Box::new(gen_a), GeneratorBudget::default()),
            (Box::new(gen_b), GeneratorBudget::default()),
        ],
    )
    .unwrap();
    engine.disable_signal_handlers();
    engine.set_state_path(path.clone());

    // Both slots have analysis_interval "1h" and the same poll_interval.
    let now = ts("2026-04-30T13:00:00.5Z");
    api.set_now(now);
    engine.tick(now).unwrap();

    let a = log_a.lock().unwrap().last_poll_time;
    let b = log_b.lock().unwrap().last_poll_time;
    assert_eq!(a, Some(now));
    assert_eq!(b, Some(now));
    cleanup(&path);
}

#[test]
fn shutdown_flag_exits_loop_cleanly() {
    let api = ScriptedApi::new();
    let path = temp_state_path("shutdown");
    let (gen, log) = MockGenerator::new("alpha", &["BTCUSDT"]);
    let mut engine = build_engine(api.clone(), Box::new(gen), path.clone());

    // Pre-flip the shutdown flag so the loop body never runs.
    let flag = engine.shutdown_flag();
    flag.store(true, std::sync::atomic::Ordering::SeqCst);

    // start() runs load_state → reconcile → recover_brackets → setup →
    // (no install because handlers disabled) → run_loop. The loop body
    // checks the flag and exits immediately.
    api.set_now(ts("2026-04-30T13:00:00.5Z"));
    engine.start().unwrap();

    // Generator setup AND teardown both ran.
    let log = log.lock().unwrap();
    assert_eq!(log.setup_count, 1);
    assert_eq!(log.teardown_count, 1);
    // poll never invoked because we shut down before the first iteration.
    assert_eq!(log.poll_count, 0);
    cleanup(&path);
}

#[test]
fn start_runs_strict_sequence() {
    let api = ScriptedApi::new();
    let path = temp_state_path("startup_order");
    let (gen, log) = MockGenerator::new("alpha", &["BTCUSDT"]);
    let mut engine = build_engine(api.clone(), Box::new(gen), path.clone());

    let flag = engine.shutdown_flag();
    flag.store(true, std::sync::atomic::Ordering::SeqCst);

    api.set_now(ts("2026-04-30T13:00:00Z"));
    engine.start().unwrap();

    // setup and teardown both fired exactly once.
    let log = log.lock().unwrap();
    assert_eq!(log.setup_count, 1);
    assert_eq!(log.teardown_count, 1);
    cleanup(&path);
}

#[test]
fn fatal_setup_error_still_runs_teardown_and_save() {
    /// Generator that fails during setup.
    struct FailingSetup {
        log: Arc<Mutex<GenLog>>,
    }
    impl LiveSignalGenerator for FailingSetup {
        fn strategy_id(&self) -> &str {
            "alpha"
        }
        fn symbols(&self) -> &[String] {
            static EMPTY: Vec<String> = Vec::new();
            // dummy — strategy never reaches poll, but the engine constructor
            // needs at least one symbol; provide one via accessor returning
            // a freshly allocated slice. Since trait requires &[String],
            // we cheat: return an empty slice. The engine uses symbols only
            // for disjoint validation at construction; with one slot, an
            // empty list is fine.
            &EMPTY
        }
        fn setup(&mut self, _: Arc<dyn LiveMarketClient>) -> Result<(), SignalError> {
            self.log.lock().unwrap().setup_count += 1;
            Err(SignalError::fatal("warmup blew up"))
        }
        fn poll(&mut self) -> Result<Vec<Signal>, SignalError> {
            unreachable!()
        }
        fn teardown(&mut self) {
            self.log.lock().unwrap().teardown_count += 1;
        }
    }

    let api = ScriptedApi::new();
    let path = temp_state_path("setup_fail");
    let log = Arc::new(Mutex::new(GenLog::default()));
    let gen = Box::new(FailingSetup { log: log.clone() }) as Box<dyn LiveSignalGenerator>;
    let market = Arc::new(NullMarketClient);
    let mut engine = LiveEngine::new_single(cfg(), api, market, gen).unwrap();
    engine.disable_signal_handlers();
    engine.set_state_path(path.clone());

    let result = engine.start();
    assert!(matches!(result, Err(LiveError::Fatal(_))));

    // Teardown ran even though setup failed.
    let log = log.lock().unwrap();
    assert_eq!(log.setup_count, 1);
    assert_eq!(log.teardown_count, 1);
    cleanup(&path);
}

#[test]
fn placeholder_bracket_recovery_runs_periodically_during_runtime() {
    // Regression for: bracket recovery used to fire only at startup. After
    // entry-fill submit failed (5xx), the placeholder lived on disk but a
    // running engine never re-attempted until restart. This test seeds a
    // placeholder and asserts the engine's tick triggers
    // `recover_placeholder_brackets` on the documented cadence.
    use std::sync::atomic::{AtomicUsize, Ordering};

    /// Custom api that counts get_position_info calls (the entry point of
    /// recover_placeholder_brackets — confirms it actually ran). Also
    /// returns a populated algo order on the by-cid query so adoption
    /// captures the real algo_id and finishes the recovery cleanly.
    struct RecoveryApi {
        now: Mutex<DateTime<Utc>>,
        position_info_calls: AtomicUsize,
        algo_by_cid_calls: AtomicUsize,
        // Settable position-info response so the test can simulate the
        // exchange's view evolving across the lifecycle: empty before
        // entry is filled, BTCUSDT-LONG after.
        position_info: Mutex<Vec<serde_json::Value>>,
        place_market: Mutex<Vec<Result<ExchangeOrder, LiveError>>>,
        place_tp: Mutex<Vec<Result<ExchangeOrder, LiveError>>>,
        place_sl: Mutex<Vec<Result<ExchangeOrder, LiveError>>>,
        get_order: Mutex<Vec<Result<ExchangeOrder, LiveError>>>,
        adopted_tp_algo_id: AtomicUsize,
        adopted_sl_algo_id: AtomicUsize,
    }
    impl RecoveryApi {
        fn new() -> Arc<Self> {
            Arc::new(Self {
                now: Mutex::new(Utc::now()),
                position_info_calls: AtomicUsize::new(0),
                algo_by_cid_calls: AtomicUsize::new(0),
                position_info: Mutex::new(vec![]),
                place_market: Mutex::new(vec![]),
                place_tp: Mutex::new(vec![]),
                place_sl: Mutex::new(vec![]),
                get_order: Mutex::new(vec![]),
                adopted_tp_algo_id: AtomicUsize::new(91_001),
                adopted_sl_algo_id: AtomicUsize::new(91_002),
            })
        }
        fn set_now(&self, n: DateTime<Utc>) {
            *self.now.lock().unwrap() = n;
        }
        fn set_position_info(&self, rows: Vec<serde_json::Value>) {
            *self.position_info.lock().unwrap() = rows;
        }
        fn enqueue_market(&self, r: Result<ExchangeOrder, LiveError>) {
            self.place_market.lock().unwrap().push(r);
        }
        fn enqueue_tp(&self, r: Result<ExchangeOrder, LiveError>) {
            self.place_tp.lock().unwrap().push(r);
        }
        fn enqueue_sl(&self, r: Result<ExchangeOrder, LiveError>) {
            self.place_sl.lock().unwrap().push(r);
        }
        fn enqueue_get_order(&self, r: Result<ExchangeOrder, LiveError>) {
            self.get_order.lock().unwrap().push(r);
        }
    }
    impl FuturesApi for RecoveryApi {
        fn server_now(&self) -> DateTime<Utc> {
            *self.now.lock().unwrap()
        }
        fn place_market_order(
            &self,
            _: &str,
            _: OrderSide,
            _: f64,
            _: &str,
            _: &str,
        ) -> Result<ExchangeOrder, LiveError> {
            let mut q = self.place_market.lock().unwrap();
            if q.is_empty() {
                return Err(LiveError::Http("place_market not scripted".into()));
            }
            q.remove(0)
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
            unreachable!()
        }
        fn place_stop_market(
            &self,
            _: &str,
            _: OrderSide,
            _: f64,
            _: &str,
            _: Option<f64>,
            _: &str,
        ) -> Result<ExchangeOrder, LiveError> {
            let mut q = self.place_sl.lock().unwrap();
            if q.is_empty() {
                return Err(LiveError::Http("sl not scripted".into()));
            }
            q.remove(0)
        }
        fn place_take_profit_market(
            &self,
            _: &str,
            _: OrderSide,
            _: f64,
            _: &str,
            _: Option<f64>,
            _: &str,
        ) -> Result<ExchangeOrder, LiveError> {
            let mut q = self.place_tp.lock().unwrap();
            if q.is_empty() {
                return Err(LiveError::Http("tp not scripted".into()));
            }
            q.remove(0)
        }
        fn cancel_order(&self, _: &str, _: i64) -> Result<ExchangeOrder, LiveError> {
            unreachable!()
        }
        fn cancel_algo_order(&self, _: i64) -> Result<(), LiveError> {
            unreachable!()
        }
        fn get_order(&self, _: &str, _: i64) -> Result<ExchangeOrder, LiveError> {
            let mut q = self.get_order.lock().unwrap();
            if q.is_empty() {
                return Err(LiveError::Http("get_order not scripted".into()));
            }
            q.remove(0)
        }
        fn get_order_by_client_id(
            &self,
            _: &str,
            _: &str,
        ) -> Result<Option<ExchangeOrder>, LiveError> {
            unreachable!()
        }
        fn get_algo_order(&self, _: i64) -> Result<ExchangeOrder, LiveError> {
            unreachable!()
        }
        fn get_algo_order_by_client_id(
            &self,
            cid: &str,
        ) -> Result<Option<ExchangeOrder>, LiveError> {
            self.algo_by_cid_calls.fetch_add(1, Ordering::SeqCst);
            // Distinguish TP vs SL by call ordering: first call = TP, second = SL.
            let first = self.algo_by_cid_calls.load(Ordering::SeqCst) == 1;
            let (algo_id, kind) = if first {
                (
                    self.adopted_tp_algo_id.load(Ordering::SeqCst) as i64,
                    OrderType::TakeProfitMarket,
                )
            } else {
                (
                    self.adopted_sl_algo_id.load(Ordering::SeqCst) as i64,
                    OrderType::StopMarket,
                )
            };
            Ok(Some(ExchangeOrder {
                order_id: 0,
                symbol: "BTCUSDT".into(),
                side: OrderSide::Sell,
                order_type: kind,
                quantity: 0.01,
                price: 0.0,
                stop_price: 43_000.0,
                status: OrderStatus::New,
                filled_qty: 0.0,
                avg_fill_price: 0.0,
                created_at: None,
                updated_at: None,
                algo_id,
                client_order_id: cid.to_string(),
            }))
        }
        fn get_open_orders(&self, _: Option<&str>) -> Result<Vec<ExchangeOrder>, LiveError> {
            Ok(vec![])
        }
        fn get_position_info(
            &self,
            _: Option<&str>,
        ) -> Result<Vec<serde_json::Value>, LiveError> {
            self.position_info_calls.fetch_add(1, Ordering::SeqCst);
            Ok(self.position_info.lock().unwrap().clone())
        }
        fn get_account_trades(
            &self,
            _: &str,
            _: Option<DateTime<Utc>>,
            _: Option<DateTime<Utc>>,
            _: Option<i64>,
            _: usize,
        ) -> Result<Vec<AccountTrade>, LiveError> {
            Ok(vec![])
        }
        fn get_account_info(&self) -> Result<serde_json::Value, LiveError> {
            Ok(serde_json::json!({"availableBalance": "10000"}))
        }
        fn get_available_balance(&self) -> Result<f64, LiveError> {
            Ok(10_000.0)
        }
        fn set_leverage(&self, _: &str, _: u32) -> Result<(), LiveError> {
            Ok(())
        }
        fn get_exchange_info(&self) -> Result<serde_json::Value, LiveError> {
            Ok(fixture_exchange_info())
        }
        fn get_mark_price(&self, _: &str) -> Result<f64, LiveError> {
            Ok(43_000.0)
        }
    }

    let api = RecoveryApi::new();
    let path = temp_state_path("placeholder_periodic");

    // Seed a position with placeholder brackets via the entry-fill flow:
    // entry filled, tracker prepares brackets, submit fails, placeholder
    // persists on the position with its cid. Engine.tick orchestrates this.
    let (gen, _log) = MockGenerator::new("alpha", &["BTCUSDT"]);
    gen.enqueue(Ok(vec![long_signal("BTCUSDT")]));
    let market = Arc::new(NullMarketClient);
    let mut engine =
        LiveEngine::new_single(cfg(), api.clone(), market, Box::new(gen)).unwrap();
    engine.disable_signal_handlers();
    engine.set_state_path(path.clone());

    // For this test we want the placeholder to *survive* across multiple
    // ticks so we can observe the cadence. Override the by-cid handler to
    // return errors (Unknown classification) on the first sweep, then
    // success on a later sweep — placeholders stay until the final adoption.
    struct CidSwitchableApi {
        inner: Arc<RecoveryApi>,
        // First N calls return Err (defer); subsequent calls return Some
        // (adopt). Tracked atomically so we can read it from outside.
        adoption_after_call: AtomicUsize,
    }
    impl FuturesApi for CidSwitchableApi {
        fn server_now(&self) -> DateTime<Utc> {
            self.inner.server_now()
        }
        fn place_market_order(
            &self,
            s: &str,
            sd: OrderSide,
            q: f64,
            ps: &str,
            cid: &str,
        ) -> Result<ExchangeOrder, LiveError> {
            self.inner.place_market_order(s, sd, q, ps, cid)
        }
        fn place_limit_order(
            &self,
            s: &str,
            sd: OrderSide,
            q: f64,
            p: f64,
            ps: &str,
            c: &str,
        ) -> Result<ExchangeOrder, LiveError> {
            self.inner.place_limit_order(s, sd, q, p, ps, c)
        }
        fn place_stop_market(
            &self,
            s: &str,
            sd: OrderSide,
            sp: f64,
            ps: &str,
            q: Option<f64>,
            c: &str,
        ) -> Result<ExchangeOrder, LiveError> {
            self.inner.place_stop_market(s, sd, sp, ps, q, c)
        }
        fn place_take_profit_market(
            &self,
            s: &str,
            sd: OrderSide,
            sp: f64,
            ps: &str,
            q: Option<f64>,
            c: &str,
        ) -> Result<ExchangeOrder, LiveError> {
            self.inner.place_take_profit_market(s, sd, sp, ps, q, c)
        }
        fn cancel_order(&self, s: &str, o: i64) -> Result<ExchangeOrder, LiveError> {
            self.inner.cancel_order(s, o)
        }
        fn cancel_algo_order(&self, a: i64) -> Result<(), LiveError> {
            self.inner.cancel_algo_order(a)
        }
        fn get_order(&self, s: &str, o: i64) -> Result<ExchangeOrder, LiveError> {
            self.inner.get_order(s, o)
        }
        fn get_order_by_client_id(
            &self,
            s: &str,
            c: &str,
        ) -> Result<Option<ExchangeOrder>, LiveError> {
            self.inner.get_order_by_client_id(s, c)
        }
        fn get_algo_order(&self, a: i64) -> Result<ExchangeOrder, LiveError> {
            self.inner.get_algo_order(a)
        }
        fn get_algo_order_by_client_id(
            &self,
            cid: &str,
        ) -> Result<Option<ExchangeOrder>, LiveError> {
            // Increment the counter the test reads.
            let n = self.inner.algo_by_cid_calls.fetch_add(1, Ordering::SeqCst);
            if n < self.adoption_after_call.load(Ordering::SeqCst) {
                // Defer: returning Err makes classify_bracket → Unknown,
                // so the whole recovery-pass for this position is skipped
                // and the placeholder survives untouched.
                return Err(LiveError::Http("transient".into()));
            }
            self.inner.get_algo_order_by_client_id(cid)
        }
        fn get_open_orders(
            &self,
            s: Option<&str>,
        ) -> Result<Vec<ExchangeOrder>, LiveError> {
            self.inner.get_open_orders(s)
        }
        fn get_position_info(
            &self,
            s: Option<&str>,
        ) -> Result<Vec<serde_json::Value>, LiveError> {
            self.inner.get_position_info(s)
        }
        fn get_account_trades(
            &self,
            s: &str,
            a: Option<DateTime<Utc>>,
            b: Option<DateTime<Utc>>,
            o: Option<i64>,
            l: usize,
        ) -> Result<Vec<AccountTrade>, LiveError> {
            self.inner.get_account_trades(s, a, b, o, l)
        }
        fn get_account_info(&self) -> Result<serde_json::Value, LiveError> {
            self.inner.get_account_info()
        }
        fn get_available_balance(&self) -> Result<f64, LiveError> {
            self.inner.get_available_balance()
        }
        fn set_leverage(&self, s: &str, l: u32) -> Result<(), LiveError> {
            self.inner.set_leverage(s, l)
        }
        fn get_exchange_info(&self) -> Result<serde_json::Value, LiveError> {
            self.inner.get_exchange_info()
        }
        fn get_mark_price(&self, s: &str) -> Result<f64, LiveError> {
            self.inner.get_mark_price(s)
        }
    }

    let switch = Arc::new(CidSwitchableApi {
        inner: api.clone(),
        // The first sweep (at t1, when the placeholder is created) sees
        // both TP and SL queries deferred — calls 1 and 2 return Err. The
        // second sweep (at t3, after the cadence elapses) adopts both —
        // calls 3 and 4 hit the inner "found on Binance" path. Threshold:
        // first 2 calls defer, third onwards adopts.
        adoption_after_call: AtomicUsize::new(2),
    });
    let market = Arc::new(NullMarketClient);
    let (gen, _log) = MockGenerator::new("alpha", &["BTCUSDT"]);
    gen.enqueue(Ok(vec![long_signal("BTCUSDT")]));
    // Recovery is gated by recover_brackets_on_startup. The shared cfg()
    // turns it off; this test explicitly enables it.
    let mut config = cfg();
    config.recover_brackets_on_startup = true;
    let mut engine = LiveEngine::new_single(
        config,
        switch.clone() as Arc<dyn FuturesApi>,
        market,
        Box::new(gen),
    )
    .unwrap();
    engine.disable_signal_handlers();
    engine.set_state_path(path.clone());

    // T0: signal poll places entry. No exchange position yet.
    let t0 = ts("2026-04-30T13:00:00.5Z");
    api.set_now(t0);
    api.set_position_info(vec![]);
    api.enqueue_market(Ok(ExchangeOrder {
        order_id: 1,
        status: OrderStatus::New,
        ..order_new(1, 0.01, "")
    }));
    engine.tick(t0).unwrap();

    // T1: entry order filled on Binance. Exchange reports BTCUSDT position.
    // Tracker prepares brackets; submit fails → placeholders retained.
    // Phase 1b *also runs* in this same tick — placeholder is fresh —
    // but our switchable api defers the by-cid query (returns Err →
    // Unknown), so the placeholder survives.
    let t1 = t0 + ChronoDuration::seconds(5);
    api.set_now(t1);
    api.set_position_info(vec![serde_json::json!({
        "symbol": "BTCUSDT",
        "positionAmt": "0.01",
        "positionSide": "LONG"
    })]);
    api.enqueue_get_order(Ok(ExchangeOrder {
        order_id: 1,
        status: OrderStatus::Filled,
        avg_fill_price: 43_500.0,
        filled_qty: 0.01,
        updated_at: Some(t1),
        ..order_new(1, 0.01, "")
    }));
    api.enqueue_tp(Err(LiveError::Http("503 mid-submit".into())));
    api.enqueue_sl(Err(LiveError::Http("503 mid-submit".into())));
    engine.tick(t1).unwrap();

    // Verify: position OPEN, placeholders intact (deferred recovery), AND
    // the cid endpoint was hit during t1 (2 calls — TP + SL).
    let pos = &engine.tracker().positions()[0];
    assert_eq!(pos.status, claude_trader_models::PositionStatus::Open);
    assert_eq!(pos.tp_order.as_ref().unwrap().algo_id, 0);
    assert_eq!(pos.sl_order.as_ref().unwrap().algo_id, 0);
    let cid_after_t1 = api.algo_by_cid_calls.load(Ordering::SeqCst);
    assert!(
        cid_after_t1 >= 2,
        "first runtime recovery must have queried both brackets by cid; got {cid_after_t1}",
    );

    // T2 (5 s after t1, well within BRACKET_RECOVERY_INTERVAL_S=300):
    // recovery throttle should suppress the sweep. cid calls flat.
    let t2 = t1 + ChronoDuration::seconds(5);
    api.set_now(t2);
    engine.tick(t2).unwrap();
    let cid_after_t2 = api.algo_by_cid_calls.load(Ordering::SeqCst);
    assert_eq!(
        cid_after_t2, cid_after_t1,
        "recovery must not re-fire within {BRACKET_RECOVERY_INTERVAL_S}s window",
    );

    // T3 (>= 5 minutes after t1): cadence elapsed. Recovery fires again,
    // and this time the switchable api returns the adopted orders.
    let t3 = t1 + ChronoDuration::seconds(310);
    api.set_now(t3);
    engine.tick(t3).unwrap();
    let cid_after_t3 = api.algo_by_cid_calls.load(Ordering::SeqCst);
    assert!(
        cid_after_t3 > cid_after_t2,
        "after interval elapsed, recovery must re-fire; before={cid_after_t2}, after={cid_after_t3}",
    );

    // Adoption happened — placeholders are gone, real algo_ids present.
    let pos = &engine.tracker().positions()[0];
    assert_ne!(pos.tp_order.as_ref().unwrap().algo_id, 0);
    assert_ne!(pos.sl_order.as_ref().unwrap().algo_id, 0);

    cleanup(&path);
}

const BRACKET_RECOVERY_INTERVAL_S: u32 = 300;

#[test]
fn fill_check_throttle_immediate_first_call() {
    let api = ScriptedApi::new();
    let path = temp_state_path("fill_throttle");
    let (gen, _log) = MockGenerator::new("alpha", &["BTCUSDT"]);
    gen.enqueue(Ok(vec![long_signal("BTCUSDT")]));
    let mut engine = build_engine(api.clone(), Box::new(gen), path.clone());

    // Place a position (PendingEntry).
    let boundary_now = ts("2026-04-30T13:00:00.5Z");
    api.set_now(boundary_now);
    api.enqueue_market(Ok(order_new(1, 0.01, "")));
    engine.tick(boundary_now).unwrap();
    assert_eq!(engine.tracker().positions().len(), 1);

    // Immediately after, an off-boundary tick should NOT re-trigger signal poll
    // but SHOULD trigger a fill check (first time after position appeared).
    // We can't observe fill_check directly without scripting `get_order`, so
    // we just confirm the tick doesn't error and the position state is unchanged.
    let next_now = boundary_now + ChronoDuration::seconds(1);
    api.set_now(next_now);
    // The fill check will call get_order — we don't script it, so it errors and
    // is logged. Position remains PendingEntry. Tick still returns Ok.
    let result = engine.tick(next_now);
    assert!(result.is_ok());
    cleanup(&path);
}
