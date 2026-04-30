//! `LiveSignalGenerator` — the only strategy-facing contract for live trading.
//!
//! Mirrors `live/signal_generator.SignalGenerator` (the live-side hooks, not
//! the backtest-side `generate_backtest_signals`, which lives separately in
//! `claude-trader-research-runtime::ResearchStrategy`). A strategy implements
//! this trait, hands the boxed object to `LiveEngine::new_single`, and the
//! engine drives:
//!
//!   1. `setup(market_client)` once before the loop — used to warm up
//!      indicator state. Returning `Err(FatalSignalError)` aborts startup
//!      with the engine's normal shutdown path (state save + teardown).
//!   2. `set_poll_time(now)` immediately before every `poll()`. Generators
//!      MUST use this instead of `Utc::now()` so all slots polling on the
//!      same boundary share a clock.
//!   3. `poll()` returns the signals to consider this poll. Empty Vec is the
//!      no-trade case. `Err(FatalSignalError)` halts the engine.
//!   4. `teardown()` on shutdown.
//!
//! The trait is `Send` so the engine can hold `Box<dyn LiveSignalGenerator>`
//! and so future multi-thread polling stays an option. It is *not* `Sync` —
//! a generator is owned by exactly one slot and accessed from one thread at
//! a time.

use std::sync::Arc;

use chrono::{DateTime, Utc};
use claude_trader_models::Signal;

use crate::market_client::LiveMarketClient;

/// Errors a strategy can return from `setup` / `poll`.
///
/// - `Fatal` halts the engine via the normal save+teardown path. Use for
///   "this strategy cannot run" conditions: insufficient warmup history,
///   misconfiguration, missing data.
/// - `Recoverable` is logged and treated as "no signals this poll" — engine
///   keeps running. Use for transient data-fetch failures: a single symbol's
///   klines failed, breadth check missing one input, etc.
///
/// Setup errors are always treated as fatal regardless of variant — the
/// engine refuses to start trading without a successful warmup.
#[derive(Debug, thiserror::Error)]
pub enum SignalError {
    #[error("fatal signal-generator error: {0}")]
    Fatal(String),
    #[error("recoverable signal-generator error: {0}")]
    Recoverable(String),
}

impl SignalError {
    pub fn fatal(msg: impl Into<String>) -> Self {
        Self::Fatal(msg.into())
    }
    pub fn recoverable(msg: impl Into<String>) -> Self {
        Self::Recoverable(msg.into())
    }
    /// Backward-compat: `FatalSignalError::new("...")` defaulted to fatal.
    pub fn new(msg: impl Into<String>) -> Self {
        Self::Fatal(msg.into())
    }
    pub fn message(&self) -> &str {
        match self {
            Self::Fatal(m) | Self::Recoverable(m) => m,
        }
    }
    pub fn is_fatal(&self) -> bool {
        matches!(self, Self::Fatal(_))
    }
}

/// Backward-compatible alias. Existing strategies that constructed
/// `FatalSignalError("…")` still compile via `From`.
pub type FatalSignalError = SignalError;

pub trait LiveSignalGenerator: Send {
    /// Stable identifier — used by the engine for slot-disjointness checks
    /// and by the tracker for `open_count_for(strategy_id)`. Must be unique
    /// within an engine instance.
    fn strategy_id(&self) -> &str;

    /// The set of tickers this strategy is allowed to trade. The engine
    /// validates disjointness across slots at construction and filters every
    /// emitted signal against this set at runtime — undeclared symbols are
    /// dropped with a warning, not silently passed through.
    fn symbols(&self) -> &[String];

    /// Candle interval the engine polls this generator on. Defaults to
    /// `analysis_interval()`. A strategy that wants to evaluate the
    /// in-progress higher-interval candle on a finer cadence can override
    /// this (e.g. analysis "1h", poll "15m").
    fn poll_interval(&self) -> &str {
        self.analysis_interval()
    }

    /// Candle interval the strategy actually analyses.
    fn analysis_interval(&self) -> &str {
        "1h"
    }

    /// Position-leverage hint used by the engine's affordable-entries
    /// calculation: `buying_power = available_balance * leverage`. Default 1.0.
    fn leverage(&self) -> f64 {
        1.0
    }

    /// Called once before the polling loop starts. The generator may stash
    /// the `Arc<dyn LiveMarketClient>` for use during subsequent `poll()`
    /// calls (e.g. to fetch klines for indicator warmup).
    ///
    /// Setup errors of either variant are treated as fatal — the engine
    /// will not start trading without a successful warmup.
    fn setup(
        &mut self,
        _client: Arc<dyn LiveMarketClient>,
    ) -> Result<(), SignalError> {
        Ok(())
    }

    /// Engine-injected wall time. Generators must use this instead of
    /// `Utc::now()` so two slots polling the same boundary observe a
    /// consistent clock.
    fn set_poll_time(&mut self, _now: DateTime<Utc>) {}

    /// Produce zero or more signals. Empty Vec means no-trade this poll.
    /// - `Err(SignalError::Fatal)` halts the engine after a clean shutdown.
    /// - `Err(SignalError::Recoverable)` is logged and treated as no-signals;
    ///   engine continues. Use this for transient data-fetch failures so a
    ///   single bad poll doesn't kill a long-running engine.
    fn poll(&mut self) -> Result<Vec<Signal>, SignalError>;

    /// Called once on engine shutdown — even on error paths. Default no-op.
    fn teardown(&mut self) {}
}
