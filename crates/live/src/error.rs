//! Live-runtime error type.
//!
//! Distinguishes the *kinds* of failure the engine and tracker need to react
//! to differently:
//!
//! - `Api { code, msg }` carries the Binance error payload so callers can
//!   special-case `-1021` (clock skew → force-resync), `-2013` ("Order does
//!   not exist" → idempotent re-POST allowed), and the terminal-4xx allow-list
//!   that lets `submit_entry_order` mark a position FAILED definitively.
//! - `Http(String)` is the catch-all for everything below the API layer
//!   (transport, JSON parse). Outcome is *unknown* — order placement must
//!   fall through the idempotent-recovery path before retrying.
//! - `InsufficientBalance` / `UnknownSymbol` / `ZeroQuantity` are pre-flight
//!   rejections raised by the executor before any HTTP call.
//! - `Config(ConfigError)` and `Io(std::io::Error)` propagate from disk-side
//!   helpers; rarely retried, surfaced to the operator at startup.

use thiserror::Error;

use claude_trader_models::ConfigError;

#[derive(Debug, Error)]
pub enum LiveError {
    #[error("HTTP error: {0}")]
    Http(String),

    /// A Binance API error payload. `code` matches Binance's documented error
    /// numbers (negative integers); `msg` is the server-supplied message.
    /// Callers route on `code`:
    /// - `-1021` → force time resync, retry once.
    /// - `-2011` (cancel of unknown order) → swallowed by `cancel_order_safe`.
    /// - `-2013` (query/get unknown order) → idempotent path allowed to re-POST.
    /// - terminal-4xx allow-list → mark position FAILED.
    #[error("Binance API error {code}: {msg}")]
    Api { code: i64, msg: String },

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Insufficient balance: need {required:.2} USDT, have {available:.2}")]
    InsufficientBalance { required: f64, available: f64 },

    #[error("Symbol not found in exchange info: {0}")]
    UnknownSymbol(String),

    #[error("Quantity rounded to zero for {0}")]
    ZeroQuantity(String),

    #[error("State persistence error: {0}")]
    State(String),

    #[error("Config error: {0}")]
    Config(#[from] ConfigError),

    /// Reserved for `FatalSignalError` from `signal_generator.rs`. The actual
    /// type is added in Phase E; the variant exists now so callers can pattern
    /// match without a future-incompatible change.
    #[error("Fatal signal-generator error: {0}")]
    Fatal(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T, E = LiveError> = std::result::Result<T, E>;

impl LiveError {
    /// Returns the Binance error code if this is an `Api` variant, else `None`.
    /// Convenience for the recovery-loop dispatch.
    pub fn api_code(&self) -> Option<i64> {
        match self {
            LiveError::Api { code, .. } => Some(*code),
            _ => None,
        }
    }

    /// True for the `-1021` "timestamp out of bounds" code, signalling that
    /// `submit_entry_order` should force-resync server time and retry once.
    pub fn is_timestamp_skew(&self) -> bool {
        self.api_code() == Some(-1021)
    }

    /// True for `-2013` "Order does not exist", which the idempotent-recovery
    /// loop interprets as "safe to re-POST with the same client ID".
    pub fn is_unknown_order(&self) -> bool {
        self.api_code() == Some(-2013)
    }

    /// True for `-2011` "Unknown order sent" (returned by cancels of orders
    /// that have already been canceled or filled). `cancel_order_safe`
    /// swallows this code so a tracker re-cancel doesn't blow up.
    pub fn is_cancel_of_unknown(&self) -> bool {
        self.api_code() == Some(-2011)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn api_code_extraction() {
        let err = LiveError::Api {
            code: -1021,
            msg: "Timestamp for this request is outside of the recvWindow".into(),
        };
        assert_eq!(err.api_code(), Some(-1021));
        assert!(err.is_timestamp_skew());
        assert!(!err.is_unknown_order());
        assert!(!err.is_cancel_of_unknown());
    }

    #[test]
    fn non_api_errors_have_no_code() {
        let err = LiveError::Http("connection reset".into());
        assert_eq!(err.api_code(), None);
        assert!(!err.is_timestamp_skew());
    }

    #[test]
    fn unknown_order_code() {
        let err = LiveError::Api {
            code: -2013,
            msg: "Order does not exist.".into(),
        };
        assert!(err.is_unknown_order());
    }

    #[test]
    fn cancel_unknown_code() {
        let err = LiveError::Api {
            code: -2011,
            msg: "Unknown order sent.".into(),
        };
        assert!(err.is_cancel_of_unknown());
    }

    #[test]
    fn config_error_propagates_via_from() {
        let cfg_err = ConfigError::Invalid("bad".into());
        let live_err: LiveError = cfg_err.into();
        assert!(matches!(live_err, LiveError::Config(_)));
    }
}
