//! Indicator engine — Rust port of `backtester/indicators.py`.
//!
//! Computes technical indicators on OHLCV column arrays. All indicators
//! match Python's pandas-based implementation exactly (same EWM alpha,
//! same window sizes, same edge cases).
//!
//! ## Usage
//!
//! ```ignore
//! use claude_trader_indicators::{compute_indicators, required_warmup};
//!
//! let result = compute_indicators(&ohlcv, &["rsi_14", "atr_14", "bb_upper"]).unwrap();
//! let warmup = required_warmup(&["rsi_14", "atr_14"]);
//! ```

mod compute;
mod registry;

pub use compute::compute_indicators;
pub use registry::{required_warmup, IndicatorSpec, INDICATOR_SPECS, RAW_INPUTS};

/// OHLCV data as column arrays. All vectors must have the same length.
#[derive(Debug, Clone)]
pub struct OhlcvFrame {
    pub open: Vec<f64>,
    pub high: Vec<f64>,
    pub low: Vec<f64>,
    pub close: Vec<f64>,
    pub volume: Vec<f64>,
    pub taker_buy_volume: Vec<f64>,
}

impl OhlcvFrame {
    pub fn len(&self) -> usize {
        self.close.len()
    }

    pub fn is_empty(&self) -> bool {
        self.close.is_empty()
    }
}

/// Result of indicator computation — named columns of f64 values.
pub type IndicatorResult = std::collections::HashMap<String, Vec<f64>>;
