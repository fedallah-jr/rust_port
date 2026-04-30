//! BTC structure configuration — all tunable thresholds.

use serde::{Deserialize, Serialize};

/// Default level windows (days) for confluence detection.
pub const BASE_LEVEL_DAYS: &[usize] = &[1, 3, 7, 10, 30, 90, 180, 300, 365];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BtcStructureConfig {
    pub interval: String,
    pub market_type: String,
    pub years: usize,

    // Swing detection
    pub rolling_lookback: usize,
    pub atr_window: usize,
    pub atr_multiplier: f64,
    pub pct_threshold: f64,
    pub min_bars_confirmation: usize,
    pub force_confirmation_after_bars: usize,
    pub max_candidate_bars: usize,

    // Confluence
    pub level_windows: Vec<usize>,
    pub level_confluence_required: usize,
    pub level_tolerance_atr_multiplier: f64,
    pub require_multi_horizon_confluence: bool,
    pub short_confluence_max_window: usize,
    pub long_confluence_min_window: usize,
    pub min_short_confluence_hits: usize,
    pub min_long_confluence_hits: usize,

    // Candidate replacement
    pub candidate_replace_min_atr_step: f64,
    pub candidate_replace_min_pct_step: f64,

    // Classification
    pub hhll_tolerance_atr_multiplier: f64,
    pub hhll_tolerance_pct: f64,

    // Break detection
    pub bos_choch_atr_multiplier: f64,
    pub bos_choch_pct: f64,
}

impl Default for BtcStructureConfig {
    fn default() -> Self {
        Self {
            interval: "1d".to_string(),
            market_type: "futures".to_string(),
            years: 5,
            rolling_lookback: 400,
            atr_window: 14,
            atr_multiplier: 1.25,
            pct_threshold: 0.015,
            min_bars_confirmation: 3,
            force_confirmation_after_bars: 7,
            max_candidate_bars: 18,
            level_windows: BASE_LEVEL_DAYS.to_vec(),
            level_confluence_required: 2,
            level_tolerance_atr_multiplier: 0.50,
            require_multi_horizon_confluence: true,
            short_confluence_max_window: 30,
            long_confluence_min_window: 90,
            min_short_confluence_hits: 1,
            min_long_confluence_hits: 1,
            candidate_replace_min_atr_step: 0.10,
            candidate_replace_min_pct_step: 0.001,
            hhll_tolerance_atr_multiplier: 0.15,
            hhll_tolerance_pct: 0.001,
            bos_choch_atr_multiplier: 0.35,
            bos_choch_pct: 0.003,
        }
    }
}

impl BtcStructureConfig {
    /// Validate that critical parameters won't cause panics.
    pub fn validate(&self) -> Result<(), String> {
        if self.atr_window == 0 {
            return Err("atr_window must be > 0".into());
        }
        Ok(())
    }
}
