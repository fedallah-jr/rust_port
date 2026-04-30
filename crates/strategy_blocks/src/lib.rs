//! `claude_trader_strategy_blocks` — composable, pure strategy primitives.
//!
//! This crate factors out the detectors, gates, and setup → trigger
//! bookkeeping that research crates otherwise end up re-implementing.
//! Every experiment in `rust_port/research/` can depend on this crate
//! and compose signals instead of copy-pasting divergence / breakout /
//! level / bias-gate code.
//!
//! # Purity contract
//!
//! **Every function in this crate MUST be a pure function of its
//! arguments.** This is not a style preference — it's a correctness
//! requirement. The research runtime's no-lookahead validator treats
//! `generate_signals()` as a black box. It replays each signal with
//! *truncated* inputs (candles, supplementary data, BTC bias, calibration
//! params) and requires identical output. Any of the following would
//! silently break that guarantee:
//!
//! - caching data between calls (e.g. a `static` indicator cache)
//! - `thread_local!` / `static mut` / `Mutex`-guarded state
//! - I/O of any kind (file, network, clock)
//! - reading data outside the slices/references passed in
//! - internal RNG not seeded from the arguments
//!
//! The only mutable state allowed is PER-INVOCATION: e.g. a
//! [`TwoStageBook`] owned on the stack by `generate_signals()` and
//! thrown away when the function returns. The runtime's validator
//! rebuilds all state each call, so per-invocation state is fine.
//!
//! # Layout
//!
//! - [`events`]: shared detection/event structs returned by detectors.
//! - [`divergence`]: CVD + RSI bearish / bullish double-divergence.
//! - [`breakout`]: Donchian-style high / low breakouts.
//! - [`levels`]: key-level pierce/rejection detectors.
//! - [`setup`]: `TwoStageBook<S>` — generic setup → trigger state machine.
//! - [`gates`]: BTC-bias / funding / 4h HTF gate helpers.
//!
//! # Performance
//!
//! Hot-path detectors allocate nothing and do a single bounded scan per
//! call. `TwoStageBook` allocates only when its internal Vec grows
//! (use [`TwoStageBook::with_capacity`] to pre-size).

pub mod breakout;
pub mod divergence;
pub mod events;
pub mod gates;
pub mod levels;
pub mod setup;

// Re-exports of the public surface. Strategy crates should mostly import
// from the top-level to keep use-lines short:
//     use claude_trader_strategy_blocks::{
//         bearish_double_divergence, DivergenceParams, TwoStageBook,
//         btc_bias_bullish, FundingGateParams,
//     };

pub use breakout::{donchian_high_break, donchian_low_break};
pub use divergence::{bearish_double_divergence, bullish_double_divergence, DivergenceParams};
pub use events::{BreakoutDetection, Direction, DivergenceDetection, LevelPierce};
pub use gates::{
    btc_bias_bearish, btc_bias_bullish, funding_crowded_long, funding_crowded_short,
    htf_4h_not_strongly_bearish, htf_4h_not_strongly_bullish, FundingGateParams,
};
pub use levels::{pierced_resistance, pierced_support, LevelPierceParams};
pub use setup::TwoStageBook;
