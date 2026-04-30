//! DailyStructureProvider — high-level interface for strategies.
//!
//! Fetches BTC daily OHLCV data, runs the structure simulation and feature
//! lab, and provides merge-onto functionality for attaching daily structure
//! features to intraday trading frames.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use claude_trader_models::Candle;

use crate::config::BtcStructureConfig;
use crate::engine::{self, FeatureValue, StructureArtifacts, StructureCheckpoint};
use crate::features::{self, StructureLabArtifacts};

/// High-level provider for daily BTC structure features.
///
/// Strategies use this to gate entries based on market structure.
/// Supports both backtesting (ensure_computed_until + merge_onto) and
/// live trading (refresh_if_stale + latest).
pub struct DailyStructureProvider {
    config: BtcStructureConfig,
    structure: Option<StructureArtifacts>,
    lab: Option<StructureLabArtifacts>,
    checkpoint: Option<StructureCheckpoint>,
    last_computed_until: Option<DateTime<Utc>>,
}

impl DailyStructureProvider {
    pub fn new() -> Self {
        Self {
            config: BtcStructureConfig::default(),
            structure: None,
            lab: None,
            checkpoint: None,
            last_computed_until: None,
        }
    }

    /// Whether computation has completed at least once.
    pub fn is_ready(&self) -> bool {
        self.lab.is_some()
    }

    /// Recompute structure features from daily candles up to `cutoff`.
    ///
    /// In backtesting, call this with the end of the data window.
    /// The provider will fetch BTC daily candles and run the full pipeline.
    pub fn compute_from_candles(&mut self, candles: &[Candle]) {
        if candles.is_empty() {
            return;
        }
        if let Err(e) = self.config.validate() {
            eprintln!("ERROR: BtcStructureConfig invalid: {e}");
            return;
        }

        let dates: Vec<DateTime<Utc>> = candles.iter().map(|c| c.close_time).collect();
        let open: Vec<f64> = candles.iter().map(|c| c.open).collect();
        let high: Vec<f64> = candles.iter().map(|c| c.high).collect();
        let low: Vec<f64> = candles.iter().map(|c| c.low).collect();
        let close: Vec<f64> = candles.iter().map(|c| c.close).collect();

        let (artifacts, checkpoint) = engine::simulate_btc_structure(
            &dates,
            &open,
            &high,
            &low,
            &close,
            &self.config,
            self.checkpoint.take(),
            self.structure.take(),
        );

        let lab = features::run_structure_feature_lab(&artifacts);

        self.last_computed_until = dates.last().copied();
        self.checkpoint = Some(checkpoint);
        self.structure = Some(artifacts);
        self.lab = Some(lab);
    }

    /// Get the latest feature row (most recent completed daily bar).
    pub fn latest(&self) -> Option<&HashMap<&'static str, FeatureValue>> {
        self.lab.as_ref().and_then(|lab| lab.feature_matrix.last())
    }

    /// Get the full feature matrix.
    pub fn feature_matrix(&self) -> Option<&Vec<HashMap<&'static str, FeatureValue>>> {
        self.lab.as_ref().map(|lab| &lab.feature_matrix)
    }

    /// Reset all cached state.
    pub fn reset(&mut self) {
        self.structure = None;
        self.lab = None;
        self.checkpoint = None;
        self.last_computed_until = None;
    }
}
