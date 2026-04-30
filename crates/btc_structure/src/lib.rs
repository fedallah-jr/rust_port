//! BTC Structure engine — swing detection, structure breaks, feature extraction.
//!
//! The canonical way to consume this crate from a research strategy is via
//! `ContextKey::BtcStructure` (which returns `ContextValue::Bias(MarketBias)`
//! from the shared models crate). The runtime wires that up using
//! [`DailyStructureProvider`] internally.
//!
//! For advanced use (e.g. building a custom structure-driven feature set),
//! reach into [`engine`] directly: `StructureArtifacts`, `StructureCheckpoint`,
//! `FeatureValue`, and `simulate_btc_structure` are all defined there.

mod config;
pub mod engine;
mod features;
mod provider;
mod ranking;

pub use provider::DailyStructureProvider;
