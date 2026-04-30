//! Cooldown contract shared between strategies and the runtime.
//!
//! Strategies emit every candidate signal and declare a `CooldownSpec` per
//! signal. The runtime applies the resulting filter globally across a run —
//! strategies MUST NOT track cooldown internally.
//!
//! # Key construction
//!
//! Use the named constructors (`CooldownKey::symbol`, `::symbol_side`,
//! `::symbol_pattern`, `::pattern`) for the common cases. `CooldownKey::custom`
//! wraps the input under an internal `custom:` prefix, so user-supplied keys
//! cannot collide with the built-in namespaces.
//!
//! # Invariant
//!
//! Two signals that share the same `CooldownKey` must always carry the same
//! `hours` within a run. Encode different cooldown regimes as different keys.
//! The runtime panics on inconsistent hours under `debug_assertions`.

use std::sync::Arc;

use crate::{PositionType, Signal};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CooldownKey(Arc<str>);

impl CooldownKey {
    pub fn symbol(ticker: &str) -> Self {
        Self(Arc::from(format!("sym:{ticker}")))
    }

    pub fn symbol_side(ticker: &str, side: PositionType) -> Self {
        Self(Arc::from(format!("sym:{ticker}|side:{}", side.as_str())))
    }

    pub fn symbol_pattern(ticker: &str, pattern: &str) -> Self {
        Self(Arc::from(format!("sym:{ticker}|pat:{pattern}")))
    }

    pub fn pattern(pattern: &str) -> Self {
        Self(Arc::from(format!("pat:{pattern}")))
    }

    /// Caller-defined key. The input is namespaced under `custom:` so it
    /// cannot shadow `symbol`, `symbol_side`, `symbol_pattern`, or `pattern`.
    pub fn custom(s: &str) -> Self {
        Self(Arc::from(format!("custom:{s}")))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub struct CooldownSpec {
    pub key: CooldownKey,
    pub hours: f64,
}

impl CooldownSpec {
    pub fn symbol_side(signal: &Signal, hours: f64) -> Self {
        Self {
            key: CooldownKey::symbol_side(&signal.ticker, signal.position_type),
            hours,
        }
    }

    pub fn symbol(signal: &Signal, hours: f64) -> Self {
        Self {
            key: CooldownKey::symbol(&signal.ticker),
            hours,
        }
    }

    pub fn symbol_pattern(signal: &Signal, hours: f64) -> Self {
        Self {
            key: CooldownKey::symbol_pattern(&signal.ticker, &signal.pattern),
            hours,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn custom_cannot_collide_with_symbol() {
        let custom = CooldownKey::custom("sym:BTCUSDT");
        let real = CooldownKey::symbol("BTCUSDT");
        assert_ne!(custom, real);
        assert_eq!(custom.as_str(), "custom:sym:BTCUSDT");
        assert_eq!(real.as_str(), "sym:BTCUSDT");
    }

    #[test]
    fn symbol_side_distinguishes_direction() {
        let long = CooldownKey::symbol_side("ETHUSDT", PositionType::Long);
        let short = CooldownKey::symbol_side("ETHUSDT", PositionType::Short);
        assert_ne!(long, short);
    }

    #[test]
    fn same_ticker_same_side_hashes_equal() {
        let a = CooldownKey::symbol_side("BTCUSDT", PositionType::Long);
        let b = CooldownKey::symbol_side("BTCUSDT", PositionType::Long);
        assert_eq!(a, b);
    }
}
