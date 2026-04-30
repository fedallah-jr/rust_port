//! Position tracker — the lifecycle state machine.
//!
//! Mirrors `live/tracker.PositionTracker`. Owns the call path for every
//! position transition that touches durable state, so callers cannot bypass
//! persistence. The public API is deliberately small:
//!
//!   - `place_entry` — full prepare → persist → submit → persist sequence.
//!   - `check_fills` — periodic poller that drives PendingEntry → Open and
//!     Open → Closed transitions, including TP/SL fills, external closes,
//!     and timeout closes.
//!   - `reconcile_with_exchange` — refreshes the (symbol, side) set of
//!     exchange-level positions that aren't in our state.
//!   - `recover_brackets` — startup sweep that re-places TP/SL for OPEN
//!     positions whose persisted brackets are missing or in a terminal state.
//!   - `load_state` / `save_state_now` — startup / shutdown hooks.
//!   - read-only accessors: `positions`, `open_count`, `open_count_for`,
//!     `has_external_conflict`.
//!
//! There is no `add_position`, `position_mut`, or any other mutation hook —
//! the only way state changes is through the four transition methods above.
//!
//! Persistence: atomic tmp-file + rename. Only `PendingEntry` / `Open`
//! positions are written; `Closed` / `Failed` are kept in memory for the
//! current run but never on disk. Format mirrors Python `tracker._serialize_position`.

use std::collections::HashSet;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;

use chrono::{DateTime, Duration, Utc};
use claude_trader_models::{
    AccountTrade, ExchangeOrder, LiveConfig, LivePosition, OrderSide, OrderStatus, PositionStatus,
    PositionType, Signal,
};
use claude_trader_resolver::compute_pnl;

use crate::auth_client::{symbol_for_api, FuturesApi, TERMINAL_4XX_CODES};
use crate::error::{LiveError, Result};
use crate::executor::OrderExecutor;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const POSITION_AMOUNT_EPSILON: f64 = 1e-8;
const QTY_EPSILON_FLOOR: f64 = 1e-9;
const QTY_EPSILON_REL: f64 = 1e-6;

// ---------------------------------------------------------------------------
// PlacementResult
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct PlacementResult {
    pub position_id: String,
    /// USDT margin the engine should treat as committed. `Placed` consumes
    /// `margin_required`; `Deferred` does too (pessimistic — outcome unknown,
    /// don't over-issue). `Rejected` consumes 0.
    pub margin_consumed: f64,
    pub status: PlacementStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlacementStatus {
    /// POST returned 2xx. Position is `PendingEntry` with `entry_order.order_id`
    /// populated. Persisted.
    Placed,
    /// Definitive rejection (terminal-4xx allow-list). Position marked
    /// `Failed`; will not be persisted on next save.
    Rejected,
    /// 5xx / network outcome unknown. Position remains `PendingEntry` with
    /// `entry_order.order_id == 0` and `client_order_id` populated. Next
    /// `check_fills` queries by `client_order_id` to resolve.
    Deferred,
}

// ---------------------------------------------------------------------------
// PositionTracker
// ---------------------------------------------------------------------------

pub struct PositionTracker {
    client: Arc<dyn FuturesApi>,
    executor: OrderExecutor,
    config: LiveConfig,
    positions: Vec<LivePosition>,
    /// Set of `(api_symbol, position_side)` that the exchange reports open
    /// but we have no tracked position for. Refreshed by
    /// `reconcile_with_exchange`. Counts toward `open_count`; blocks
    /// `place_entry` for the same symbol via `has_external_conflict`.
    external_position_keys: HashSet<(String, String)>,
    dirty: bool,
    state_path: PathBuf,
}

impl PositionTracker {
    /// Production constructor. `client` is shared with the executor (and
    /// elsewhere in the engine).
    pub fn new(client: Arc<dyn FuturesApi>, config: LiveConfig) -> Self {
        let executor = OrderExecutor::new(client.clone(), config.clone());
        Self::with_executor(client, config, executor)
    }

    /// Test/integration constructor: caller supplies a (potentially
    /// pre-populated) `OrderExecutor`. Useful when the test pre-loads the
    /// `ExchangeInfoCache` to avoid HTTP.
    pub fn with_executor(
        client: Arc<dyn FuturesApi>,
        config: LiveConfig,
        executor: OrderExecutor,
    ) -> Self {
        let state_path = default_state_path();
        Self {
            client,
            executor,
            config,
            positions: Vec::new(),
            external_position_keys: HashSet::new(),
            dirty: false,
            state_path,
        }
    }

    /// Override the on-disk state path. Used by tests with temp directories;
    /// production uses `~/.claude_trader/live_state.json`.
    pub fn set_state_path(&mut self, path: PathBuf) {
        self.state_path = path;
    }

    pub fn state_path(&self) -> &std::path::Path {
        &self.state_path
    }

    // -- Read-only accessors -----------------------------------------------

    pub fn positions(&self) -> &[LivePosition] {
        &self.positions
    }

    /// Count of open + pending tracked positions plus distinct external
    /// (symbol, side) pairs we've observed. Engine uses this against
    /// `config.max_concurrent_positions` for the global ceiling.
    pub fn open_count(&self) -> usize {
        let tracked = self
            .positions
            .iter()
            .filter(|p| matches!(p.status, PositionStatus::PendingEntry | PositionStatus::Open))
            .count();
        tracked + self.external_position_keys.len()
    }

    /// Open + pending count attributable to a single strategy slot.
    pub fn open_count_for(&self, strategy_id: &str) -> usize {
        self.positions
            .iter()
            .filter(|p| {
                p.strategy_id == strategy_id
                    && matches!(p.status, PositionStatus::PendingEntry | PositionStatus::Open)
            })
            .count()
    }

    /// True when an exchange-level position exists for the same symbol that
    /// we don't track. Engine uses this to skip placement so we don't pile
    /// onto manual / external exposure.
    pub fn has_external_conflict(&self, signal: &Signal) -> bool {
        let api_symbol = symbol_for_api(&signal.ticker);
        self.external_position_keys
            .iter()
            .any(|(sym, _side)| sym == &api_symbol)
    }

    // -- Place entry: prepare → persist → submit → persist -----------------

    pub fn place_entry(
        &mut self,
        signal: &Signal,
        strategy_id: &str,
        available_balance: Option<f64>,
        position_size_usdt: Option<f64>,
    ) -> Result<PlacementResult> {
        // Phase 1: prepare. May return InsufficientBalance / ZeroQuantity /
        // UnknownSymbol — caller handles and skips.
        let mut prepared = self.executor.prepare_entry_position(
            signal,
            available_balance,
            position_size_usdt,
        )?;
        prepared.position.strategy_id = strategy_id.to_string();
        let position_id = prepared.position.position_id.clone();
        let margin = prepared.margin_required;

        // Phase 2: persist BEFORE the POST.
        self.positions.push(prepared.position);
        self.dirty = true;
        self.save_state_internal();

        // Phase 3: submit.
        let idx = self.positions.len() - 1;
        let outcome = self.executor.submit_entry_order(&mut self.positions[idx]);
        match outcome {
            Ok(()) => {
                self.dirty = true;
                self.save_state_internal();
                Ok(PlacementResult {
                    position_id,
                    margin_consumed: margin,
                    status: PlacementStatus::Placed,
                })
            }
            Err(LiveError::Api { code, msg }) if TERMINAL_4XX_CODES.contains(&code) => {
                eprintln!(
                    "[{}] Entry rejected (terminal 4xx {}): {}",
                    position_id, code, msg
                );
                self.positions[idx].status = PositionStatus::Failed;
                self.dirty = true;
                self.save_state_internal();
                Ok(PlacementResult {
                    position_id,
                    margin_consumed: 0.0,
                    status: PlacementStatus::Rejected,
                })
            }
            Err(other) => {
                eprintln!(
                    "[{}] Entry POST outcome unknown ({}); deferring to check_fills",
                    position_id, other,
                );
                // Position stays PendingEntry on disk with placeholder
                // entry_order (order_id=0, client_order_id set). Next
                // check_fills resolves via get_order_by_client_id.
                Ok(PlacementResult {
                    position_id,
                    margin_consumed: margin,
                    status: PlacementStatus::Deferred,
                })
            }
        }
    }

    // -- Reconciliation -----------------------------------------------------

    /// Refresh `external_position_keys` from the exchange. Returns `true` on
    /// success (including no-change), `false` on API failure (caller defers).
    pub fn reconcile_with_exchange(&mut self) -> bool {
        let raw = match self.client.get_position_info(None) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Failed to reconcile exchange positions: {e}");
                return false;
            }
        };
        let exchange_keys = exchange_position_keys(&raw);
        let tracked_keys = self.tracked_keys();
        let new_external: HashSet<_> = exchange_keys.difference(&tracked_keys).cloned().collect();
        if new_external != self.external_position_keys {
            if !new_external.is_empty() {
                let rendered: Vec<_> = new_external
                    .iter()
                    .map(|(s, side)| format!("{s}:{side}"))
                    .collect();
                eprintln!(
                    "Recovered {} untracked exchange position(s): {}",
                    new_external.len(),
                    rendered.join(", ")
                );
            } else {
                eprintln!("No untracked exchange positions remain.");
            }
            self.external_position_keys = new_external;
        }
        true
    }

    fn tracked_keys(&self) -> HashSet<(String, String)> {
        self.positions
            .iter()
            .filter(|p| matches!(p.status, PositionStatus::PendingEntry | PositionStatus::Open))
            .map(|p| {
                (
                    symbol_for_api(&p.signal.ticker),
                    position_side_str(p.signal.position_type).to_string(),
                )
            })
            .collect()
    }

    // -- check_fills --------------------------------------------------------

    /// Walk every active position and advance its state. The `now_utc`
    /// argument is the engine-injected server-anchored timestamp.
    ///
    /// **Persists state at the end** if any position transitioned. Without
    /// this, a crash after entry-fill + TP/SL placement (which mark dirty
    /// but don't auto-save) would restart from the stale `PENDING_ENTRY`
    /// snapshot and re-run the entry flow — placing duplicate brackets on
    /// the same exchange position.
    pub fn check_fills(&mut self, now_utc: DateTime<Utc>) {
        // One bulk positionRisk fetch when we have any OPEN — saves
        // weight=5 per position. Used by the exit fast-path.
        let all_positions: Option<Vec<serde_json::Value>> =
            if self.positions.iter().any(|p| p.status == PositionStatus::Open) {
                self.client.get_position_info(None).ok()
            } else {
                None
            };

        // Side-share counts so the fast-path can be disabled when multiple
        // tracked positions own the same exchange-level (symbol, side).
        let mut side_counts: std::collections::HashMap<(String, String), usize> =
            std::collections::HashMap::new();
        for p in &self.positions {
            if p.status == PositionStatus::Open {
                let key = (
                    symbol_for_api(&p.signal.ticker),
                    position_side_str(p.signal.position_type).to_string(),
                );
                *side_counts.entry(key).or_insert(0) += 1;
            }
        }

        for i in 0..self.positions.len() {
            match self.positions[i].status {
                PositionStatus::PendingEntry => self.check_entry_fill(i, now_utc),
                PositionStatus::Open => {
                    let key = (
                        symbol_for_api(&self.positions[i].signal.ticker),
                        position_side_str(self.positions[i].signal.position_type).to_string(),
                    );
                    let shares_side = side_counts.get(&key).copied().unwrap_or(0) > 1;
                    let outcome =
                        self.check_exit_fills(i, now_utc, all_positions.as_deref(), shares_side);
                    if matches!(outcome, ExitOutcome::Open) {
                        // Position not closed yet — check whether it's hit max_holding_hours.
                        self.check_timeout(i, now_utc);
                    }
                }
                _ => {}
            }
        }
        // Persist any transitions before returning. Internal helper is
        // dirty-aware so this is a no-op when no position changed.
        self.save_state_internal();
    }

    fn check_entry_fill(&mut self, idx: usize, now_utc: DateTime<Utc>) {
        let entry = match self.positions[idx].entry_order.clone() {
            Some(e) => e,
            None => {
                self.positions[idx].status = PositionStatus::Failed;
                self.dirty = true;
                return;
            }
        };
        let ticker = self.positions[idx].signal.ticker.clone();

        // Two query paths:
        //   - order_id known → get_order
        //   - order_id == 0  → get_order_by_client_id (Deferred placement)
        let updated_result = if entry.order_id == 0 && !entry.client_order_id.is_empty() {
            match self
                .client
                .get_order_by_client_id(&ticker, &entry.client_order_id)
            {
                Ok(Some(o)) => Ok(o),
                Ok(None) => {
                    // -2013: definitively never reached the matching engine.
                    eprintln!(
                        "[{}] Entry never placed (client_id {} returned -2013); marking FAILED",
                        self.positions[idx].position_id, entry.client_order_id,
                    );
                    self.positions[idx].status = PositionStatus::Failed;
                    self.dirty = true;
                    return;
                }
                Err(e) => Err(e),
            }
        } else {
            self.client.get_order(&ticker, entry.order_id)
        };
        let updated = match updated_result {
            Ok(u) => u,
            Err(e) => {
                eprintln!(
                    "[{}] Entry status query failed: {e}; deferring",
                    self.positions[idx].position_id
                );
                return;
            }
        };

        // Stale limit-entry timeout — only meaningful when fill_timeout_seconds > 0.
        if updated.status == OrderStatus::New
            && self.positions[idx].signal.fill_timeout_seconds > 0
        {
            if let Some(created) = updated.created_at {
                let age = (now_utc - created).num_seconds();
                if age >= self.positions[idx].signal.fill_timeout_seconds {
                    // Cancel; -2011 (already-canceled) is swallowed by safe path.
                    self.cancel_safely_normal(&ticker, updated.order_id);
                    self.positions[idx].status = PositionStatus::Failed;
                    self.positions[idx].entry_order = Some(updated);
                    self.dirty = true;
                    eprintln!(
                        "[{}] Entry timed out for {} after {}s — canceled.",
                        self.positions[idx].position_id, ticker, age,
                    );
                    return;
                }
            }
        }

        match updated.status {
            OrderStatus::Filled => {
                let fill_price = if updated.avg_fill_price > 0.0 {
                    updated.avg_fill_price
                } else {
                    updated.price
                };
                let quantity = if updated.filled_qty > 0.0 {
                    updated.filled_qty
                } else {
                    self.positions[idx].quantity
                };
                let opened_at = updated.updated_at.or(updated.created_at).unwrap_or(now_utc);
                self.positions[idx].fill_price = fill_price;
                self.positions[idx].quantity = quantity;
                self.positions[idx].opened_at = Some(opened_at);
                self.positions[idx].entry_order = Some(updated);
                self.dirty = true;

                // Two-phase TP/SL placement so a 5xx/network failure during
                // submit leaves the placeholder (with its clientAlgoId) on
                // disk for startup bracket-recovery to query and adopt.
                //
                // Phase 1: build placeholders. No network here.
                if let Err(e) = self.executor.prepare_brackets(&mut self.positions[idx]) {
                    eprintln!(
                        "[{}] Entry filled but bracket prepare failed: {e}",
                        self.positions[idx].position_id,
                    );
                    self.positions[idx].status = PositionStatus::Open;
                    return;
                }
                self.dirty = true;
                self.save_state_internal(); // persist placeholders BEFORE the POST
                // Phase 2: POST. On error, placeholders persist for recovery.
                match self.executor.submit_brackets(&mut self.positions[idx]) {
                    Ok(()) => {
                        self.positions[idx].status = PositionStatus::Open;
                        eprintln!(
                            "[{}] Entry filled {} @ {} — TP/SL placed.",
                            self.positions[idx].position_id, ticker, fill_price,
                        );
                    }
                    Err(e) => {
                        self.positions[idx].status = PositionStatus::Open;
                        eprintln!(
                            "[{}] Entry filled but bracket submit failed: {e}; \
                             placeholders retained for recovery on next startup",
                            self.positions[idx].position_id,
                        );
                    }
                }
                self.dirty = true;
            }
            OrderStatus::Canceled | OrderStatus::Expired | OrderStatus::Rejected => {
                eprintln!(
                    "[{}] Entry order {} for {}",
                    self.positions[idx].position_id,
                    updated.status,
                    ticker
                );
                self.positions[idx].status = PositionStatus::Failed;
                self.positions[idx].entry_order = Some(updated);
                self.dirty = true;
            }
            _ => {}
        }
    }

    // -- Exit fill detection -----------------------------------------------

    fn check_exit_fills(
        &mut self,
        idx: usize,
        now_utc: DateTime<Utc>,
        all_positions: Option<&[serde_json::Value]>,
        shares_side: bool,
    ) -> ExitOutcome {
        // Fast path: exchange still shows position open and no-one else
        // shares the side → no TP/SL fill possible.
        let mut exchange_open: Option<bool> = None;
        if let Some(raw) = all_positions {
            let still = exchange_position_open_for(&self.positions[idx], raw);
            exchange_open = Some(still);
            if still && !shares_side {
                return ExitOutcome::Open;
            }
        }

        // Slow path: query TP, then SL (skip SL if TP already filled).
        let mut tp_filled = false;
        let mut sl_filled = false;
        let mut query_failed = false;
        let ticker = self.positions[idx].signal.ticker.clone();

        if let Some(tp) = self.positions[idx].tp_order.clone() {
            match self.query_existing_order(&ticker, &tp) {
                Ok(updated) => {
                    tp_filled = updated.status == OrderStatus::Filled;
                    self.positions[idx].tp_order = Some(updated);
                }
                Err(e) => {
                    eprintln!(
                        "[{}] TP query failed: {e}",
                        self.positions[idx].position_id
                    );
                    query_failed = true;
                }
            }
        }
        if !tp_filled {
            if let Some(sl) = self.positions[idx].sl_order.clone() {
                match self.query_existing_order(&ticker, &sl) {
                    Ok(updated) => {
                        sl_filled = updated.status == OrderStatus::Filled;
                        self.positions[idx].sl_order = Some(updated);
                    }
                    Err(e) => {
                        eprintln!(
                            "[{}] SL query failed: {e}",
                            self.positions[idx].position_id
                        );
                        query_failed = true;
                    }
                }
            }
        }

        if tp_filled || sl_filled {
            // Cancel sibling if it's still NEW.
            if tp_filled {
                if let Some(sl) = self.positions[idx].sl_order.clone() {
                    if sl.status == OrderStatus::New {
                        self.cancel_safely_any(&ticker, &sl);
                    }
                }
            }
            if sl_filled {
                if let Some(tp) = self.positions[idx].tp_order.clone() {
                    if tp.status == OrderStatus::New {
                        self.cancel_safely_any(&ticker, &tp);
                    }
                }
            }
            let exit_reason = if tp_filled { ExitReason::Tp } else { ExitReason::Sl };
            let exit_order = if tp_filled {
                self.positions[idx].tp_order.clone()
            } else {
                self.positions[idx].sl_order.clone()
            };
            self.finalize_close(idx, exit_order, exit_reason, now_utc, None, None);
            return ExitOutcome::Closed;
        }

        // No order-side fill. Refresh exchange_open if we haven't already.
        if exchange_open.is_none() {
            if let Some(raw) = all_positions {
                exchange_open = Some(exchange_position_open_for(&self.positions[idx], raw));
            } else {
                // Fall back to a per-symbol query.
                exchange_open = match self
                    .client
                    .get_position_info(Some(&self.positions[idx].signal.ticker))
                {
                    Ok(r) => Some(exchange_position_open_for(&self.positions[idx], &r)),
                    Err(e) => {
                        eprintln!(
                            "[{}] positionRisk query failed: {e}",
                            self.positions[idx].position_id
                        );
                        None
                    }
                };
            }
        }

        match exchange_open {
            Some(true) => ExitOutcome::Open,
            Some(false) => {
                // Position closed externally / by some other means. Try to
                // infer the reason from cached order state, then fall back
                // to walking userTrades.
                let inferred = infer_exchange_exit(&self.positions[idx]);
                match inferred {
                    InferredExit::Tp => {
                        let order = self.positions[idx].tp_order.clone();
                        self.finalize_close(idx, order, ExitReason::Tp, now_utc, None, None);
                    }
                    InferredExit::Sl => {
                        let order = self.positions[idx].sl_order.clone();
                        self.finalize_close(idx, order, ExitReason::Sl, now_utc, None, None);
                    }
                    InferredExit::External => {
                        let recovered =
                            self.resolve_external_exit_from_trades(idx, now_utc);
                        if let Some((reason, price, when)) = recovered {
                            self.finalize_close(
                                idx,
                                None,
                                reason,
                                now_utc,
                                Some(Some(price)),
                                Some(when),
                            );
                        } else {
                            self.finalize_close(
                                idx,
                                None,
                                ExitReason::External,
                                now_utc,
                                Some(None),
                                Some(now_utc),
                            );
                        }
                    }
                }
                ExitOutcome::Closed
            }
            None => {
                if query_failed {
                    ExitOutcome::Unknown
                } else {
                    ExitOutcome::Open
                }
            }
        }
    }

    fn check_timeout(&mut self, idx: usize, now_utc: DateTime<Utc>) {
        let opened = match self.positions[idx].opened_at {
            Some(o) => o,
            None => return,
        };
        let max_h = self.positions[idx].signal.max_holding_hours;
        let deadline = opened + Duration::hours(max_h);
        if now_utc < deadline {
            return;
        }

        // Cancel any still-NEW brackets first.
        let ticker = self.positions[idx].signal.ticker.clone();
        if let Some(tp) = self.positions[idx].tp_order.clone() {
            if tp.status == OrderStatus::New {
                self.cancel_safely_any(&ticker, &tp);
            }
        }
        if let Some(sl) = self.positions[idx].sl_order.clone() {
            if sl.status == OrderStatus::New {
                self.cancel_safely_any(&ticker, &sl);
            }
        }

        let exit_order = match self.executor.close_position_market(&self.positions[idx]) {
            Ok(o) => o,
            Err(e) => {
                eprintln!(
                    "[{}] Timeout close failed: {e}",
                    self.positions[idx].position_id
                );
                return;
            }
        };
        self.finalize_close(
            idx,
            Some(exit_order),
            ExitReason::Timeout,
            now_utc,
            None,
            None,
        );
    }

    fn finalize_close(
        &mut self,
        idx: usize,
        exit_order: Option<ExchangeOrder>,
        reason: ExitReason,
        now_utc: DateTime<Utc>,
        // `Some(Some(price))` overrides cascade with explicit price; `Some(None)`
        // says "no exit price available" (external close with no userTrades match);
        // `None` runs the default cascade.
        resolved_exit_price: Option<Option<f64>>,
        resolved_closed_at: Option<DateTime<Utc>>,
    ) {
        let exit_price = match resolved_exit_price {
            Some(p) => p,
            None => match &exit_order {
                Some(o) if o.avg_fill_price > 0.0 => Some(o.avg_fill_price),
                Some(o) if o.price > 0.0 => Some(o.price),
                Some(o) if matches!(reason, ExitReason::Tp | ExitReason::Sl) && o.stop_price > 0.0 => {
                    Some(o.stop_price)
                }
                _ => {
                    if !matches!(reason, ExitReason::External) {
                        Some(self.positions[idx].fill_price)
                    } else {
                        None
                    }
                }
            },
        };

        if let Some(ep) = exit_price {
            let (net, gross, fees) = compute_pnl(
                self.positions[idx].fill_price,
                ep,
                self.positions[idx].signal.position_type.is_long(),
                self.positions[idx].signal.leverage,
                self.positions[idx].signal.taker_fee_rate,
            );
            self.positions[idx].pnl_pct = Some(net);
            self.positions[idx].gross_pnl_pct = Some(gross);
            self.positions[idx].fee_drag_pct = Some(fees);
        } else {
            self.positions[idx].pnl_pct = None;
            self.positions[idx].gross_pnl_pct = None;
            self.positions[idx].fee_drag_pct = None;
        }

        self.positions[idx].exit_price = exit_price;
        self.positions[idx].status = PositionStatus::Closed;
        let closed_at = resolved_closed_at.unwrap_or_else(|| {
            exit_order
                .as_ref()
                .and_then(|o| o.updated_at.or(o.created_at))
                .unwrap_or(now_utc)
        });
        self.positions[idx].closed_at = Some(closed_at);
        self.dirty = true;

        let exit_text = exit_price
            .map(|p| format!("{p:.4}"))
            .unwrap_or_else(|| "unknown".to_string());
        let pnl_text = self
            .positions[idx]
            .pnl_pct
            .map(|p| format!("{p:+.2}%"))
            .unwrap_or_else(|| "unknown".to_string());
        eprintln!(
            "[{}] {} closed via {:?} @ {} | PnL: {}",
            self.positions[idx].position_id,
            self.positions[idx].signal.ticker,
            reason,
            exit_text,
            pnl_text,
        );
    }

    /// Walk userTrades since `opened_at` to derive the VWAP exit price for a
    /// position the exchange reports gone but for which neither TP nor SL
    /// shows a fill. Returns (reason inferred from matching order_id, vwap, last trade time).
    fn resolve_external_exit_from_trades(
        &self,
        idx: usize,
        now_utc: DateTime<Utc>,
    ) -> Option<(ExitReason, f64, DateTime<Utc>)> {
        let pos = &self.positions[idx];
        let opened_at = pos.opened_at?;
        if pos.quantity <= 0.0 {
            return None;
        }
        let trades = match self.client.get_account_trades(
            &pos.signal.ticker,
            Some(opened_at),
            Some(now_utc),
            None,
            100,
        ) {
            Ok(t) => t,
            Err(e) => {
                eprintln!(
                    "[{}] Failed to query account trades: {e}",
                    pos.position_id
                );
                return None;
            }
        };
        let target_side = position_side_str(pos.signal.position_type);
        let close_side = match pos.signal.position_type {
            PositionType::Long => OrderSide::Sell,
            PositionType::Short => OrderSide::Buy,
        };
        let mut relevant: Vec<AccountTrade> = trades
            .into_iter()
            .filter(|t| t.time >= opened_at && t.time <= now_utc)
            .filter(|t| t.side == close_side)
            .filter(|t| {
                let ps = t.position_side.as_str();
                if ps == "LONG" || ps == "SHORT" {
                    ps == target_side
                } else {
                    true
                }
            })
            .collect();
        if relevant.is_empty() {
            return None;
        }
        relevant.sort_by_key(|t| (t.time, t.trade_id));

        let qty_needed = pos.quantity.max(0.0);
        let qty_eps = QTY_EPSILON_FLOOR.max(qty_needed * QTY_EPSILON_REL);
        let mut total_qty = 0.0;
        let mut total_notional = 0.0;
        let mut exit_time: Option<DateTime<Utc>> = None;
        let mut matched_order_ids: HashSet<i64> = HashSet::new();
        for trade in relevant {
            let remaining = qty_needed - total_qty;
            if remaining <= qty_eps {
                break;
            }
            let used = trade.quantity.min(remaining);
            if used <= 0.0 {
                continue;
            }
            total_qty += used;
            total_notional += trade.price * used;
            exit_time = Some(trade.time);
            if trade.order_id > 0 {
                matched_order_ids.insert(trade.order_id);
            }
        }

        if total_qty <= qty_eps || total_qty + qty_eps < qty_needed {
            return None;
        }
        let exit_time = exit_time?;
        let vwap = total_notional / total_qty;

        let reason = if pos
            .tp_order
            .as_ref()
            .map(|o| o.order_id > 0 && matched_order_ids.contains(&o.order_id))
            .unwrap_or(false)
        {
            ExitReason::Tp
        } else if pos
            .sl_order
            .as_ref()
            .map(|o| o.order_id > 0 && matched_order_ids.contains(&o.order_id))
            .unwrap_or(false)
        {
            ExitReason::Sl
        } else {
            ExitReason::External
        };
        Some((reason, vwap, exit_time))
    }

    fn query_existing_order(
        &self,
        ticker: &str,
        order: &ExchangeOrder,
    ) -> Result<ExchangeOrder> {
        if order.algo_id > 0 {
            self.client.get_algo_order(order.algo_id)
        } else if order.order_id != 0 {
            self.client.get_order(ticker, order.order_id)
        } else if !order.client_order_id.is_empty() {
            // Placeholder shape (order_id=0, algo_id=0, cid set). Routes by
            // order_type:
            //   - StopMarket / TakeProfitMarket → algo endpoint
            //   - everything else (Market, Limit) → normal endpoint
            // -2013 from either endpoint surfaces as Ok(None), which we
            // wrap in a Canceled shim so the slow path's status check
            // classifies the placeholder as "definitively dead — re-place".
            let is_algo = matches!(
                order.order_type,
                claude_trader_models::OrderType::StopMarket
                    | claude_trader_models::OrderType::TakeProfitMarket
            );
            let result = if is_algo {
                self.client.get_algo_order_by_client_id(&order.client_order_id)?
            } else {
                self.client.get_order_by_client_id(ticker, &order.client_order_id)?
            };
            match result {
                Some(o) => Ok(o),
                None => {
                    let mut shim = order.clone();
                    shim.status = OrderStatus::Canceled;
                    Ok(shim)
                }
            }
        } else {
            Err(LiveError::State(format!(
                "query_existing_order: order has no id (order_id=0, algo_id=0, client_id empty)"
            )))
        }
    }

    /// Cancel a normal order, swallowing `-2011 "Unknown order sent"`.
    /// Other errors are logged and swallowed (Python parity).
    fn cancel_safely_normal(&self, ticker: &str, order_id: i64) {
        match self.client.cancel_order(ticker, order_id) {
            Ok(_) => {}
            Err(e) if e.is_cancel_of_unknown() => {}
            Err(e) => eprintln!("cancel {ticker}#{order_id} failed: {e}"),
        }
    }

    /// Cancel either a normal or an algo order, swallowing -2011.
    fn cancel_safely_any(&self, ticker: &str, order: &ExchangeOrder) {
        if order.algo_id > 0 {
            match self.client.cancel_algo_order(order.algo_id) {
                Ok(_) => {}
                Err(e) if e.is_cancel_of_unknown() => {}
                Err(e) => eprintln!("cancel algoId={} failed: {e}", order.algo_id),
            }
        } else if order.order_id != 0 {
            self.cancel_safely_normal(ticker, order.order_id);
        }
    }

    // -- Bracket recovery ---------------------------------------------------

    /// Startup sweep — sweeps every OPEN position regardless of whether its
    /// brackets look healthy on disk, because the loaded state file may be
    /// stale. Adopted Healthy responses capture real `algo_id`s; canceled
    /// or expired brackets get re-placed via the same prepare-save-submit
    /// flow as the entry-fill path.
    pub fn recover_brackets(&mut self) {
        self.recover_brackets_inner(BracketRecoveryScope::All);
    }

    /// Runtime liveness sweep — only OPEN positions with at least one
    /// placeholder bracket (algo_id == 0 with cid set). A placeholder
    /// arises when the entry-fill path's `submit_brackets` failed with
    /// 5xx/network: the position is OPEN, the bracket cid is on disk,
    /// but Binance's response was lost. Without a periodic runtime sweep
    /// the position stays unbracketed until restart or `max_holding_hours`
    /// timeout. Engine drives this on a 5-minute cadence.
    pub fn recover_placeholder_brackets(&mut self) {
        self.recover_brackets_inner(BracketRecoveryScope::PlaceholdersOnly);
    }

    fn recover_brackets_inner(&mut self, scope: BracketRecoveryScope) {
        if !self.config.recover_brackets_on_startup {
            return;
        }

        let mut changed = false;
        for i in 0..self.positions.len() {
            if self.positions[i].status != PositionStatus::Open {
                continue;
            }
            if matches!(scope, BracketRecoveryScope::PlaceholdersOnly)
                && !position_has_placeholder_bracket(&self.positions[i])
            {
                continue;
            }
            if self.positions[i].fill_price <= 0.0 || self.positions[i].quantity <= 0.0 {
                eprintln!(
                    "[{}] skip bracket recovery: fill_price={} quantity={}",
                    self.positions[i].position_id,
                    self.positions[i].fill_price,
                    self.positions[i].quantity,
                );
                continue;
            }
            // Confirm exchange position still open on the matching side.
            let raw = match self
                .client
                .get_position_info(Some(&self.positions[i].signal.ticker))
            {
                Ok(r) => r,
                Err(e) => {
                    eprintln!(
                        "[{}] positionRisk query failed during recovery: {e}; deferring",
                        self.positions[i].position_id
                    );
                    continue;
                }
            };
            if !exchange_position_open_for(&self.positions[i], &raw) {
                continue;
            }
            // Classify each bracket. Unknown blocks recovery for the whole
            // position. Healthy carries the order Binance actually has on
            // file — this is how we *adopt* a placeholder that was saved
            // before a 5xx/network failure but actually reached the matching
            // engine. Without adoption, recovery would re-place an order
            // that already exists, creating a duplicate.
            let tp_state = self.classify_bracket(self.positions[i].tp_order.as_ref());
            let sl_state = self.classify_bracket(self.positions[i].sl_order.as_ref());
            if matches!(tp_state, BracketState::Unknown)
                || matches!(sl_state, BracketState::Unknown)
            {
                eprintln!(
                    "[{}] bracket query failed during recovery; deferring",
                    self.positions[i].position_id
                );
                continue;
            }
            // Adopt any Healthy responses (captures real algo_id from
            // Binance for placeholders we couldn't confirm at submit time).
            if let BracketState::Healthy(ref updated) = tp_state {
                if self.positions[i].tp_order.as_ref().map(|o| &o.client_order_id)
                    != Some(&updated.client_order_id)
                    || self.positions[i].tp_order.as_ref().map(|o| o.algo_id) != Some(updated.algo_id)
                {
                    self.positions[i].tp_order = Some(updated.clone());
                    changed = true;
                }
            }
            if let BracketState::Healthy(ref updated) = sl_state {
                if self.positions[i].sl_order.as_ref().map(|o| &o.client_order_id)
                    != Some(&updated.client_order_id)
                    || self.positions[i].sl_order.as_ref().map(|o| o.algo_id) != Some(updated.algo_id)
                {
                    self.positions[i].sl_order = Some(updated.clone());
                    changed = true;
                }
            }
            // Healthy on both sides → nothing to re-place; loop on.
            if matches!(tp_state, BracketState::Healthy(_))
                && matches!(sl_state, BracketState::Healthy(_))
            {
                continue;
            }
            // Re-place dead sides via the same two-phase pattern as the
            // entry-fill path: prepare placeholders, persist before POST,
            // then submit. Persistence between phases is what makes the
            // submit retry on a fresh boot recoverable.
            let mut prepared_any = false;
            if matches!(tp_state, BracketState::NeedsReplace) {
                if let Err(e) = self.executor.prepare_tp_placeholder(&mut self.positions[i]) {
                    eprintln!(
                        "[{}] TP recovery prepare failed: {e}",
                        self.positions[i].position_id
                    );
                    continue;
                }
                prepared_any = true;
            }
            if matches!(sl_state, BracketState::NeedsReplace) {
                if let Err(e) = self.executor.prepare_sl_placeholder(&mut self.positions[i]) {
                    eprintln!(
                        "[{}] SL recovery prepare failed: {e}",
                        self.positions[i].position_id
                    );
                    continue;
                }
                prepared_any = true;
            }
            if prepared_any {
                self.dirty = true;
                self.save_state_internal();
            }
            if matches!(tp_state, BracketState::NeedsReplace) {
                if let Err(e) = self.executor.submit_tp_only(&mut self.positions[i]) {
                    eprintln!(
                        "[{}] TP recovery submit failed: {e}; placeholder retained",
                        self.positions[i].position_id
                    );
                }
            }
            if matches!(sl_state, BracketState::NeedsReplace) {
                if let Err(e) = self.executor.submit_sl_only(&mut self.positions[i]) {
                    eprintln!(
                        "[{}] SL recovery submit failed: {e}; placeholder retained",
                        self.positions[i].position_id
                    );
                }
            }
            if prepared_any {
                changed = true;
                eprintln!(
                    "[{}] bracket recovery applied: tp={} sl={}",
                    self.positions[i].position_id,
                    bracket_state_label(&tp_state),
                    bracket_state_label(&sl_state),
                );
            }
        }
        if changed {
            self.dirty = true;
            self.save_state_internal();
        }
    }

    fn classify_bracket(&self, order: Option<&ExchangeOrder>) -> BracketState {
        let Some(order) = order else {
            return BracketState::NeedsReplace;
        };
        let ticker = &order.symbol;
        match self.query_existing_order(ticker, order) {
            Ok(updated) => match updated.status {
                OrderStatus::New | OrderStatus::Filled => BracketState::Healthy(updated),
                OrderStatus::Canceled | OrderStatus::Expired | OrderStatus::Rejected => {
                    BracketState::NeedsReplace
                }
            },
            Err(_) => BracketState::Unknown,
        }
    }

    // -- Persistence --------------------------------------------------------

    /// Force a save. Used in shutdown paths that don't trust `dirty`.
    pub fn save_state_now(&mut self) {
        self.dirty = true;
        self.save_state_internal();
    }

    fn save_state_internal(&mut self) {
        if !self.dirty {
            return;
        }
        let active: Vec<&LivePosition> = self
            .positions
            .iter()
            .filter(|p| matches!(p.status, PositionStatus::PendingEntry | PositionStatus::Open))
            .collect();

        if let Some(parent) = self.state_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        // Atomic-write contract: write to a unique tmp file, then rename.
        // Rename is atomic on POSIX so a crash mid-write never leaves a
        // partially-written `live_state.json`.
        let pid = std::process::id();
        let tmp = self.state_path.with_file_name(format!(
            "{}.{}.tmp",
            self.state_path.file_name().unwrap().to_string_lossy(),
            pid,
        ));
        let payload = match serde_json::to_vec_pretty(&active) {
            Ok(b) => b,
            Err(e) => {
                eprintln!("save_state serialize: {e}");
                return;
            }
        };
        if let Err(e) = (|| -> std::io::Result<()> {
            let mut f = fs::File::create(&tmp)?;
            f.write_all(&payload)?;
            f.sync_all()?;
            Ok(())
        })() {
            eprintln!("save_state write {}: {e}", tmp.display());
            let _ = fs::remove_file(&tmp);
            return;
        }
        if let Err(e) = fs::rename(&tmp, &self.state_path) {
            eprintln!("save_state rename: {e}");
            let _ = fs::remove_file(&tmp);
            return;
        }
        self.dirty = false;
    }

    /// Load positions from the state file. Best-effort: missing or malformed
    /// files leave the tracker empty and log to stderr. Accepts:
    ///   - Python-shaped JSON (no `client_order_id`, no `pattern`)
    ///   - Rust-shaped JSON
    pub fn load_state(&mut self) {
        let raw = match fs::read_to_string(&self.state_path) {
            Ok(s) => s,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return,
            Err(e) => {
                eprintln!("load_state read {}: {e}", self.state_path.display());
                return;
            }
        };
        match serde_json::from_str::<Vec<LivePosition>>(&raw) {
            Ok(positions) => {
                for p in &positions {
                    eprintln!(
                        "[{}] Recovered position {} status={}",
                        p.position_id, p.signal.ticker, p.status,
                    );
                }
                self.positions = positions;
            }
            Err(e) => {
                eprintln!("load_state parse: {e}");
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Internal types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExitOutcome {
    Closed,
    Open,
    /// Could not determine state; caller skips and tries again next loop.
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExitReason {
    Tp,
    Sl,
    Timeout,
    External,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InferredExit {
    Tp,
    Sl,
    External,
}

#[derive(Debug, Clone)]
enum BracketState {
    /// Order is alive on Binance. Carries the order Binance currently
    /// holds — caller adopts it onto the position so the persisted state
    /// reflects real `algo_id` / `status`. This is what closes the bracket
    /// idempotency hole: a 5xx during submit leaves a placeholder on disk;
    /// next-startup recovery queries by `clientAlgoId`, finds the order,
    /// and adopts instead of re-placing.
    Healthy(ExchangeOrder),
    NeedsReplace,
    Unknown,
}

fn bracket_state_label(state: &BracketState) -> &'static str {
    match state {
        BracketState::Healthy(_) => "Healthy",
        BracketState::NeedsReplace => "NeedsReplace",
        BracketState::Unknown => "Unknown",
    }
}

#[derive(Debug, Clone, Copy)]
enum BracketRecoveryScope {
    /// Sweep every OPEN position. Used at startup against state loaded from disk.
    All,
    /// Sweep only OPEN positions with at least one placeholder bracket
    /// (algo_id=0, cid set). Used during runtime — Healthy positions
    /// have already been adopted at startup, so re-querying them every
    /// 5 minutes burns API weight for nothing.
    PlaceholdersOnly,
}

fn position_has_placeholder_bracket(pos: &LivePosition) -> bool {
    is_placeholder_bracket(pos.tp_order.as_ref()) || is_placeholder_bracket(pos.sl_order.as_ref())
}

/// A placeholder is a bracket order we built locally before posting to
/// Binance. Distinguished by `algo_id == 0` (Binance assigns non-zero
/// algoIds on success) plus a populated `client_order_id` (we generate
/// it during `prepare_*_placeholder`). A `Some` with both fields zero/
/// empty is *not* a placeholder — that's a never-attempted bracket.
fn is_placeholder_bracket(order: Option<&ExchangeOrder>) -> bool {
    matches!(order, Some(o) if o.algo_id == 0 && !o.client_order_id.is_empty())
}

// ---------------------------------------------------------------------------
// Free helpers
// ---------------------------------------------------------------------------

fn position_side_str(t: PositionType) -> &'static str {
    match t {
        PositionType::Long => "LONG",
        PositionType::Short => "SHORT",
    }
}

fn exchange_position_keys(raw: &[serde_json::Value]) -> HashSet<(String, String)> {
    let mut out = HashSet::new();
    for row in raw {
        let symbol = row.get("symbol").and_then(|s| s.as_str()).unwrap_or("");
        if symbol.is_empty() {
            continue;
        }
        let amt = parse_position_amount(row);
        if amt.abs() <= POSITION_AMOUNT_EPSILON {
            continue;
        }
        let mut side = row
            .get("positionSide")
            .and_then(|s| s.as_str())
            .unwrap_or("BOTH")
            .to_string();
        if side != "LONG" && side != "SHORT" {
            side = if amt > 0.0 { "LONG" } else { "SHORT" }.to_string();
        }
        out.insert((symbol.to_string(), side));
    }
    out
}

fn exchange_position_open_for(pos: &LivePosition, raw: &[serde_json::Value]) -> bool {
    let api_symbol = symbol_for_api(&pos.signal.ticker);
    let target_side = position_side_str(pos.signal.position_type);
    for row in raw {
        if row.get("symbol").and_then(|s| s.as_str()).unwrap_or("") != api_symbol {
            continue;
        }
        let amt = parse_position_amount(row);
        if amt.abs() <= POSITION_AMOUNT_EPSILON {
            continue;
        }
        let side = row
            .get("positionSide")
            .and_then(|s| s.as_str())
            .unwrap_or("BOTH");
        if side == "LONG" || side == "SHORT" {
            if side == target_side {
                return true;
            }
            continue;
        }
        if target_side == "LONG" && amt > POSITION_AMOUNT_EPSILON {
            return true;
        }
        if target_side == "SHORT" && amt < -POSITION_AMOUNT_EPSILON {
            return true;
        }
    }
    false
}

fn parse_position_amount(row: &serde_json::Value) -> f64 {
    row.get("positionAmt")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<f64>().ok())
        .or_else(|| row.get("positionAmt").and_then(|v| v.as_f64()))
        .unwrap_or(0.0)
}

fn infer_exchange_exit(pos: &LivePosition) -> InferredExit {
    if let Some(tp) = pos.tp_order.as_ref() {
        if tp.status == OrderStatus::Filled || tp.avg_fill_price > 0.0 {
            return InferredExit::Tp;
        }
    }
    if let Some(sl) = pos.sl_order.as_ref() {
        if sl.status == OrderStatus::Filled || sl.avg_fill_price > 0.0 {
            return InferredExit::Sl;
        }
    }
    InferredExit::External
}

fn default_state_path() -> PathBuf {
    let home = std::env::var("HOME")
        .ok()
        .or_else(|| std::env::var("USERPROFILE").ok())
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    home.join(".claude_trader").join("live_state.json")
}
