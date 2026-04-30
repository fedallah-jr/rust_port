//! Binance data client with disk cache, pagination, and rate limiting.
//!
//! Rust port of `backtester/data.py`. Fetches klines, agg trades, funding
//! rates, mark price klines, and premium index klines from the Binance
//! Futures REST API.
//!
//! ## Caching
//!
//! Uses a triple-layered cache: memory → disk → HTTP.
//! Disk cache uses SHA-256 keyed bincode files under `~/.claude_trader/cache/binance_rs/`.
//! The Python cache (`binance/`) is not shared — each runtime has its own namespace.

pub mod cache;
pub mod candle_store;
pub mod client;
pub mod coverage;
pub mod rate_limiter;

pub use candle_store::CandleStore;
pub use client::{BinanceClient, DataError, KlineFetch};
pub use coverage::Coverage;
