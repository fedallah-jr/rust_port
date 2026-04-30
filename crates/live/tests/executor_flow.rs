//! Integration tests for `OrderExecutor` (Phase C).
//!
//! Uses a recording `MockFuturesApi` that the executor talks to as if it were
//! the real Binance client. Each test pre-populates `ExchangeInfoCache` with
//! a JSON fixture so we don't need an HTTP layer.

use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use claude_trader_live::auth_client::FuturesApi;
use claude_trader_live::error::LiveError;
use claude_trader_live::exchange_info::ExchangeInfoCache;
use claude_trader_live::executor::OrderExecutor;
use claude_trader_models::{
    AccountTrade, ExchangeOrder, LiveConfig, LivePosition, MarketType, OrderSide, OrderStatus,
    OrderType, PositionStatus, PositionType, Signal,
};

// ---------------------------------------------------------------------------
// Mock FuturesApi
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
#[allow(dead_code)] // fields are read via Debug in failure output
enum Call {
    SetLeverage { symbol: String, leverage: u32 },
    GetMarkPrice { symbol: String },
    GetAvailableBalance,
    PlaceMarket {
        symbol: String,
        side: OrderSide,
        quantity: f64,
        position_side: String,
        client_order_id: String,
    },
    PlaceLimit {
        symbol: String,
        side: OrderSide,
        quantity: f64,
        price: f64,
        position_side: String,
        client_order_id: String,
    },
    PlaceTakeProfitMarket {
        symbol: String,
        side: OrderSide,
        stop_price: f64,
        position_side: String,
        quantity: Option<f64>,
        client_algo_id: String,
    },
    PlaceStopMarket {
        symbol: String,
        side: OrderSide,
        stop_price: f64,
        position_side: String,
        quantity: Option<f64>,
        client_algo_id: String,
    },
    GetOrder { symbol: String, order_id: i64 },
}

#[derive(Default)]
struct MockState {
    calls: Vec<Call>,
    mark_price: f64,
    available_balance: f64,
    /// Scripted responses for `place_market_order`. Popped in FIFO order.
    market_responses: Vec<Result<ExchangeOrder, LiveError>>,
    limit_responses: Vec<Result<ExchangeOrder, LiveError>>,
    tp_responses: Vec<Result<ExchangeOrder, LiveError>>,
    sl_responses: Vec<Result<ExchangeOrder, LiveError>>,
    /// For requery-on-incomplete-fill scenarios in close_position_market.
    get_order_responses: Vec<Result<ExchangeOrder, LiveError>>,
}

#[derive(Default)]
struct MockFuturesApi {
    state: Mutex<MockState>,
}

impl MockFuturesApi {
    fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }
    fn set_mark_price(&self, p: f64) {
        self.state.lock().unwrap().mark_price = p;
    }
    fn set_balance(&self, b: f64) {
        self.state.lock().unwrap().available_balance = b;
    }
    fn enqueue_market(&self, r: Result<ExchangeOrder, LiveError>) {
        self.state.lock().unwrap().market_responses.push(r);
    }
    fn enqueue_limit(&self, r: Result<ExchangeOrder, LiveError>) {
        self.state.lock().unwrap().limit_responses.push(r);
    }
    fn enqueue_tp(&self, r: Result<ExchangeOrder, LiveError>) {
        self.state.lock().unwrap().tp_responses.push(r);
    }
    fn enqueue_sl(&self, r: Result<ExchangeOrder, LiveError>) {
        self.state.lock().unwrap().sl_responses.push(r);
    }
    fn enqueue_get_order(&self, r: Result<ExchangeOrder, LiveError>) {
        self.state.lock().unwrap().get_order_responses.push(r);
    }
    fn calls(&self) -> Vec<Call> {
        self.state.lock().unwrap().calls.clone()
    }
}

fn err_for(missing: &str) -> LiveError {
    LiveError::Http(format!("MockFuturesApi: no scripted response for {missing}"))
}

impl FuturesApi for MockFuturesApi {
    fn server_now(&self) -> DateTime<Utc> {
        Utc::now()
    }
    fn place_market_order(
        &self,
        symbol: &str,
        side: OrderSide,
        quantity: f64,
        position_side: &str,
        client_order_id: &str,
    ) -> Result<ExchangeOrder, LiveError> {
        let mut s = self.state.lock().unwrap();
        s.calls.push(Call::PlaceMarket {
            symbol: symbol.into(),
            side,
            quantity,
            position_side: position_side.into(),
            client_order_id: client_order_id.into(),
        });
        if s.market_responses.is_empty() {
            return Err(err_for("place_market_order"));
        }
        s.market_responses.remove(0)
    }
    fn place_limit_order(
        &self,
        symbol: &str,
        side: OrderSide,
        quantity: f64,
        price: f64,
        position_side: &str,
        client_order_id: &str,
    ) -> Result<ExchangeOrder, LiveError> {
        let mut s = self.state.lock().unwrap();
        s.calls.push(Call::PlaceLimit {
            symbol: symbol.into(),
            side,
            quantity,
            price,
            position_side: position_side.into(),
            client_order_id: client_order_id.into(),
        });
        if s.limit_responses.is_empty() {
            return Err(err_for("place_limit_order"));
        }
        s.limit_responses.remove(0)
    }
    fn place_stop_market(
        &self,
        symbol: &str,
        side: OrderSide,
        stop_price: f64,
        position_side: &str,
        quantity: Option<f64>,
        client_algo_id: &str,
    ) -> Result<ExchangeOrder, LiveError> {
        let mut s = self.state.lock().unwrap();
        s.calls.push(Call::PlaceStopMarket {
            symbol: symbol.into(),
            side,
            stop_price,
            position_side: position_side.into(),
            quantity,
            client_algo_id: client_algo_id.into(),
        });
        if s.sl_responses.is_empty() {
            return Err(err_for("place_stop_market"));
        }
        s.sl_responses.remove(0)
    }
    fn place_take_profit_market(
        &self,
        symbol: &str,
        side: OrderSide,
        stop_price: f64,
        position_side: &str,
        quantity: Option<f64>,
        client_algo_id: &str,
    ) -> Result<ExchangeOrder, LiveError> {
        let mut s = self.state.lock().unwrap();
        s.calls.push(Call::PlaceTakeProfitMarket {
            symbol: symbol.into(),
            side,
            stop_price,
            position_side: position_side.into(),
            quantity,
            client_algo_id: client_algo_id.into(),
        });
        if s.tp_responses.is_empty() {
            return Err(err_for("place_take_profit_market"));
        }
        s.tp_responses.remove(0)
    }
    fn cancel_order(&self, _: &str, _: i64) -> Result<ExchangeOrder, LiveError> {
        Err(err_for("cancel_order"))
    }
    fn cancel_algo_order(&self, _: i64) -> Result<(), LiveError> {
        Err(err_for("cancel_algo_order"))
    }
    fn get_order(&self, symbol: &str, order_id: i64) -> Result<ExchangeOrder, LiveError> {
        let mut s = self.state.lock().unwrap();
        s.calls.push(Call::GetOrder {
            symbol: symbol.into(),
            order_id,
        });
        if s.get_order_responses.is_empty() {
            return Err(err_for("get_order"));
        }
        s.get_order_responses.remove(0)
    }
    fn get_order_by_client_id(
        &self,
        _: &str,
        _: &str,
    ) -> Result<Option<ExchangeOrder>, LiveError> {
        Err(err_for("get_order_by_client_id"))
    }
    fn get_algo_order(&self, _: i64) -> Result<ExchangeOrder, LiveError> {
        Err(err_for("get_algo_order"))
    }
    fn get_algo_order_by_client_id(
        &self,
        _: &str,
    ) -> Result<Option<ExchangeOrder>, LiveError> {
        Err(err_for("get_algo_order_by_client_id"))
    }
    fn get_open_orders(&self, _: Option<&str>) -> Result<Vec<ExchangeOrder>, LiveError> {
        Err(err_for("get_open_orders"))
    }
    fn get_position_info(
        &self,
        _: Option<&str>,
    ) -> Result<Vec<serde_json::Value>, LiveError> {
        Err(err_for("get_position_info"))
    }
    fn get_account_trades(
        &self,
        _: &str,
        _: Option<DateTime<Utc>>,
        _: Option<DateTime<Utc>>,
        _: Option<i64>,
        _: usize,
    ) -> Result<Vec<AccountTrade>, LiveError> {
        Err(err_for("get_account_trades"))
    }
    fn get_account_info(&self) -> Result<serde_json::Value, LiveError> {
        Err(err_for("get_account_info"))
    }
    fn get_available_balance(&self) -> Result<f64, LiveError> {
        let mut s = self.state.lock().unwrap();
        s.calls.push(Call::GetAvailableBalance);
        Ok(s.available_balance)
    }
    fn set_leverage(&self, symbol: &str, leverage: u32) -> Result<(), LiveError> {
        let mut s = self.state.lock().unwrap();
        s.calls.push(Call::SetLeverage {
            symbol: symbol.into(),
            leverage,
        });
        Ok(())
    }
    fn get_exchange_info(&self) -> Result<serde_json::Value, LiveError> {
        Err(err_for("get_exchange_info"))
    }
    fn get_mark_price(&self, symbol: &str) -> Result<f64, LiveError> {
        let mut s = self.state.lock().unwrap();
        s.calls.push(Call::GetMarkPrice {
            symbol: symbol.into(),
        });
        Ok(s.mark_price)
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn fixture_exchange_info() -> serde_json::Value {
    serde_json::json!({
        "symbols": [
            {
                "symbol": "BTCUSDT",
                "filters": [
                    {"filterType": "PRICE_FILTER",     "tickSize": "0.10",   "minPrice": "0.10"},
                    {"filterType": "LOT_SIZE",         "stepSize": "0.001",  "minQty": "0.001"},
                    {"filterType": "MARKET_LOT_SIZE",  "stepSize": "0.01",   "minQty": "0.01"},
                    {"filterType": "MIN_NOTIONAL",     "notional": "5.0"}
                ]
            },
            {
                "symbol": "DOGEUSDT",
                "filters": [
                    {"filterType": "PRICE_FILTER",  "tickSize": "0.0001"},
                    {"filterType": "LOT_SIZE",      "stepSize": "1",     "minQty": "1"},
                    {"filterType": "NOTIONAL",      "minNotional": "5.0"}
                ]
            }
        ]
    })
}

fn config() -> LiveConfig {
    LiveConfig {
        api_key: "k".into(),
        api_secret: "s".into(),
        base_url: "http://test".into(),
        position_size_usdt: 100.0,
        max_concurrent_positions: 3,
        order_check_interval_seconds: 5.0,
        testnet: false,
        recover_brackets_on_startup: true,
    }
}

fn build_executor(client: Arc<MockFuturesApi>) -> OrderExecutor {
    let info = ExchangeInfoCache::from_static(HashMap::new());
    info.populate_from_exchange_info(&fixture_exchange_info())
        .unwrap();
    OrderExecutor::with_exchange_info(client, config(), info)
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
        leverage: 5.0,
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

fn placed_market(qty: f64, fill_price: f64, client_id: &str) -> ExchangeOrder {
    ExchangeOrder {
        order_id: 999_001,
        symbol: "BTCUSDT".into(),
        side: OrderSide::Buy,
        order_type: OrderType::Market,
        quantity: qty,
        price: 0.0,
        stop_price: 0.0,
        status: OrderStatus::Filled,
        filled_qty: qty,
        avg_fill_price: fill_price,
        created_at: None,
        updated_at: None,
        algo_id: 0,
        client_order_id: client_id.into(),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[test]
fn prepare_then_submit_places_market_with_client_order_id() {
    let api = MockFuturesApi::new();
    api.set_mark_price(43_000.0);
    api.set_balance(1_000.0);
    let exec = build_executor(api.clone());

    let sig = long_signal("BTCUSDT");
    let mut prepared = exec.prepare_entry_position(&sig, Some(1_000.0), Some(100.0)).unwrap();

    // Placeholder shape:
    assert_eq!(prepared.position.status, PositionStatus::PendingEntry);
    let entry = prepared.position.entry_order.as_ref().unwrap();
    assert_eq!(entry.order_id, 0);
    assert!(!entry.client_order_id.is_empty());
    assert_eq!(prepared.position.fill_price, 0.0); // not yet filled
    assert!(prepared.margin_required > 0.0);
    let cid = entry.client_order_id.clone();

    // Now submit. Mock returns FILLED.
    api.enqueue_market(Ok(placed_market(prepared.position.quantity, 43_000.5, &cid)));
    exec.submit_entry_order(&mut prepared.position).unwrap();

    let entry = prepared.position.entry_order.as_ref().unwrap();
    assert_eq!(entry.order_id, 999_001);
    assert_eq!(entry.client_order_id, cid);

    // Verify the recorded calls.
    let calls = api.calls();
    let placed = calls
        .iter()
        .find(|c| matches!(c, Call::PlaceMarket { .. }))
        .expect("PlaceMarket recorded");
    if let Call::PlaceMarket {
        symbol,
        position_side,
        client_order_id,
        ..
    } = placed
    {
        assert_eq!(symbol, "BTCUSDT");
        assert_eq!(position_side, "LONG");
        assert_eq!(client_order_id, &cid);
    } else {
        unreachable!()
    }
}

#[test]
fn prepare_uses_market_lot_size_for_market_entries() {
    let api = MockFuturesApi::new();
    api.set_mark_price(43_000.0);
    api.set_balance(1_000.0);
    let exec = build_executor(api);

    // 100 USDT / 43000 ≈ 0.002326. With MARKET_LOT_SIZE step=0.01, min_qty=0.01:
    //   round_down → 0.0 → bumped to 0.01 (min_qty) → round_up at step 0.01 → 0.01
    let mut sig = long_signal("BTCUSDT");
    sig.entry_price = None; // market entry
    let prepared = exec.prepare_entry_position(&sig, Some(1_000.0), Some(100.0)).unwrap();
    let qty = prepared.position.quantity;
    assert!((qty - 0.01).abs() < 1e-12, "expected 0.01, got {qty}");
}

#[test]
fn prepare_uses_lot_size_for_limit_entries() {
    let api = MockFuturesApi::new();
    api.set_balance(1_000.0);
    let exec = build_executor(api);

    // Limit entry at 43_000.0. LOT_SIZE step=0.001, min_qty=0.001.
    // 100 USDT / 43_000 = 0.002326 → round_down at step 0.001 → 0.002 → already
    // above min_qty → 0.002 → final round_up → 0.002
    let mut sig = long_signal("BTCUSDT");
    sig.entry_price = Some(43_000.0);
    let prepared = exec.prepare_entry_position(&sig, Some(1_000.0), Some(100.0)).unwrap();
    let qty = prepared.position.quantity;
    assert!(
        (qty - 0.002).abs() < 1e-12,
        "expected 0.002, got {qty}",
    );
}

#[test]
fn prepare_size_multiplier_scales_quantity() {
    let api = MockFuturesApi::new();
    api.set_mark_price(43_000.0);
    api.set_balance(10_000.0);
    let exec = build_executor(api);

    let mut sig = long_signal("BTCUSDT");
    sig.size_multiplier = 2.0;
    sig.entry_price = Some(43_000.0); // limit so we use LOT_SIZE step 0.001
    // 100 USDT * 2.0 / 43_000 ≈ 0.00465 → round_down step 0.001 → 0.004
    let prepared = exec.prepare_entry_position(&sig, Some(10_000.0), Some(100.0)).unwrap();
    let qty = prepared.position.quantity;
    assert!(
        (qty - 0.004).abs() < 1e-12,
        "expected 0.004 (2× sizing), got {qty}",
    );
}

#[test]
fn prepare_min_notional_bumps_quantity_up() {
    let api = MockFuturesApi::new();
    api.set_mark_price(43_000.0);
    api.set_balance(1_000.0);
    let exec = build_executor(api);

    // tiny position_size_usdt => quantity below min_notional. Should bump up.
    // BTCUSDT min_notional = 5 USDT. 1 USDT / 43_000 ≈ 0.0000232 → step round
    // → 0 → bumped to min_qty (0.01 for market) → 0.01 * 43_000 = 430 USDT,
    // above min_notional. That's the market path.
    let mut sig = long_signal("BTCUSDT");
    sig.entry_price = None;
    let prepared = exec.prepare_entry_position(&sig, Some(1_000.0), Some(1.0)).unwrap();
    assert!(prepared.position.quantity >= 0.01);
}

#[test]
fn prepare_zero_quantity_rejected() {
    let api = MockFuturesApi::new();
    api.set_mark_price(43_000.0);
    api.set_balance(1_000.0);
    // Replace exchange_info: zero step+zero minimums + zero notional → quantity falls to zero.
    let raw = serde_json::json!({
        "symbols": [
            {
                "symbol": "BTCUSDT",
                "filters": [
                    {"filterType": "LOT_SIZE", "stepSize": "100", "minQty": "0"}
                ]
            }
        ]
    });
    let info = ExchangeInfoCache::from_static(HashMap::new());
    info.populate_from_exchange_info(&raw).unwrap();
    let exec = OrderExecutor::with_exchange_info(api, config(), info);

    // 1 USDT / 43000 ≈ 0.0000232 → round_down at step 100 → 0
    let mut sig = long_signal("BTCUSDT");
    sig.entry_price = Some(43_000.0); // limit, uses LOT_SIZE
    let err = exec
        .prepare_entry_position(&sig, Some(1_000.0), Some(1.0))
        .unwrap_err();
    assert!(matches!(err, LiveError::ZeroQuantity(_)), "got {err:?}");
}

#[test]
fn prepare_balance_check_uses_notional_over_leverage() {
    let api = MockFuturesApi::new();
    api.set_mark_price(43_000.0);
    let exec = build_executor(api);

    // Required margin = required_notional / leverage.
    // qty = 0.01 → notional = 430. leverage = 5 → margin = 86.
    // Set balance to 50 → balance < margin, expect rejection.
    let mut sig = long_signal("BTCUSDT");
    sig.entry_price = None; // market
    sig.leverage = 5.0;
    let err = exec
        .prepare_entry_position(&sig, Some(50.0), Some(100.0))
        .unwrap_err();
    match err {
        LiveError::InsufficientBalance { required, available } => {
            assert!(required > available, "required={required} available={available}");
            assert!(required > 80.0 && required < 90.0, "got required={required}");
            assert_eq!(available, 50.0);
        }
        other => panic!("expected InsufficientBalance, got {other:?}"),
    }
}

#[test]
fn prepare_uses_provided_balance_when_some() {
    let api = MockFuturesApi::new();
    api.set_mark_price(43_000.0);
    api.set_balance(0.0); // would fail, but available_balance arg overrides
    let exec = build_executor(api.clone());

    let sig = long_signal("BTCUSDT");
    let _ = exec
        .prepare_entry_position(&sig, Some(10_000.0), Some(100.0))
        .unwrap();

    // We did NOT call get_available_balance because we passed Some.
    let calls = api.calls();
    assert!(
        !calls.iter().any(|c| matches!(c, Call::GetAvailableBalance)),
        "should not have queried balance: {calls:?}",
    );
}

#[test]
fn leverage_set_only_once_per_symbol() {
    let api = MockFuturesApi::new();
    api.set_mark_price(43_000.0);
    api.set_balance(10_000.0);
    let exec = build_executor(api.clone());

    let sig = long_signal("BTCUSDT");
    let _ = exec.prepare_entry_position(&sig, Some(10_000.0), Some(100.0)).unwrap();
    let _ = exec.prepare_entry_position(&sig, Some(10_000.0), Some(100.0)).unwrap();

    let leverage_calls = api
        .calls()
        .iter()
        .filter(|c| matches!(c, Call::SetLeverage { .. }))
        .count();
    assert_eq!(leverage_calls, 1, "leverage cache should dedupe second call");
}

#[test]
fn place_tp_sl_uses_fill_price_not_signal_entry() {
    let api = MockFuturesApi::new();
    api.set_balance(1_000.0);
    api.set_mark_price(43_000.0);
    let exec = build_executor(api.clone());

    let sig = long_signal("BTCUSDT");
    let mut prepared = exec.prepare_entry_position(&sig, Some(1_000.0), Some(100.0)).unwrap();
    let cid = prepared.position.entry_order.as_ref().unwrap().client_order_id.clone();
    api.enqueue_market(Ok(placed_market(prepared.position.quantity, 43_500.0, &cid)));
    exec.submit_entry_order(&mut prepared.position).unwrap();
    // Simulate fill — the tracker (Phase D) will set this from the order
    // response on entry-fill detection. For Phase C we set it directly.
    prepared.position.fill_price = 43_500.0;

    api.enqueue_tp(Ok(ExchangeOrder {
        order_id: 0,
        symbol: "BTCUSDT".into(),
        side: OrderSide::Sell,
        order_type: OrderType::TakeProfitMarket,
        quantity: prepared.position.quantity,
        price: 0.0,
        stop_price: 44_375.0, // round-shaped, just for the response
        status: OrderStatus::New,
        filled_qty: 0.0,
        avg_fill_price: 0.0,
        created_at: None,
        updated_at: None,
        algo_id: 1_111_111,
        client_order_id: "tp-cid".into(),
    }));
    api.enqueue_sl(Ok(ExchangeOrder {
        order_id: 0,
        symbol: "BTCUSDT".into(),
        side: OrderSide::Sell,
        order_type: OrderType::StopMarket,
        quantity: prepared.position.quantity,
        price: 0.0,
        stop_price: 43_065.0,
        status: OrderStatus::New,
        filled_qty: 0.0,
        avg_fill_price: 0.0,
        created_at: None,
        updated_at: None,
        algo_id: 2_222_222,
        client_order_id: "sl-cid".into(),
    }));
    exec.place_tp_sl(&mut prepared.position).unwrap();

    let calls = api.calls();
    let tp_call = calls
        .iter()
        .find(|c| matches!(c, Call::PlaceTakeProfitMarket { .. }))
        .expect("TP placement recorded");
    let sl_call = calls
        .iter()
        .find(|c| matches!(c, Call::PlaceStopMarket { .. }))
        .expect("SL placement recorded");

    if let Call::PlaceTakeProfitMarket {
        stop_price,
        quantity,
        client_algo_id,
        ..
    } = tp_call
    {
        // For LONG, TP > fill_price.  fill = 43500, tp_pct = 2%, fee_offset = 0.1%
        // → tp_with_fees = 2.1%, raw = 43500 * 1.021 = 44413.5
        // → rounded at tickSize 0.10 → 44413.5
        // (Python computes the same; what matters is it's > fill, not the
        // entry signal price.)
        assert!(
            *stop_price > 43_500.0,
            "TP stop_price {stop_price} should be above fill price 43_500.0",
        );
        assert!((*stop_price - 44_413.5).abs() < 1.0, "got {stop_price}");
        assert_eq!(*quantity, Some(prepared.position.quantity));
        assert_eq!(client_algo_id.len(), 32);
    } else {
        unreachable!()
    }
    if let Call::PlaceStopMarket { stop_price, quantity, .. } = sl_call {
        // SL < fill_price for LONG.
        assert!(*stop_price < 43_500.0, "SL {stop_price} should be below fill");
        assert_eq!(*quantity, Some(prepared.position.quantity));
    } else {
        unreachable!()
    }

    // Position now carries the placed orders.
    assert_eq!(prepared.position.tp_order.as_ref().unwrap().algo_id, 1_111_111);
    assert_eq!(prepared.position.sl_order.as_ref().unwrap().algo_id, 2_222_222);
}

#[test]
fn place_tp_sl_rejects_zero_fill_price() {
    let api = MockFuturesApi::new();
    let exec = build_executor(api);
    let mut pos = LivePosition {
        signal: long_signal("BTCUSDT"),
        position_id: "x".into(),
        strategy_id: String::new(),
        status: PositionStatus::Open,
        entry_order: None,
        tp_order: None,
        sl_order: None,
        fill_price: 0.0,
        quantity: 0.01,
        opened_at: None,
        exit_price: None,
        pnl_pct: None,
        gross_pnl_pct: None,
        fee_drag_pct: None,
        closed_at: None,
    };
    let err = exec.place_tp_sl(&mut pos).unwrap_err();
    assert!(matches!(err, LiveError::State(_)), "got {err:?}");
}

#[test]
fn close_position_market_rounds_quantity_down_with_market_filter() {
    let api = MockFuturesApi::new();
    let exec = build_executor(api.clone());

    // Position quantity 0.0199 — MARKET_LOT_SIZE step 0.01 → round down to 0.01.
    let pos = LivePosition {
        signal: long_signal("BTCUSDT"),
        position_id: "p".into(),
        strategy_id: String::new(),
        status: PositionStatus::Open,
        entry_order: None,
        tp_order: None,
        sl_order: None,
        fill_price: 43_000.0,
        quantity: 0.0199,
        opened_at: None,
        exit_price: None,
        pnl_pct: None,
        gross_pnl_pct: None,
        fee_drag_pct: None,
        closed_at: None,
    };
    api.enqueue_market(Ok(ExchangeOrder {
        order_id: 1,
        symbol: "BTCUSDT".into(),
        side: OrderSide::Sell,
        order_type: OrderType::Market,
        quantity: 0.01,
        price: 0.0,
        stop_price: 0.0,
        status: OrderStatus::Filled,
        filled_qty: 0.01,
        avg_fill_price: 43_010.0,
        created_at: None,
        updated_at: None,
        algo_id: 0,
        client_order_id: "close-cid".into(),
    }));
    let exit = exec.close_position_market(&pos).unwrap();
    assert_eq!(exit.status, OrderStatus::Filled);

    let calls = api.calls();
    let placed = calls
        .iter()
        .find_map(|c| {
            if let Call::PlaceMarket { quantity, side, .. } = c {
                Some((*quantity, *side))
            } else {
                None
            }
        })
        .unwrap();
    assert!((placed.0 - 0.01).abs() < 1e-12, "expected 0.01, got {}", placed.0);
    assert_eq!(placed.1, OrderSide::Sell, "long close → sell");
}

#[test]
fn close_position_market_requeries_when_response_incomplete() {
    let api = MockFuturesApi::new();
    let exec = build_executor(api.clone());

    let pos = LivePosition {
        signal: long_signal("BTCUSDT"),
        position_id: "p".into(),
        strategy_id: String::new(),
        status: PositionStatus::Open,
        entry_order: None,
        tp_order: None,
        sl_order: None,
        fill_price: 43_000.0,
        quantity: 0.01,
        opened_at: None,
        exit_price: None,
        pnl_pct: None,
        gross_pnl_pct: None,
        fee_drag_pct: None,
        closed_at: None,
    };
    // Initial response: NEW, no avg fill price → triggers requery.
    api.enqueue_market(Ok(ExchangeOrder {
        order_id: 555,
        symbol: "BTCUSDT".into(),
        side: OrderSide::Sell,
        order_type: OrderType::Market,
        quantity: 0.01,
        price: 0.0,
        stop_price: 0.0,
        status: OrderStatus::New,
        filled_qty: 0.0,
        avg_fill_price: 0.0,
        created_at: None,
        updated_at: None,
        algo_id: 0,
        client_order_id: "close-cid".into(),
    }));
    // Requery returns the filled order.
    api.enqueue_get_order(Ok(ExchangeOrder {
        order_id: 555,
        symbol: "BTCUSDT".into(),
        side: OrderSide::Sell,
        order_type: OrderType::Market,
        quantity: 0.01,
        price: 0.0,
        stop_price: 0.0,
        status: OrderStatus::Filled,
        filled_qty: 0.01,
        avg_fill_price: 43_010.0,
        created_at: None,
        updated_at: None,
        algo_id: 0,
        client_order_id: "close-cid".into(),
    }));
    let exit = exec.close_position_market(&pos).unwrap();
    assert_eq!(exit.status, OrderStatus::Filled);
    assert_eq!(exit.avg_fill_price, 43_010.0);

    // Verify requery happened.
    let calls = api.calls();
    assert_eq!(
        calls
            .iter()
            .filter(|c| matches!(c, Call::GetOrder { .. }))
            .count(),
        1,
    );
}

#[test]
fn close_rejects_when_quantity_rounds_to_zero() {
    let api = MockFuturesApi::new();
    let exec = build_executor(api);
    let mut pos = LivePosition {
        signal: long_signal("BTCUSDT"),
        position_id: "p".into(),
        strategy_id: String::new(),
        status: PositionStatus::Open,
        entry_order: None,
        tp_order: None,
        sl_order: None,
        fill_price: 43_000.0,
        quantity: 0.0001, // below MARKET_LOT_SIZE step 0.01 → rounds to 0
        opened_at: None,
        exit_price: None,
        pnl_pct: None,
        gross_pnl_pct: None,
        fee_drag_pct: None,
        closed_at: None,
    };
    pos.quantity = 0.001; // below MARKET_LOT_SIZE 0.01 step → rounds down to 0
    let err = exec.close_position_market(&pos).unwrap_err();
    assert!(matches!(err, LiveError::ZeroQuantity(_)));
}

#[test]
fn submit_uses_limit_when_signal_has_entry_price() {
    let api = MockFuturesApi::new();
    api.set_balance(1_000.0);
    let exec = build_executor(api.clone());

    let mut sig = long_signal("BTCUSDT");
    sig.entry_price = Some(42_999.99); // not on tickSize boundary
    let mut prepared = exec.prepare_entry_position(&sig, Some(1_000.0), Some(100.0)).unwrap();
    let cid = prepared.position.entry_order.as_ref().unwrap().client_order_id.clone();
    api.enqueue_limit(Ok(ExchangeOrder {
        order_id: 1,
        symbol: "BTCUSDT".into(),
        side: OrderSide::Buy,
        order_type: OrderType::Limit,
        quantity: prepared.position.quantity,
        price: 42_999.9,
        stop_price: 0.0,
        status: OrderStatus::New,
        filled_qty: 0.0,
        avg_fill_price: 0.0,
        created_at: None,
        updated_at: None,
        algo_id: 0,
        client_order_id: cid.clone(),
    }));
    exec.submit_entry_order(&mut prepared.position).unwrap();

    let calls = api.calls();
    let limit = calls
        .iter()
        .find_map(|c| {
            if let Call::PlaceLimit { price, client_order_id, .. } = c {
                Some((*price, client_order_id.clone()))
            } else {
                None
            }
        })
        .expect("PlaceLimit recorded");
    // 42_999.99 floored to tickSize 0.10 → 42_999.9
    assert!((limit.0 - 42_999.9).abs() < 1e-9, "got {}", limit.0);
    assert_eq!(limit.1, cid);
}
