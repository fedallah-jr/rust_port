//! Backtester model types — mirrors `backtester/models.py`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PositionType {
    #[serde(rename = "LONG")]
    Long,
    #[serde(rename = "SHORT")]
    Short,
}

impl PositionType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Long => "LONG",
            Self::Short => "SHORT",
        }
    }

    pub fn is_long(&self) -> bool {
        matches!(self, Self::Long)
    }
}

impl std::fmt::Display for PositionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MarketType {
    #[serde(rename = "SPOT")]
    Spot,
    #[serde(rename = "FUTURES")]
    Futures,
}

impl MarketType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Spot => "SPOT",
            Self::Futures => "FUTURES",
        }
    }
}

impl std::fmt::Display for MarketType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ExitReason {
    #[serde(rename = "TP")]
    Tp,
    #[serde(rename = "SL")]
    Sl,
    #[serde(rename = "TIMEOUT")]
    Timeout,
    #[serde(rename = "UNFILLED")]
    Unfilled,
}

impl ExitReason {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Tp => "TP",
            Self::Sl => "SL",
            Self::Timeout => "TIMEOUT",
            Self::Unfilled => "UNFILLED",
        }
    }
}

impl std::fmt::Display for ExitReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResolutionLevel {
    #[serde(rename = "1h")]
    Hour,
    #[serde(rename = "1m")]
    Minute,
    #[serde(rename = "trade")]
    Trade,
}

impl ResolutionLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Hour => "1h",
            Self::Minute => "1m",
            Self::Trade => "trade",
        }
    }
}

impl std::fmt::Display for ResolutionLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

// ---------------------------------------------------------------------------
// Signal
// ---------------------------------------------------------------------------

/// A trading signal — the contract between strategy logic and execution.
///
/// Immutable once constructed. Mirrors Python `Signal` (frozen dataclass).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signal {
    pub signal_date: DateTime<Utc>,
    pub position_type: PositionType,
    pub ticker: String,

    /// Strategy pattern that generated this signal (e.g. "buy_dip", "sell_rip").
    #[serde(default)]
    pub pattern: String,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tp_pct: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sl_pct: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tp_price: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sl_price: Option<f64>,

    #[serde(default = "default_leverage")]
    pub leverage: f64,
    #[serde(default = "default_market_type")]
    pub market_type: MarketType,
    #[serde(default = "default_taker_fee_rate")]
    pub taker_fee_rate: f64,

    /// `None` = market order; `Some(price)` = limit order threshold.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entry_price: Option<f64>,
    #[serde(default = "default_fill_timeout_seconds")]
    pub fill_timeout_seconds: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entry_delay_seconds: Option<i64>,
    #[serde(default = "default_max_holding_hours")]
    pub max_holding_hours: i64,

    #[serde(default = "default_size_multiplier")]
    pub size_multiplier: f64,

    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

fn default_leverage() -> f64 {
    1.0
}
fn default_market_type() -> MarketType {
    MarketType::Futures
}
fn default_taker_fee_rate() -> f64 {
    0.0005
}
fn default_fill_timeout_seconds() -> i64 {
    3600
}
fn default_max_holding_hours() -> i64 {
    72
}
fn default_size_multiplier() -> f64 {
    1.0
}

impl Signal {
    /// Validate invariants (mirrors Python `__post_init__`).
    pub fn validate(&self) -> Result<(), String> {
        if self.tp_pct.is_none() && self.tp_price.is_none() {
            return Err("Signal requires at least one of tp_pct or tp_price".into());
        }
        if self.sl_pct.is_none() && self.sl_price.is_none() {
            return Err("Signal requires at least one of sl_pct or sl_price".into());
        }
        // Validate percentages are finite and positive when provided
        if let Some(tp) = self.tp_pct {
            if !tp.is_finite() || tp <= 0.0 {
                return Err(format!("tp_pct must be finite and positive, got {tp}"));
            }
        }
        if let Some(sl) = self.sl_pct {
            if !sl.is_finite() || sl <= 0.0 {
                return Err(format!("sl_pct must be finite and positive, got {sl}"));
            }
        }
        if let Some(tp) = self.tp_price {
            if !tp.is_finite() || tp <= 0.0 {
                return Err(format!("tp_price must be finite and positive, got {tp}"));
            }
        }
        if let Some(sl) = self.sl_price {
            if !sl.is_finite() || sl <= 0.0 {
                return Err(format!("sl_price must be finite and positive, got {sl}"));
            }
        }
        if !self.leverage.is_finite() || self.leverage <= 0.0 {
            return Err(format!(
                "leverage must be finite and positive, got {}",
                self.leverage
            ));
        }
        if !self.size_multiplier.is_finite() || self.size_multiplier <= 0.0 {
            return Err(format!(
                "size_multiplier must be finite and positive, got {}",
                self.size_multiplier
            ));
        }
        if let Some(delay) = self.entry_delay_seconds {
            if delay < 0 {
                return Err("entry_delay_seconds must be non-negative".into());
            }
        }
        if self.max_holding_hours <= 0 {
            return Err("max_holding_hours must be positive".into());
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// AggTrade
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct AggTrade {
    pub trade_id: i64,
    pub timestamp: DateTime<Utc>,
    pub price: f64,
    pub quantity: f64,
}

// ---------------------------------------------------------------------------
// Candle
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Candle {
    pub open_time: DateTime<Utc>,
    pub close_time: DateTime<Utc>,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    #[serde(default)]
    pub volume: f64,
    #[serde(default)]
    pub taker_buy_volume: f64,
}

// ---------------------------------------------------------------------------
// ExitResolution
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ExitResolution {
    pub reason: ExitReason,
    pub exit_time: DateTime<Utc>,
    pub exit_price: f64,
    pub resolution_level: ResolutionLevel,
    #[serde(default)]
    pub exit_fallback: bool,
    #[serde(default)]
    pub random_resolved: bool,
}

// ---------------------------------------------------------------------------
// TradeResult
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeResult {
    pub signal: Arc<Signal>,
    pub entry_price: f64,
    pub entry_time: DateTime<Utc>,
    pub exit_price: f64,
    pub exit_time: DateTime<Utc>,
    pub exit_reason: ExitReason,
    pub resolution_level: ResolutionLevel,
    pub tp_price: f64,
    pub sl_price: f64,
    pub pnl_pct: f64,
    pub gross_pnl_pct: f64,
    pub fee_drag_pct: f64,
    #[serde(default)]
    pub entry_fallback: bool,
    #[serde(default)]
    pub exit_fallback: bool,
    #[serde(default)]
    pub random_resolved: bool,
}

// ---------------------------------------------------------------------------
// BacktestResult
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BacktestResult {
    pub trades: Vec<TradeResult>,
    pub total_trades: usize,
    pub wins: usize,
    pub losses: usize,
    pub open_trades: usize,
    pub unfilled: usize,
    pub win_rate: f64,
    pub total_pnl_pct: f64,
    pub avg_pnl_pct: f64,
    pub profit_factor: f64,
    pub max_drawdown_pct: f64,
    pub equity_curve: Vec<f64>,
}

// ---------------------------------------------------------------------------
// FundingRate
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct FundingRate {
    pub timestamp: DateTime<Utc>,
    pub funding_rate: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mark_price: Option<f64>,
}

// ---------------------------------------------------------------------------
// Shared equity / drawdown helpers
// ---------------------------------------------------------------------------

/// Compute additive equity curve and max drawdown from a chronological trade list.
/// Uses fixed-dollar sizing: each trade's weighted PnL is added to equity.
pub fn compute_equity_and_drawdown(trades: &[&TradeResult]) -> (Vec<f64>, f64) {
    let mut equity = 100.0f64;
    let mut peak = equity;
    let mut max_dd = 0.0f64;
    let mut equity_curve = Vec::with_capacity(trades.len() + 1);
    equity_curve.push(equity);

    for t in trades {
        let pnl_weighted = t.pnl_pct * t.signal.size_multiplier;
        if !pnl_weighted.is_finite() {
            continue;
        }
        equity += pnl_weighted;
        equity_curve.push(equity);

        if equity > peak {
            peak = equity;
        }
        if peak > 0.0 {
            let dd = (peak - equity) / peak * 100.0;
            if dd > max_dd {
                max_dd = dd;
            }
        }
    }

    (equity_curve, max_dd)
}

// ---------------------------------------------------------------------------
// Parity test epsilon constants
// ---------------------------------------------------------------------------

/// Epsilon for price comparisons (entry_price, exit_price, tp_price, sl_price).
pub const EPSILON_PRICE: f64 = 1e-10;

/// Epsilon for percentage comparisons (pnl_pct, win_rate, drawdown).
pub const EPSILON_PCT: f64 = 1e-6;
