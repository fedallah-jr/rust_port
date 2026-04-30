//! Order executor — Signal → exchange orders.
//!
//! Mirrors `live/executor.OrderExecutor`. The Phase C plan v2 §4.5 / §1.1
//! design splits entry placement into two phases so a 5xx mid-flight is
//! recoverable:
//!
//!   1. `prepare_entry_position(signal, balance, position_size_usdt)` —
//!      pre-flight only. Validates filters, sizes the position, sets
//!      leverage if needed, generates a `client_order_id`, and returns a
//!      `PreparedEntry` carrying a placeholder `LivePosition` (status
//!      `PendingEntry`, `entry_order.order_id == 0`, `client_order_id` set).
//!      No network calls happen *to the order endpoint*.
//!
//!   2. *Tracker persists the placeholder.* Phase D's
//!      `PositionTracker::place_entry` is the only legitimate call site —
//!      it writes the placeholder to disk before invoking submit.
//!
//!   3. `submit_entry_order(&mut LivePosition)` — POSTs the order. Internally
//!      uses the idempotent-recovery path from `auth_client::execute_order_post`,
//!      so a 5xx / network failure during the POST will query Binance by
//!      `client_order_id` and either adopt the existing order or re-POST.
//!
//! ## Bypass guard
//!
//! `submit_entry_order` enforces the placeholder shape at runtime so callers
//! cannot fabricate a `LivePosition` and skip prepare. The check is:
//!
//!   - `status == PendingEntry`
//!   - `entry_order.is_some()` and `order_id == 0` (placement hasn't returned)
//!   - `entry_order.client_order_id` is non-empty (prepare set it)
//!
//! Phase D's tracker locks down the call path further by owning the
//! prepare→persist→submit sequence as one method. The runtime check here is
//! the belt-and-suspenders guard that survives even when the engine is wired
//! by hand in tests or future research code.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use claude_trader_models::{
    LiveConfig, LivePosition, OrderSide, OrderStatus, PositionStatus, PositionType, Signal,
};
use claude_trader_resolver::compute_tp_sl_prices_from_signal;
use uuid::Uuid;

use crate::auth_client::{symbol_for_api, FuturesApi};
use crate::error::{LiveError, Result};
use crate::exchange_info::ExchangeInfoCache;

// ---------------------------------------------------------------------------
// PreparedEntry
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct PreparedEntry {
    /// Placeholder position. `status == PendingEntry`, `entry_order` set with
    /// `order_id == 0` and `client_order_id` populated. Caller (the tracker
    /// in Phase D, the engine in Phase E) must persist this before calling
    /// `submit_entry_order`.
    pub position: LivePosition,
    /// Margin (USDT) the placement will consume — `notional / effective_leverage`.
    /// Used by the engine's capital-allocation cascade to decrement the
    /// `remaining_balance` running counter.
    pub margin_required: f64,
}

// ---------------------------------------------------------------------------
// OrderExecutor
// ---------------------------------------------------------------------------

pub struct OrderExecutor {
    client: Arc<dyn FuturesApi>,
    config: LiveConfig,
    info: ExchangeInfoCache,
    leverage_cache: Mutex<HashMap<String, u32>>,
}

impl OrderExecutor {
    pub fn new(client: Arc<dyn FuturesApi>, config: LiveConfig) -> Self {
        let info = ExchangeInfoCache::new(client.clone());
        Self {
            client,
            config,
            info,
            leverage_cache: Mutex::new(HashMap::new()),
        }
    }

    /// Test/reuse constructor: caller supplies a (potentially pre-populated)
    /// `ExchangeInfoCache`.
    pub fn with_exchange_info(
        client: Arc<dyn FuturesApi>,
        config: LiveConfig,
        info: ExchangeInfoCache,
    ) -> Self {
        Self {
            client,
            config,
            info,
            leverage_cache: Mutex::new(HashMap::new()),
        }
    }

    pub fn exchange_info(&self) -> &ExchangeInfoCache {
        &self.info
    }

    // -- Phase 1: prepare ---------------------------------------------------

    pub fn prepare_entry_position(
        &self,
        signal: &Signal,
        available_balance: Option<f64>,
        position_size_usdt: Option<f64>,
    ) -> Result<PreparedEntry> {
        let api_symbol = symbol_for_api(&signal.ticker);
        self.info.ensure_loaded(&signal.ticker)?;

        // Set leverage if needed. Cached per symbol so repeat signals on the
        // same ticker don't burn weight on a second call.
        let target_leverage = effective_leverage_u32(signal);
        if target_leverage > 0 {
            let needs_set = {
                let cache = self.leverage_cache.lock().unwrap();
                cache.get(&api_symbol).copied() != Some(target_leverage)
            };
            if needs_set {
                self.client.set_leverage(&signal.ticker, target_leverage)?;
                self.leverage_cache
                    .lock()
                    .unwrap()
                    .insert(api_symbol.clone(), target_leverage);
            }
        }

        // Determine entry price for sizing.
        let entry_price = match signal.entry_price {
            Some(p) => p,
            None => self.client.get_mark_price(&signal.ticker)?,
        };

        let effective_size = position_size_usdt.unwrap_or(self.config.position_size_usdt);
        let use_market_filter = signal.entry_price.is_none();
        let (quantity, required_notional) = self.compute_entry_quantity(
            &signal.ticker,
            entry_price,
            effective_size,
            signal.size_multiplier,
            use_market_filter,
        )?;

        // Balance check: required margin = notional / leverage.
        let effective_leverage = effective_leverage_f64(signal);
        let required_margin = required_notional / effective_leverage;
        let bal = match available_balance {
            Some(b) => b,
            None => self.client.get_available_balance()?,
        };
        if bal + 1e-9 < required_margin {
            return Err(LiveError::InsufficientBalance {
                required: required_margin,
                available: bal,
            });
        }

        // Build the placeholder.
        let client_order_id = new_client_order_id();
        let position_id = uuid_short();
        let placeholder_entry = claude_trader_models::ExchangeOrder {
            order_id: 0,
            symbol: api_symbol.clone(),
            side: side_for_signal(signal),
            order_type: order_type_for_signal(signal),
            quantity,
            price: signal.entry_price.unwrap_or(0.0),
            stop_price: 0.0,
            status: OrderStatus::New,
            filled_qty: 0.0,
            avg_fill_price: 0.0,
            created_at: None,
            updated_at: None,
            algo_id: 0,
            client_order_id: client_order_id.clone(),
        };
        let position = LivePosition {
            signal: signal.clone(),
            position_id,
            strategy_id: String::new(), // engine sets after prepare returns
            status: PositionStatus::PendingEntry,
            entry_order: Some(placeholder_entry),
            tp_order: None,
            sl_order: None,
            fill_price: 0.0,
            quantity,
            opened_at: None,
            exit_price: None,
            pnl_pct: None,
            gross_pnl_pct: None,
            fee_drag_pct: None,
            closed_at: None,
        };
        Ok(PreparedEntry {
            position,
            margin_required: required_margin,
        })
    }

    // -- Phase 2: submit ----------------------------------------------------

    /// Submits the entry order POST for a position previously returned by
    /// `prepare_entry_position`. The runtime guard rejects positions that
    /// don't bear the placeholder shape — callers cannot fabricate a
    /// `LivePosition` and skip prepare.
    ///
    /// Phase D's `PositionTracker::place_entry` is the only intended caller;
    /// it persists the position to disk *before* invoking this method, so a
    /// 5xx / network outcome during the POST is recoverable on next startup
    /// via the persisted `client_order_id`.
    pub fn submit_entry_order(&self, position: &mut LivePosition) -> Result<()> {
        let entry = position
            .entry_order
            .as_ref()
            .ok_or_else(|| {
                LiveError::State(
                    "submit_entry_order: position has no entry_order placeholder. \
                     Call prepare_entry_position first."
                        .into(),
                )
            })?;
        if position.status != PositionStatus::PendingEntry {
            return Err(LiveError::State(format!(
                "submit_entry_order: position status is {:?}, expected PendingEntry. \
                 Call prepare_entry_position first.",
                position.status,
            )));
        }
        if entry.order_id != 0 {
            return Err(LiveError::State(format!(
                "submit_entry_order: entry_order.order_id={} (must be 0 — placement \
                 has already returned). Call prepare_entry_position first.",
                entry.order_id,
            )));
        }
        if entry.client_order_id.is_empty() {
            return Err(LiveError::State(
                "submit_entry_order: entry_order.client_order_id is empty. \
                 Call prepare_entry_position first."
                    .into(),
            ));
        }

        let signal = &position.signal;
        let side = entry.side;
        let position_side = position_side_for(signal);
        let client_order_id = entry.client_order_id.clone();
        let quantity = position.quantity;

        let placed = match signal.entry_price {
            Some(price) => {
                let rounded_price = self.info.round_price(&signal.ticker, price)?;
                self.client.place_limit_order(
                    &signal.ticker,
                    side,
                    quantity,
                    rounded_price,
                    position_side,
                    &client_order_id,
                )?
            }
            None => self.client.place_market_order(
                &signal.ticker,
                side,
                quantity,
                position_side,
                &client_order_id,
            )?,
        };
        position.entry_order = Some(placed);
        Ok(())
    }

    // -- TP/SL placement (two-phase) ---------------------------------------

    /// Phase 1: build TP+SL placeholders against the actual fill price and
    /// stash them on the position. **No network calls** — the tracker is
    /// responsible for `save_state` between this and `submit_brackets` so
    /// a 5xx/network failure during submit leaves the placeholder (with
    /// its `clientAlgoId`) durably on disk for startup recovery to find.
    pub fn prepare_brackets(&self, position: &mut LivePosition) -> Result<()> {
        if position.tp_order.is_none() {
            self.prepare_tp_placeholder(position)?;
        }
        if position.sl_order.is_none() {
            self.prepare_sl_placeholder(position)?;
        }
        Ok(())
    }

    /// Phase 1 (TP-only): construct a placeholder ExchangeOrder and assign
    /// to `position.tp_order`. The placeholder carries
    /// `algo_id=0`, `order_id=0`, freshly generated `client_order_id`,
    /// `status=NEW`, and the rounded `stop_price`. Bracket recovery on
    /// startup uses `client_order_id` (via `get_algo_order_by_client_id`)
    /// to determine whether the eventual POST landed.
    pub fn prepare_tp_placeholder(&self, position: &mut LivePosition) -> Result<()> {
        let signal = &position.signal;
        if position.fill_price <= 0.0 {
            return Err(LiveError::State(format!(
                "prepare_tp_placeholder: fill_price={} (must be > 0)",
                position.fill_price,
            )));
        }
        self.info.ensure_loaded(&signal.ticker)?;
        let (raw_tp, _) = compute_tp_sl_prices_from_signal(position.fill_price, signal)
            .map_err(LiveError::Parse)?;
        let tp_price = self.info.round_price(&signal.ticker, raw_tp)?;
        position.tp_order = Some(claude_trader_models::ExchangeOrder {
            order_id: 0,
            symbol: symbol_for_api(&signal.ticker),
            side: close_side_for(signal),
            order_type: claude_trader_models::OrderType::TakeProfitMarket,
            quantity: position.quantity,
            price: 0.0,
            stop_price: tp_price,
            status: OrderStatus::New,
            filled_qty: 0.0,
            avg_fill_price: 0.0,
            created_at: None,
            updated_at: None,
            algo_id: 0,
            client_order_id: new_client_order_id(),
        });
        Ok(())
    }

    /// Phase 1 (SL-only). Symmetric counterpart of `prepare_tp_placeholder`.
    pub fn prepare_sl_placeholder(&self, position: &mut LivePosition) -> Result<()> {
        let signal = &position.signal;
        if position.fill_price <= 0.0 {
            return Err(LiveError::State(format!(
                "prepare_sl_placeholder: fill_price={} (must be > 0)",
                position.fill_price,
            )));
        }
        self.info.ensure_loaded(&signal.ticker)?;
        let (_, raw_sl) = compute_tp_sl_prices_from_signal(position.fill_price, signal)
            .map_err(LiveError::Parse)?;
        let sl_price = self.info.round_price(&signal.ticker, raw_sl)?;
        position.sl_order = Some(claude_trader_models::ExchangeOrder {
            order_id: 0,
            symbol: symbol_for_api(&signal.ticker),
            side: close_side_for(signal),
            order_type: claude_trader_models::OrderType::StopMarket,
            quantity: position.quantity,
            price: 0.0,
            stop_price: sl_price,
            status: OrderStatus::New,
            filled_qty: 0.0,
            avg_fill_price: 0.0,
            created_at: None,
            updated_at: None,
            algo_id: 0,
            client_order_id: new_client_order_id(),
        });
        Ok(())
    }

    /// Phase 2: POST any unsubmitted placeholders. A "placeholder" is a
    /// `Some(order)` with `algo_id == 0` and a non-empty `client_order_id`.
    /// Already-submitted slots (algo_id > 0) are skipped — useful for
    /// retries after a partial failure. Returns the first error encountered
    /// while still attempting the other side.
    pub fn submit_brackets(&self, position: &mut LivePosition) -> Result<()> {
        let mut first_err: Option<LiveError> = None;
        if matches!(position.tp_order.as_ref(), Some(o) if o.algo_id == 0 && !o.client_order_id.is_empty()) {
            if let Err(e) = self.submit_tp_only(position) {
                first_err.get_or_insert(e);
            }
        }
        if matches!(position.sl_order.as_ref(), Some(o) if o.algo_id == 0 && !o.client_order_id.is_empty()) {
            if let Err(e) = self.submit_sl_only(position) {
                first_err.get_or_insert(e);
            }
        }
        match first_err {
            Some(e) => Err(e),
            None => Ok(()),
        }
    }

    /// Phase 2 (TP). POSTs using the placeholder's `client_order_id` so a
    /// network/5xx failure leaves the position state recoverable: the
    /// placeholder's cid persists on disk, and the auth-client's
    /// idempotency-recovery loop can identify whether the order actually
    /// reached Binance via `clientAlgoId` lookup.
    pub fn submit_tp_only(&self, position: &mut LivePosition) -> Result<()> {
        let placeholder = position
            .tp_order
            .clone()
            .ok_or_else(|| LiveError::State("submit_tp_only: no tp placeholder".into()))?;
        if placeholder.algo_id != 0 {
            // Already submitted (e.g. resume-after-restart that already
            // adopted via classify_bracket). No-op.
            return Ok(());
        }
        if placeholder.client_order_id.is_empty() {
            return Err(LiveError::State(
                "submit_tp_only: placeholder has no clientAlgoId — \
                 prepare_tp_placeholder must run first"
                    .into(),
            ));
        }
        let posted = self.client.place_take_profit_market(
            &position.signal.ticker,
            placeholder.side,
            placeholder.stop_price,
            position_side_for(&position.signal),
            Some(position.quantity),
            &placeholder.client_order_id,
        )?;
        position.tp_order = Some(posted);
        Ok(())
    }

    /// Phase 2 (SL). Symmetric counterpart of `submit_tp_only`.
    pub fn submit_sl_only(&self, position: &mut LivePosition) -> Result<()> {
        let placeholder = position
            .sl_order
            .clone()
            .ok_or_else(|| LiveError::State("submit_sl_only: no sl placeholder".into()))?;
        if placeholder.algo_id != 0 {
            return Ok(());
        }
        if placeholder.client_order_id.is_empty() {
            return Err(LiveError::State(
                "submit_sl_only: placeholder has no clientAlgoId — \
                 prepare_sl_placeholder must run first"
                    .into(),
            ));
        }
        let posted = self.client.place_stop_market(
            &position.signal.ticker,
            placeholder.side,
            placeholder.stop_price,
            position_side_for(&position.signal),
            Some(position.quantity),
            &placeholder.client_order_id,
        )?;
        position.sl_order = Some(posted);
        Ok(())
    }

    /// One-shot convenience: prepare + submit both sides. Use this only
    /// from contexts that don't need the persist-between-phases guarantee
    /// (tests, throwaway scripts). The tracker drives the explicit two-phase
    /// flow because crash safety requires `save_state` between phases.
    pub fn place_tp_sl(&self, position: &mut LivePosition) -> Result<()> {
        self.prepare_brackets(position)?;
        self.submit_brackets(position)
    }

    // -- Timeout close ------------------------------------------------------

    /// Close an open position with an opposing market order. Quantity rounds
    /// DOWN with `MARKET_LOT_SIZE` so we never oversell. If the response is
    /// missing fill data we requery once via `get_order` to get the executed
    /// price for PnL math.
    pub fn close_position_market(
        &self,
        position: &LivePosition,
    ) -> Result<claude_trader_models::ExchangeOrder> {
        let signal = &position.signal;
        self.info.ensure_loaded(&signal.ticker)?;
        let qty = self
            .info
            .round_quantity_down(&signal.ticker, position.quantity, true)?;
        if qty <= 0.0 {
            return Err(LiveError::ZeroQuantity(format!(
                "close {}: rounded quantity is zero",
                signal.ticker
            )));
        }

        let close_side = match signal.position_type {
            PositionType::Long => OrderSide::Sell,
            PositionType::Short => OrderSide::Buy,
        };
        let position_side = position_side_for(signal);
        let client_order_id = new_client_order_id();
        let mut placed = self.client.place_market_order(
            &signal.ticker,
            close_side,
            qty,
            position_side,
            &client_order_id,
        )?;

        // Mirror Python: if the immediate response isn't fully resolved,
        // requery for the executed price/quantity used by PnL.
        if placed.status != OrderStatus::Filled || placed.avg_fill_price <= 0.0 {
            placed = self.client.get_order(&signal.ticker, placed.order_id)?;
        }
        Ok(placed)
    }

    // -- Sizing -------------------------------------------------------------

    fn compute_entry_quantity(
        &self,
        ticker: &str,
        entry_price: f64,
        effective_size_usdt: f64,
        size_multiplier: f64,
        use_market_filter: bool,
    ) -> Result<(f64, f64)> {
        if entry_price <= 0.0 {
            return Err(LiveError::ZeroQuantity(format!(
                "{ticker}: entry_price={entry_price} (must be > 0)"
            )));
        }
        let raw_qty = effective_size_usdt * size_multiplier / entry_price;

        // Initial round DOWN (don't accidentally oversize on filter rounding).
        let mut qty = self
            .info
            .round_quantity_down(ticker, raw_qty, use_market_filter)?;

        let min_qty = self.info.min_qty(ticker, use_market_filter)?;
        let min_notional = self.info.min_notional(ticker)?;
        if min_qty > 0.0 {
            qty = qty.max(min_qty);
        }
        if min_notional > 0.0 && qty * entry_price + 1e-9 < min_notional {
            qty = qty.max(min_notional / entry_price);
        }

        // Final round UP so the bumps survive step alignment.
        qty = self
            .info
            .round_quantity_up(ticker, qty, use_market_filter)?;
        if qty <= 0.0 {
            return Err(LiveError::ZeroQuantity(format!(
                "{ticker}: rounded quantity is zero"
            )));
        }

        let required_notional = qty * entry_price;
        if min_notional > 0.0 && required_notional + 1e-9 < min_notional {
            return Err(LiveError::ZeroQuantity(format!(
                "{ticker}: required notional {required_notional:.4} < min_notional {min_notional:.4}"
            )));
        }
        Ok((qty, required_notional))
    }
}

// ---------------------------------------------------------------------------
// Free helpers
// ---------------------------------------------------------------------------

fn side_for_signal(signal: &Signal) -> OrderSide {
    match signal.position_type {
        PositionType::Long => OrderSide::Buy,
        PositionType::Short => OrderSide::Sell,
    }
}

fn close_side_for(signal: &Signal) -> OrderSide {
    match signal.position_type {
        PositionType::Long => OrderSide::Sell,
        PositionType::Short => OrderSide::Buy,
    }
}

fn order_type_for_signal(signal: &Signal) -> claude_trader_models::OrderType {
    if signal.entry_price.is_some() {
        claude_trader_models::OrderType::Limit
    } else {
        claude_trader_models::OrderType::Market
    }
}

fn position_side_for(signal: &Signal) -> &'static str {
    match signal.position_type {
        PositionType::Long => "LONG",
        PositionType::Short => "SHORT",
    }
}

fn effective_leverage_f64(signal: &Signal) -> f64 {
    if signal.leverage.is_finite() && signal.leverage > 0.0 {
        signal.leverage
    } else {
        1.0
    }
}

fn effective_leverage_u32(signal: &Signal) -> u32 {
    let f = effective_leverage_f64(signal);
    f.round().max(1.0) as u32
}

/// Generate a fresh `client_order_id` / `clientAlgoId`. UUID v4 simple format
/// is 32 alphanumeric chars, well under Binance's 36-char limit, no special
/// characters that need URL encoding.
pub fn new_client_order_id() -> String {
    Uuid::new_v4().simple().to_string()
}

/// Short position id (12 hex chars), matching Python's `uuid.uuid4().hex[:12]`.
fn uuid_short() -> String {
    let s = Uuid::new_v4().simple().to_string();
    s[..12].to_string()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    //! Bypass-guard tests for `submit_entry_order`. The full
    //! prepare→submit flow + sizing + TP/SL + close lives in
    //! `tests/executor_flow.rs` so tests share the mock-FuturesApi
    //! harness with the auth_client tests.

    use super::*;
    use chrono::Utc;
    use claude_trader_models::{MarketType, PositionStatus};

    fn signal() -> Signal {
        Signal {
            signal_date: Utc::now(),
            position_type: PositionType::Long,
            ticker: "BTCUSDT".into(),
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

    fn position(state: PositionStatus, entry_id: i64, client_id: &str) -> LivePosition {
        LivePosition {
            signal: signal(),
            position_id: "test-id".into(),
            strategy_id: String::new(),
            status: state,
            entry_order: Some(claude_trader_models::ExchangeOrder {
                order_id: entry_id,
                symbol: "BTCUSDT".into(),
                side: OrderSide::Buy,
                order_type: claude_trader_models::OrderType::Market,
                quantity: 0.01,
                price: 0.0,
                stop_price: 0.0,
                status: OrderStatus::New,
                filled_qty: 0.0,
                avg_fill_price: 0.0,
                created_at: None,
                updated_at: None,
                algo_id: 0,
                client_order_id: client_id.into(),
            }),
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
        }
    }

    /// A do-nothing FuturesApi — guard checks happen before any client call.
    struct PanicApi;
    impl FuturesApi for PanicApi {
        fn server_now(&self) -> chrono::DateTime<Utc> {
            unreachable!()
        }
        fn place_market_order(
            &self,
            _: &str,
            _: OrderSide,
            _: f64,
            _: &str,
            _: &str,
        ) -> Result<claude_trader_models::ExchangeOrder> {
            panic!("place_market_order should not be called when guard rejects");
        }
        fn place_limit_order(
            &self,
            _: &str,
            _: OrderSide,
            _: f64,
            _: f64,
            _: &str,
            _: &str,
        ) -> Result<claude_trader_models::ExchangeOrder> {
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
        ) -> Result<claude_trader_models::ExchangeOrder> {
            unreachable!()
        }
        fn place_take_profit_market(
            &self,
            _: &str,
            _: OrderSide,
            _: f64,
            _: &str,
            _: Option<f64>,
            _: &str,
        ) -> Result<claude_trader_models::ExchangeOrder> {
            unreachable!()
        }
        fn cancel_order(
            &self,
            _: &str,
            _: i64,
        ) -> Result<claude_trader_models::ExchangeOrder> {
            unreachable!()
        }
        fn cancel_algo_order(&self, _: i64) -> Result<()> {
            unreachable!()
        }
        fn get_order(
            &self,
            _: &str,
            _: i64,
        ) -> Result<claude_trader_models::ExchangeOrder> {
            unreachable!()
        }
        fn get_order_by_client_id(
            &self,
            _: &str,
            _: &str,
        ) -> Result<Option<claude_trader_models::ExchangeOrder>> {
            unreachable!()
        }
        fn get_algo_order(
            &self,
            _: i64,
        ) -> Result<claude_trader_models::ExchangeOrder> {
            unreachable!()
        }
        fn get_algo_order_by_client_id(
            &self,
            _: &str,
        ) -> Result<Option<claude_trader_models::ExchangeOrder>> {
            unreachable!()
        }
        fn get_open_orders(
            &self,
            _: Option<&str>,
        ) -> Result<Vec<claude_trader_models::ExchangeOrder>> {
            unreachable!()
        }
        fn get_position_info(
            &self,
            _: Option<&str>,
        ) -> Result<Vec<serde_json::Value>> {
            unreachable!()
        }
        fn get_account_trades(
            &self,
            _: &str,
            _: Option<chrono::DateTime<Utc>>,
            _: Option<chrono::DateTime<Utc>>,
            _: Option<i64>,
            _: usize,
        ) -> Result<Vec<claude_trader_models::AccountTrade>> {
            unreachable!()
        }
        fn get_account_info(&self) -> Result<serde_json::Value> {
            unreachable!()
        }
        fn get_available_balance(&self) -> Result<f64> {
            unreachable!()
        }
        fn set_leverage(&self, _: &str, _: u32) -> Result<()> {
            unreachable!()
        }
        fn get_exchange_info(&self) -> Result<serde_json::Value> {
            unreachable!()
        }
        fn get_mark_price(&self, _: &str) -> Result<f64> {
            unreachable!()
        }
    }

    fn executor() -> OrderExecutor {
        let info = ExchangeInfoCache::from_static(HashMap::new());
        OrderExecutor::with_exchange_info(
            Arc::new(PanicApi),
            LiveConfig {
                api_key: "k".into(),
                api_secret: "s".into(),
                base_url: "http://test".into(),
                position_size_usdt: 100.0,
                max_concurrent_positions: 3,
                order_check_interval_seconds: 5.0,
                testnet: false,
                recover_brackets_on_startup: true,
            },
            info,
        )
    }

    #[test]
    fn submit_rejects_position_in_open_state() {
        let exec = executor();
        let mut p = position(PositionStatus::Open, 0, "uuid");
        let err = exec.submit_entry_order(&mut p).unwrap_err();
        match err {
            LiveError::State(msg) => assert!(msg.contains("status is Open")),
            other => panic!("expected State, got {other:?}"),
        }
    }

    #[test]
    fn submit_rejects_position_with_already_assigned_order_id() {
        let exec = executor();
        let mut p = position(PositionStatus::PendingEntry, 12345, "uuid");
        let err = exec.submit_entry_order(&mut p).unwrap_err();
        match err {
            LiveError::State(msg) => assert!(msg.contains("order_id=12345")),
            other => panic!("expected State, got {other:?}"),
        }
    }

    #[test]
    fn submit_rejects_position_without_client_order_id() {
        let exec = executor();
        let mut p = position(PositionStatus::PendingEntry, 0, "");
        let err = exec.submit_entry_order(&mut p).unwrap_err();
        match err {
            LiveError::State(msg) => assert!(msg.contains("client_order_id is empty")),
            other => panic!("expected State, got {other:?}"),
        }
    }

    #[test]
    fn submit_rejects_position_without_entry_order_at_all() {
        let exec = executor();
        let mut p = position(PositionStatus::PendingEntry, 0, "uuid");
        p.entry_order = None;
        let err = exec.submit_entry_order(&mut p).unwrap_err();
        assert!(matches!(err, LiveError::State(_)));
    }

    #[test]
    fn new_client_order_id_is_unique_and_short() {
        let a = new_client_order_id();
        let b = new_client_order_id();
        assert_ne!(a, b);
        assert_eq!(a.len(), 32);
        assert_eq!(b.len(), 32);
        assert!(a.chars().all(|c| c.is_ascii_alphanumeric()));
    }
}
