//! `LiveEngine` — the main polling loop.
//!
//! Drives one or more `LiveSignalGenerator` slots through the boundary-aware
//! polling cycle that mirrors `live/engine.LiveEngine`. The single-slot path
//! is the documented entry point (`new_single`); the multi-slot constructor
//! exists for forward-compat but isn't exercised by current strategies.
//!
//! Startup sequence is **strict**:
//!   1. `tracker.load_state()` — recover persisted positions
//!   2. `tracker.reconcile_with_exchange()` — sync external positions
//!   3. `tracker.recover_brackets()` — re-place missing TP/SL
//!   4. `generator.setup(market_client)` — strategy warmup
//!   5. install SIGINT/SIGTERM handler
//!   6. enter the loop
//!
//! Loop iteration order matches Python:
//!   - throttled fill check (5 s when any PENDING_ENTRY, 30 s open-only)
//!   - pre-poll (~10 s before each boundary): reconcile + cache balance
//!   - signal poll (just past each boundary): set_poll_time + poll + execute
//!   - sleep (coarse when idle, intensive near boundaries)
//!
//! The loop polls `shutdown_requested` between phases. SIGINT or SIGTERM
//! arms the flag; the next loop iteration exits. `start()` always runs the
//! final state save + per-slot teardown, regardless of how the loop ended.
//!
//! Tracker-API discipline: this module calls `place_entry`, `check_fills`,
//! `reconcile_with_exchange`, `recover_brackets`, `load_state`, and
//! `save_state_now` only — no low-level mutation hooks. Adding new tracker
//! methods that bypass persistence would defeat Phase D's safety boundary.

use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use chrono::{DateTime, Utc};
use claude_trader_models::{
    floor_boundary, parse_interval_seconds, GeneratorBudget, LiveConfig, Signal,
};
use rand::seq::SliceRandom;
use rand::thread_rng;

use crate::auth_client::{FuturesApi, Sleeper, ThreadSleeper};
use crate::error::{LiveError, Result};
use crate::executor::OrderExecutor;
use crate::market_client::LiveMarketClient;
use crate::shutdown;
use crate::signal_generator::LiveSignalGenerator;
use crate::tracker::{PlacementStatus, PositionTracker};

const PRE_POLL_LEAD_CAP_S: f64 = 10.0;
const INTENSIVE_POLL_LEAD_CAP_S: f64 = 120.0;
const OPEN_FILL_CHECK_S: f64 = 30.0;

/// Cadence at which the engine sweeps placeholder brackets during runtime.
/// A placeholder appears when an entry-fill bracket POST returns 5xx/network
/// — it persists durably on disk but is unbracketed on Binance. Without a
/// runtime sweep, the position stays unbracketed until restart or
/// `max_holding_hours`. Five minutes balances exposure window vs API
/// weight cost.
const BRACKET_RECOVERY_INTERVAL_S: f64 = 300.0;

// ---------------------------------------------------------------------------
// GeneratorSlot
// ---------------------------------------------------------------------------

struct GeneratorSlot {
    generator: Box<dyn LiveSignalGenerator>,
    budget: GeneratorBudget,
    strategy_id: String,
    declared_symbols: HashSet<String>,
    poll_interval: String,
    poll_interval_seconds: f64,
    last_pre_poll_boundary: Option<DateTime<Utc>>,
    last_signal_poll_boundary: Option<DateTime<Utc>>,
}

// ---------------------------------------------------------------------------
// LiveEngine
// ---------------------------------------------------------------------------

pub struct LiveEngine {
    config: LiveConfig,
    client: Arc<dyn FuturesApi>,
    market_client: Arc<dyn LiveMarketClient>,
    tracker: PositionTracker,
    sleeper: Arc<dyn Sleeper>,
    slots: Vec<GeneratorSlot>,
    pre_poll_balance: Option<f64>,
    last_reconcile: Option<DateTime<Utc>>,
    last_fill_check: Option<DateTime<Utc>>,
    last_bracket_recovery: Option<DateTime<Utc>>,
    shutdown_requested: Arc<AtomicBool>,
    install_signal_handlers: bool,
}

impl LiveEngine {
    /// Primary constructor: one strategy with the engine-wide budget from
    /// `config.position_size_usdt` / `config.max_concurrent_positions`.
    pub fn new_single(
        config: LiveConfig,
        client: Arc<dyn FuturesApi>,
        market_client: Arc<dyn LiveMarketClient>,
        generator: Box<dyn LiveSignalGenerator>,
    ) -> Result<Self> {
        let budget = GeneratorBudget {
            position_size_usdt: config.position_size_usdt,
            max_positions: config.max_concurrent_positions,
        };
        Self::new(config, client, market_client, vec![(generator, budget)])
    }

    /// Multi-slot constructor. Validates strategy-id uniqueness and disjoint
    /// symbols across slots. The merge gates explicitly de-scope composite
    /// strategies — multi-slot here means independent strategies, not one
    /// strategy composed of others.
    pub fn new(
        config: LiveConfig,
        client: Arc<dyn FuturesApi>,
        market_client: Arc<dyn LiveMarketClient>,
        generators: Vec<(Box<dyn LiveSignalGenerator>, GeneratorBudget)>,
    ) -> Result<Self> {
        if generators.is_empty() {
            return Err(LiveError::State("LiveEngine requires ≥1 generator".into()));
        }

        let mut seen_ids: HashSet<String> = HashSet::new();
        let mut symbol_owner: std::collections::HashMap<String, String> =
            Default::default();
        let mut slots: Vec<GeneratorSlot> = Vec::with_capacity(generators.len());

        for (gen, budget) in generators {
            let strategy_id = gen.strategy_id().to_string();
            if !seen_ids.insert(strategy_id.clone()) {
                return Err(LiveError::State(format!(
                    "duplicate strategy_id: {strategy_id:?}"
                )));
            }
            let declared: Vec<String> = gen.symbols().to_vec();
            for sym in &declared {
                if let Some(other) = symbol_owner.get(sym) {
                    return Err(LiveError::State(format!(
                        "symbol {sym:?} claimed by both {strategy_id:?} and {other:?}"
                    )));
                }
                symbol_owner.insert(sym.clone(), strategy_id.clone());
            }
            let poll_interval = gen.poll_interval().to_string();
            let poll_seconds = parse_interval_seconds(&poll_interval)
                .map_err(|e| LiveError::State(format!("invalid poll_interval: {e}")))?
                as f64;
            slots.push(GeneratorSlot {
                generator: gen,
                budget,
                strategy_id,
                declared_symbols: declared.into_iter().collect(),
                poll_interval,
                poll_interval_seconds: poll_seconds,
                last_pre_poll_boundary: None,
                last_signal_poll_boundary: None,
            });
        }

        let executor = OrderExecutor::new(client.clone(), config.clone());
        let tracker = PositionTracker::with_executor(client.clone(), config.clone(), executor);
        Ok(Self {
            config,
            client,
            market_client,
            tracker,
            sleeper: Arc::new(ThreadSleeper),
            slots,
            pre_poll_balance: None,
            last_reconcile: None,
            last_fill_check: None,
            last_bracket_recovery: None,
            shutdown_requested: Arc::new(AtomicBool::new(false)),
            install_signal_handlers: true,
        })
    }

    /// Override the sleeper. Tests pass a `MockSleeper` that records but
    /// never blocks.
    pub fn set_sleeper(&mut self, sleeper: Arc<dyn Sleeper>) {
        self.sleeper = sleeper;
    }

    /// Override the on-disk state path for the underlying tracker. Used by
    /// tests with isolated tmp directories.
    pub fn set_state_path(&mut self, path: std::path::PathBuf) {
        self.tracker.set_state_path(path);
    }

    /// Disable SIGINT/SIGTERM handler installation. Tests must call this so
    /// `cargo test` doesn't hijack the process's signal handlers.
    pub fn disable_signal_handlers(&mut self) {
        self.install_signal_handlers = false;
    }

    /// Read-only access to the tracker for tests / inspection. Note: this
    /// returns `&PositionTracker`, NOT `&mut`. Mutations only happen through
    /// `place_entry`, `check_fills`, etc., all called by the engine itself.
    pub fn tracker(&self) -> &PositionTracker {
        &self.tracker
    }

    /// Clone the shutdown flag. Tests use this to stop the loop deterministically.
    pub fn shutdown_flag(&self) -> Arc<AtomicBool> {
        self.shutdown_requested.clone()
    }

    // -- Lifecycle ----------------------------------------------------------

    /// Run the strict startup sequence and enter the loop. Always runs the
    /// final state save + per-slot teardown, even on error paths.
    pub fn start(&mut self) -> Result<()> {
        // Step 1: load persisted state.
        self.tracker.load_state();
        // Step 2: sync exchange-side positions.
        self.tracker.reconcile_with_exchange();
        // Step 3: re-place missing brackets (gated by config flag).
        self.tracker.recover_brackets();

        // Step 4: per-slot strategy setup. Step 5: install signal handlers.
        // Both are inside a closure so failure short-circuits to the
        // unconditional shutdown step below.
        let setup_result = (|| -> Result<()> {
            for slot in self.slots.iter_mut() {
                // Setup errors of either variant are treated as fatal —
                // refusing to trade without a successful warmup matches the
                // Python invariant.
                slot.generator
                    .setup(self.market_client.clone())
                    .map_err(|e| LiveError::Fatal(e.message().to_string()))?;
            }
            if self.install_signal_handlers {
                shutdown::install(&self.shutdown_requested)?;
            }
            Ok(())
        })();

        // Step 6: enter the loop only if setup succeeded.
        let loop_result = if setup_result.is_ok() {
            self.run_loop()
        } else {
            setup_result
        };

        // Step 7: unconditional final save + teardown.
        self.tracker.save_state_now();
        for slot in self.slots.iter_mut() {
            slot.generator.teardown();
        }
        loop_result
    }

    fn run_loop(&mut self) -> Result<()> {
        let check_interval = self.config.order_check_interval_seconds;
        while !self.shutdown_requested.load(Ordering::Relaxed) {
            let now = self.client.server_now();
            self.tick(now)?;
            // Recheck before sleeping — a SIGINT during tick should exit
            // without first sleeping the full interval.
            if self.shutdown_requested.load(Ordering::Relaxed) {
                break;
            }
            let sleep_now = self.client.server_now();
            let sleep_s = self.sleep_interval(sleep_now, check_interval);
            self.sleeper.sleep(Duration::from_secs_f64(sleep_s));
        }
        Ok(())
    }

    /// Single iteration of the polling cycle. Public so tests can drive the
    /// engine deterministically without spawning a real loop or sleeping.
    pub fn tick(&mut self, now: DateTime<Utc>) -> Result<()> {
        let check_interval = self.config.order_check_interval_seconds;

        // Phase 1: throttled fill check.
        if self.should_check_fills(now, check_interval) {
            self.tracker.check_fills(now);
            self.last_fill_check = Some(now);
        }

        // Phase 1b: periodic placeholder-bracket recovery. Only fires when
        // at least one OPEN position has a placeholder bracket — that is,
        // when an entry-fill bracket POST returned 5xx/network and the
        // submit was deferred. Same query/adopt/re-place flow as startup
        // recovery; the cadence keeps API weight low (5 min default).
        if self.should_recover_placeholders(now) {
            self.tracker.recover_placeholder_brackets();
            self.last_bracket_recovery = Some(now);
        }

        // Phase 2: pre-poll (reconcile + balance) when within lead window.
        let pre_now = self.client.server_now();
        if self.should_pre_poll(pre_now, check_interval) {
            self.do_pre_poll(pre_now);
        }

        // Phase 3: signal poll for every slot whose boundary just passed.
        // All due slots see the same `sig_now` from the same server clock —
        // breadth/regime calculations across slots stay consistent.
        let sig_now = self.client.server_now();
        let due = self.due_slot_indices(sig_now, check_interval);
        if !due.is_empty() {
            self.do_signal_poll(sig_now, &due)?;
        }
        Ok(())
    }

    // -- Throttling ---------------------------------------------------------

    /// Returns true when the periodic placeholder-bracket sweep should run.
    /// Conditions: at least one OPEN position has a placeholder bracket
    /// (algo_id==0 with cid set), AND either we've never run the sweep
    /// before (first sighting after startup recovery resolved everything)
    /// or `BRACKET_RECOVERY_INTERVAL_S` has elapsed since the last sweep.
    fn should_recover_placeholders(&self, now: DateTime<Utc>) -> bool {
        let any_placeholder = self.tracker.positions().iter().any(|p| {
            if !matches!(p.status, claude_trader_models::PositionStatus::Open) {
                return false;
            }
            engine_is_placeholder(p.tp_order.as_ref())
                || engine_is_placeholder(p.sl_order.as_ref())
        });
        if !any_placeholder {
            return false;
        }
        match self.last_bracket_recovery {
            None => true,
            Some(t) => {
                (now - t).num_milliseconds() as f64 >= BRACKET_RECOVERY_INTERVAL_S * 1000.0
            }
        }
    }

    fn should_check_fills(&self, now: DateTime<Utc>, check_interval: f64) -> bool {
        let has_any = self.tracker.positions().iter().any(|p| {
            matches!(
                p.status,
                claude_trader_models::PositionStatus::PendingEntry
                    | claude_trader_models::PositionStatus::Open
            )
        });
        if !has_any {
            return false;
        }
        let last = match self.last_fill_check {
            Some(t) => t,
            // First check: always run when there's anything to check.
            None => return true,
        };
        let has_pending = self.tracker.positions().iter().any(|p| {
            matches!(p.status, claude_trader_models::PositionStatus::PendingEntry)
        });
        let min_interval = if has_pending {
            check_interval
        } else {
            OPEN_FILL_CHECK_S
        };
        (now - last).num_milliseconds() as f64 >= min_interval * 1000.0
    }

    fn pre_poll_lead_seconds(&self, check_interval: f64) -> f64 {
        let min_poll =
            self.slots.iter().map(|s| s.poll_interval_seconds).fold(f64::INFINITY, f64::min);
        PRE_POLL_LEAD_CAP_S.min(check_interval.max(min_poll / 6.0))
    }

    fn intensive_poll_lead_seconds(&self, check_interval: f64) -> f64 {
        let min_poll =
            self.slots.iter().map(|s| s.poll_interval_seconds).fold(f64::INFINITY, f64::min);
        INTENSIVE_POLL_LEAD_CAP_S.min(check_interval.max(min_poll / 6.0))
    }

    fn should_pre_poll(&self, now: DateTime<Utc>, check_interval: f64) -> bool {
        let lead = self.pre_poll_lead_seconds(check_interval);
        for slot in &self.slots {
            let next_b = match next_boundary_for(slot, now) {
                Some(b) => b,
                None => continue,
            };
            let lead_window_start = next_b - chrono::Duration::milliseconds((lead * 1000.0) as i64);
            if now >= lead_window_start && now < next_b && slot.last_pre_poll_boundary != Some(next_b) {
                return true;
            }
        }
        false
    }

    fn due_slot_indices(&self, now: DateTime<Utc>, check_interval: f64) -> Vec<usize> {
        let mut out = Vec::new();
        for (i, slot) in self.slots.iter().enumerate() {
            let boundary = match floor_boundary(now, &slot.poll_interval).ok() {
                Some(b) => b,
                None => continue,
            };
            let elapsed_s = (now - boundary).num_milliseconds() as f64 / 1000.0;
            if elapsed_s >= 0.0
                && elapsed_s <= 1.0 + check_interval
                && slot.last_signal_poll_boundary != Some(boundary)
            {
                out.push(i);
            }
        }
        out
    }

    fn earliest_next_boundary(&self, now: DateTime<Utc>) -> Option<DateTime<Utc>> {
        self.slots
            .iter()
            .filter_map(|s| next_boundary_for(s, now))
            .min()
    }

    fn fill_check_interval(&self, check_interval: f64) -> f64 {
        let has_pending = self.tracker.positions().iter().any(|p| {
            matches!(p.status, claude_trader_models::PositionStatus::PendingEntry)
        });
        if has_pending {
            check_interval
        } else {
            OPEN_FILL_CHECK_S
        }
    }

    fn has_local_active_positions(&self) -> bool {
        self.tracker.positions().iter().any(|p| {
            matches!(
                p.status,
                claude_trader_models::PositionStatus::PendingEntry
                    | claude_trader_models::PositionStatus::Open
            )
        })
    }

    /// Smart sleep: coarse when idle, intensive near boundaries. Mirrors
    /// `LiveEngine._sleep_interval_seconds` formulas exactly.
    pub fn sleep_interval(&self, now: DateTime<Utc>, check_interval: f64) -> f64 {
        let next_b = match self.earliest_next_boundary(now) {
            Some(b) => b,
            None => return check_interval,
        };
        let seconds_until_boundary =
            ((next_b - now).num_milliseconds() as f64 / 1000.0).max(0.0);

        if self.has_local_active_positions() {
            return self
                .fill_check_interval(check_interval)
                .min(seconds_until_boundary);
        }
        let lead = self.intensive_poll_lead_seconds(check_interval);
        if seconds_until_boundary <= lead {
            return check_interval.min(seconds_until_boundary);
        }
        let coarse_sleep = seconds_until_boundary - lead;
        check_interval.max(coarse_sleep)
    }

    // -- Pre-poll -----------------------------------------------------------

    fn do_pre_poll(&mut self, now: DateTime<Utc>) {
        self.tracker.reconcile_with_exchange();
        self.last_reconcile = Some(now);

        // Mark each slot whose boundary the lead window covers, even if the
        // slot has no pending poll — Python parity, prevents firing the
        // pre-poll twice for the same slot/boundary pair.
        let lead = self.pre_poll_lead_seconds(self.config.order_check_interval_seconds);
        for slot in self.slots.iter_mut() {
            if let Some(next_b) = next_boundary_for(slot, now) {
                let lead_window_start =
                    next_b - chrono::Duration::milliseconds((lead * 1000.0) as i64);
                if now >= lead_window_start && now < next_b {
                    slot.last_pre_poll_boundary = Some(next_b);
                }
            }
        }

        match self.client.get_available_balance() {
            Ok(bal) => {
                self.pre_poll_balance = Some(bal);
                eprintln!(
                    "Pre-poll {} | balance={:.2} USDT | open={}/{}",
                    now.format("%H:%M:%S"),
                    bal,
                    self.tracker.open_count(),
                    self.config.max_concurrent_positions,
                );
            }
            Err(e) => {
                self.pre_poll_balance = None;
                eprintln!("Pre-poll balance fetch failed: {e}");
            }
        }
    }

    // -- Signal poll --------------------------------------------------------

    fn do_signal_poll(&mut self, now: DateTime<Utc>, due: &[usize]) -> Result<()> {
        // Reconcile if not freshly reconciled in pre-poll.
        let needs_reconcile = self
            .last_reconcile
            .map(|t| (now - t).num_seconds() > 5)
            .unwrap_or(true);
        if needs_reconcile {
            self.tracker.reconcile_with_exchange();
            self.last_reconcile = Some(now);
        }

        // Capture balance once. Use cached pre-poll value when available;
        // otherwise fetch. On fetch failure, mark boundaries (so we don't
        // retry next tick) and skip the poll.
        let balance = match self.pre_poll_balance {
            Some(b) => b,
            None => match self.client.get_available_balance() {
                Ok(b) => b,
                Err(e) => {
                    eprintln!("Signal poll: balance fetch failed: {e}; skipping");
                    self.mark_signal_poll_boundaries(now, due);
                    return Ok(());
                }
            },
        };

        // Drive each due generator: set_poll_time first (all slots see the
        // same now), then poll. Per-variant handling:
        //   - SignalError::Fatal       → propagate up; LiveEngine::start saves state
        //                                 and tears down, then surfaces as LiveError::Fatal.
        //   - SignalError::Recoverable → log and treat as empty signals; engine continues.
        //                                 Mirrors Python's `except Exception: ... slot_signals[id] = []`.
        let mut all_signals: Vec<(usize, Vec<Signal>)> = Vec::with_capacity(due.len());
        for &i in due {
            let slot = &mut self.slots[i];
            slot.generator.set_poll_time(now);
            match slot.generator.poll() {
                Ok(sigs) => all_signals.push((i, sigs)),
                Err(crate::signal_generator::SignalError::Fatal(msg)) => {
                    return Err(LiveError::Fatal(msg));
                }
                Err(crate::signal_generator::SignalError::Recoverable(msg)) => {
                    eprintln!(
                        "[{}] recoverable poll error: {}; treating as no-signals",
                        slot.strategy_id, msg,
                    );
                    all_signals.push((i, Vec::new()));
                }
            }
        }
        // Boundaries marked here so a transient error in execute_slot_signals
        // doesn't cause another poll within the same boundary window.
        self.mark_signal_poll_boundaries(now, due);

        // Capital allocation cascade.
        self.execute_slot_signals(due, all_signals, balance);
        Ok(())
    }

    fn mark_signal_poll_boundaries(&mut self, now: DateTime<Utc>, due: &[usize]) {
        for &i in due {
            if let Ok(b) = floor_boundary(now, &self.slots[i].poll_interval) {
                self.slots[i].last_signal_poll_boundary = Some(b);
            }
        }
    }

    fn execute_slot_signals(
        &mut self,
        due: &[usize],
        slot_signals: Vec<(usize, Vec<Signal>)>,
        initial_balance: f64,
    ) {
        let global_max = self.config.max_concurrent_positions;
        let global_open = self.tracker.open_count();
        let mut global_remaining = global_max.saturating_sub(global_open);
        let mut remaining_balance = initial_balance;

        for (i, raw_signals) in slot_signals {
            let strategy_id = self.slots[i].strategy_id.clone();
            let position_size = self.slots[i].budget.position_size_usdt;
            let max_pos = self.slots[i].budget.max_positions;
            let leverage = self.slots[i].generator.leverage();

            // Symbol validation against declared_symbols.
            let valid: Vec<Signal> = raw_signals
                .into_iter()
                .filter(|sig| {
                    if !self.slots[i].declared_symbols.contains(&sig.ticker) {
                        eprintln!(
                            "WARN [{}] dropping signal for undeclared symbol {:?}",
                            strategy_id, sig.ticker,
                        );
                        return false;
                    }
                    true
                })
                .collect();
            if valid.is_empty() {
                continue;
            }
            // External-conflict filter — drop signals where Binance reports a
            // different position on the same symbol that we don't track.
            let mut executable: Vec<Signal> = Vec::with_capacity(valid.len());
            for sig in valid {
                if self.tracker.has_external_conflict(&sig) {
                    eprintln!(
                        "WARN [{}] skipping {:?}: existing untracked exchange exposure",
                        strategy_id, sig.ticker,
                    );
                    continue;
                }
                executable.push(sig);
            }
            if executable.is_empty() {
                continue;
            }

            let slot_open = self.tracker.open_count_for(&strategy_id);
            let slot_remaining = max_pos.saturating_sub(slot_open);
            let affordable = affordable_entries(remaining_balance, position_size, leverage);
            let max_entries = slot_remaining.min(global_remaining).min(affordable);
            if max_entries == 0 {
                continue;
            }

            // Random shuffle so we don't systematically prefer the first
            // listed symbol when there are more signals than slots.
            executable.shuffle(&mut thread_rng());

            for sig in executable.into_iter().take(max_entries) {
                let outcome = self.tracker.place_entry(
                    &sig,
                    &strategy_id,
                    Some(remaining_balance),
                    Some(position_size),
                );
                match outcome {
                    Ok(result) => {
                        if result.status != PlacementStatus::Rejected {
                            remaining_balance =
                                (remaining_balance - result.margin_consumed).max(0.0);
                            global_remaining = global_remaining.saturating_sub(1);
                        }
                    }
                    Err(e) => {
                        eprintln!(
                            "[{}] place_entry failed for {}: {e}",
                            strategy_id, sig.ticker
                        );
                    }
                }
            }
        }
        let _ = due;
    }
}

// ---------------------------------------------------------------------------
// Free helpers
// ---------------------------------------------------------------------------

fn next_boundary_for(slot: &GeneratorSlot, now: DateTime<Utc>) -> Option<DateTime<Utc>> {
    floor_boundary(now, &slot.poll_interval)
        .ok()
        .map(|b| b + chrono::Duration::seconds(slot.poll_interval_seconds as i64))
}

/// A bracket order is a "placeholder" when it was constructed locally
/// (cid generated, status NEW, stop_price computed) but never confirmed
/// by Binance — `algo_id==0` with a populated `client_order_id`. This
/// arises when an entry-fill TP/SL POST returned 5xx/network: the
/// position is durably persisted with the cid, but the matching engine's
/// state is unknown until we query.
fn engine_is_placeholder(order: Option<&claude_trader_models::ExchangeOrder>) -> bool {
    matches!(order, Some(o) if o.algo_id == 0 && !o.client_order_id.is_empty())
}

fn affordable_entries(balance: f64, position_size_usdt: f64, leverage: f64) -> usize {
    if position_size_usdt <= 0.0 {
        return 0;
    }
    let buying_power = balance.max(0.0) * leverage.max(1.0);
    (buying_power / position_size_usdt) as usize
}

// ---------------------------------------------------------------------------
// Unit tests — focus on scheduling / validation logic; full lifecycle is in
// `tests/engine_integration.rs`. No real sleeps anywhere.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signal_generator::FatalSignalError;
    use chrono::TimeZone;
    use claude_trader_models::{MarketType, PositionType};

    fn ts(rfc: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(rfc)
            .unwrap()
            .with_timezone(&Utc)
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

    /// Minimal FuturesApi — server_now is the only method exercised by these
    /// scheduling tests. Other methods would unreachable!() so a logic bug
    /// that called them surfaces loudly.
    struct ClockApi(DateTime<Utc>);
    impl FuturesApi for ClockApi {
        fn server_now(&self) -> DateTime<Utc> {
            self.0
        }
        fn place_market_order(
            &self,
            _: &str,
            _: claude_trader_models::OrderSide,
            _: f64,
            _: &str,
            _: &str,
        ) -> Result<claude_trader_models::ExchangeOrder> {
            unreachable!()
        }
        fn place_limit_order(
            &self,
            _: &str,
            _: claude_trader_models::OrderSide,
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
            _: claude_trader_models::OrderSide,
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
            _: claude_trader_models::OrderSide,
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
        fn get_algo_order(&self, _: i64) -> Result<claude_trader_models::ExchangeOrder> {
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
            Ok(vec![])
        }
        fn get_account_trades(
            &self,
            _: &str,
            _: Option<DateTime<Utc>>,
            _: Option<DateTime<Utc>>,
            _: Option<i64>,
            _: usize,
        ) -> Result<Vec<claude_trader_models::AccountTrade>> {
            unreachable!()
        }
        fn get_account_info(&self) -> Result<serde_json::Value> {
            unreachable!()
        }
        fn get_available_balance(&self) -> Result<f64> {
            Ok(1000.0)
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

    struct DummyGen {
        id: String,
        symbols: Vec<String>,
        analysis: String,
    }
    impl LiveSignalGenerator for DummyGen {
        fn strategy_id(&self) -> &str {
            &self.id
        }
        fn symbols(&self) -> &[String] {
            &self.symbols
        }
        fn analysis_interval(&self) -> &str {
            &self.analysis
        }
        fn poll(&mut self) -> Result<Vec<Signal>, FatalSignalError> {
            Ok(vec![])
        }
    }

    fn dummy(id: &str, symbols: &[&str]) -> Box<DummyGen> {
        Box::new(DummyGen {
            id: id.into(),
            symbols: symbols.iter().map(|s| s.to_string()).collect(),
            analysis: "1h".into(),
        })
    }

    fn engine(generators: Vec<Box<dyn LiveSignalGenerator>>) -> LiveEngine {
        let with_budgets: Vec<_> = generators
            .into_iter()
            .map(|g| {
                let budget = GeneratorBudget {
                    position_size_usdt: 100.0,
                    max_positions: 3,
                };
                (g, budget)
            })
            .collect();
        let mut e = LiveEngine::new(
            cfg(),
            Arc::new(ClockApi(Utc::now())),
            Arc::new(crate::market_client::NullMarketClient),
            with_budgets,
        )
        .unwrap();
        e.disable_signal_handlers();
        e
    }

    #[test]
    fn new_rejects_duplicate_strategy_ids() {
        let result = LiveEngine::new(
            cfg(),
            Arc::new(ClockApi(Utc::now())),
            Arc::new(crate::market_client::NullMarketClient),
            vec![
                (dummy("alpha", &["BTCUSDT"]), GeneratorBudget::default()),
                (dummy("alpha", &["ETHUSDT"]), GeneratorBudget::default()),
            ],
        );
        match result {
            Err(LiveError::State(msg)) => assert!(msg.contains("duplicate strategy_id")),
            Err(other) => panic!("unexpected error: {other:?}"),
            Ok(_) => panic!("expected error, got Ok"),
        }
    }

    #[test]
    fn new_rejects_overlapping_symbols() {
        let result = LiveEngine::new(
            cfg(),
            Arc::new(ClockApi(Utc::now())),
            Arc::new(crate::market_client::NullMarketClient),
            vec![
                (dummy("alpha", &["BTCUSDT", "ETHUSDT"]), GeneratorBudget::default()),
                (dummy("beta", &["ETHUSDT", "SOLUSDT"]), GeneratorBudget::default()),
            ],
        );
        match result {
            Err(LiveError::State(msg)) => {
                assert!(msg.contains("ETHUSDT"));
                assert!(msg.contains("alpha"));
                assert!(msg.contains("beta"));
            }
            Err(other) => panic!("unexpected error: {other:?}"),
            Ok(_) => panic!("expected error, got Ok"),
        }
    }

    #[test]
    fn new_single_creates_one_slot_with_engine_budget() {
        let mut config = cfg();
        config.position_size_usdt = 250.0;
        config.max_concurrent_positions = 5;
        let e = LiveEngine::new_single(
            config,
            Arc::new(ClockApi(Utc::now())),
            Arc::new(crate::market_client::NullMarketClient),
            dummy("alpha", &["BTCUSDT"]),
        )
        .unwrap();
        assert_eq!(e.slots.len(), 1);
        assert_eq!(e.slots[0].strategy_id, "alpha");
        assert_eq!(e.slots[0].budget.position_size_usdt, 250.0);
        assert_eq!(e.slots[0].budget.max_positions, 5);
    }

    #[test]
    fn new_rejects_empty_generator_list() {
        let result = LiveEngine::new(
            cfg(),
            Arc::new(ClockApi(Utc::now())),
            Arc::new(crate::market_client::NullMarketClient),
            vec![],
        );
        assert!(matches!(result, Err(LiveError::State(_))));
    }

    #[test]
    fn pre_poll_fires_once_per_boundary() {
        let now_just_before_top = ts("2026-04-30T12:59:55Z"); // 5 s before next 1h boundary
        let next_b = ts("2026-04-30T13:00:00Z");
        let mut e = engine(vec![dummy("alpha", &["BTCUSDT"])]);

        // First call inside lead window → fires.
        assert!(e.should_pre_poll(now_just_before_top, 5.0));
        // Mark as fired (simulate do_pre_poll having run).
        e.slots[0].last_pre_poll_boundary = Some(next_b);
        // Second call same boundary → does NOT fire.
        assert!(!e.should_pre_poll(now_just_before_top, 5.0));

        // Outside the window → does not fire.
        let earlier = ts("2026-04-30T12:59:30Z");
        // Reset so we can re-test the lead-window guard cleanly.
        e.slots[0].last_pre_poll_boundary = None;
        assert!(!e.should_pre_poll(earlier, 5.0));
    }

    #[test]
    fn due_slot_indices_returns_slot_just_past_boundary() {
        let mut e = engine(vec![dummy("alpha", &["BTCUSDT"])]);
        // 0.5 s past the top of the hour → due.
        let now_just_after = ts("2026-04-30T13:00:00.5Z").with_timezone(&Utc);
        let now_just_after = Utc.timestamp_millis_opt(now_just_after.timestamp_millis()).single().unwrap();
        let due = e.due_slot_indices(now_just_after, 5.0);
        assert_eq!(due, vec![0]);
        // Mark as fired and re-check — same boundary should not be due.
        e.slots[0].last_signal_poll_boundary =
            Some(floor_boundary(now_just_after, "1h").unwrap());
        let due = e.due_slot_indices(now_just_after, 5.0);
        assert!(due.is_empty());
    }

    #[test]
    fn affordable_entries_math_matches_python() {
        // 1000 USDT @ 5x leverage, 100 USDT per position → 50 entries affordable.
        assert_eq!(affordable_entries(1000.0, 100.0, 5.0), 50);
        // 50 USDT @ 1x → 0.
        assert_eq!(affordable_entries(50.0, 100.0, 1.0), 0);
        // Negative balance clamps to 0.
        assert_eq!(affordable_entries(-10.0, 100.0, 1.0), 0);
        // Zero/negative leverage falls back to 1x.
        assert_eq!(affordable_entries(100.0, 100.0, 0.0), 1);
    }

    #[test]
    fn sleep_interval_is_coarse_when_idle_and_far_from_boundary() {
        let e = engine(vec![dummy("alpha", &["BTCUSDT"])]);
        // 30 minutes before next 1h boundary, no positions open.
        let now = ts("2026-04-30T12:30:00Z");
        let sleep = e.sleep_interval(now, 5.0);
        // Coarse-sleep formula: max(check_interval, time_to_boundary - lead).
        // lead = min(120, max(5, 3600/6)) = min(120, 600) = 120
        // time_to_boundary = 30*60 = 1800 → coarse = 1800 - 120 = 1680
        assert!((sleep - 1680.0).abs() < 1.0);
    }

    #[test]
    fn sleep_interval_tightens_inside_intensive_window() {
        let e = engine(vec![dummy("alpha", &["BTCUSDT"])]);
        // 60 s before boundary, no positions. lead = 120 s, so we're inside.
        let now = ts("2026-04-30T12:59:00Z");
        let sleep = e.sleep_interval(now, 5.0);
        // min(check_interval=5, seconds_until_boundary=60) = 5
        assert!((sleep - 5.0).abs() < 1.0);
    }

    #[test]
    fn signal_examples_with_no_positions_no_pending_skips_fill_check() {
        let e = engine(vec![dummy("alpha", &["BTCUSDT"])]);
        let now = Utc::now();
        // No positions → never run.
        assert!(!e.should_check_fills(now, 5.0));
    }

    /// Sanity check that the Signal/PositionType import compiles — guards
    /// against the unused-import warning treadmill if signature changes.
    #[allow(dead_code)]
    fn _signal_compile_check() -> Signal {
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
}
