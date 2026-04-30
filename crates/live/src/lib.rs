//! Live trading core — Rust port of `live/`.
//!
//! Implements:
//! - `error.rs` — `LiveError` + `Result<T>` alias
//! - `time_sync.rs` — monotonic-anchored server-time clock
//! - `auth_client.rs` — Binance Futures REST client with HMAC signature
//! - `executor.rs` — Signal → exchange order conversion
//! - `tracker.rs` — Position state machine with persistence
//! - `engine.rs` — Main polling loop with multi-strategy support

pub mod auth_client;
pub mod engine;
pub mod error;
pub mod exchange_info;
pub mod executor;
pub mod market_client;
pub mod order_count;
pub mod shutdown;
pub mod signal_generator;
pub mod time_sync;
pub mod tracker;

pub use error::{LiveError, Result};
