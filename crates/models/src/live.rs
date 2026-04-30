//! Live trading model types — mirrors `live/models.py`.

use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::backtester::Signal;

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OrderSide {
    #[serde(rename = "BUY")]
    Buy,
    #[serde(rename = "SELL")]
    Sell,
}

impl OrderSide {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Buy => "BUY",
            Self::Sell => "SELL",
        }
    }
}

impl std::fmt::Display for OrderSide {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OrderStatus {
    #[serde(rename = "NEW")]
    New,
    #[serde(rename = "FILLED")]
    Filled,
    #[serde(rename = "CANCELED")]
    Canceled,
    #[serde(rename = "EXPIRED")]
    Expired,
    #[serde(rename = "REJECTED")]
    Rejected,
}

impl OrderStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "NEW",
            Self::Filled => "FILLED",
            Self::Canceled => "CANCELED",
            Self::Expired => "EXPIRED",
            Self::Rejected => "REJECTED",
        }
    }
}

impl std::fmt::Display for OrderStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OrderType {
    #[serde(rename = "MARKET")]
    Market,
    #[serde(rename = "LIMIT")]
    Limit,
    #[serde(rename = "STOP_MARKET")]
    StopMarket,
    #[serde(rename = "TAKE_PROFIT_MARKET")]
    TakeProfitMarket,
}

impl OrderType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Market => "MARKET",
            Self::Limit => "LIMIT",
            Self::StopMarket => "STOP_MARKET",
            Self::TakeProfitMarket => "TAKE_PROFIT_MARKET",
        }
    }
}

impl std::fmt::Display for OrderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PositionStatus {
    #[serde(rename = "PENDING_ENTRY")]
    PendingEntry,
    #[serde(rename = "OPEN")]
    Open,
    #[serde(rename = "CLOSED")]
    Closed,
    #[serde(rename = "FAILED")]
    Failed,
}

impl PositionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::PendingEntry => "PENDING_ENTRY",
            Self::Open => "OPEN",
            Self::Closed => "CLOSED",
            Self::Failed => "FAILED",
        }
    }
}

impl std::fmt::Display for PositionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

// ---------------------------------------------------------------------------
// AccountTrade
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountTrade {
    pub trade_id: i64,
    pub order_id: i64,
    pub symbol: String,
    pub side: OrderSide,
    pub price: f64,
    pub quantity: f64,
    pub time: DateTime<Utc>,
    #[serde(default)]
    pub realized_pnl: f64,
    #[serde(default)]
    pub commission: f64,
    #[serde(default)]
    pub commission_asset: String,
    #[serde(default = "default_position_side")]
    pub position_side: String,
}

fn default_position_side() -> String {
    "BOTH".to_string()
}

// ---------------------------------------------------------------------------
// ExchangeOrder
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeOrder {
    pub order_id: i64,
    pub symbol: String,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub quantity: f64,
    pub price: f64,
    pub stop_price: f64,
    pub status: OrderStatus,
    #[serde(default)]
    pub filled_qty: f64,
    #[serde(default)]
    pub avg_fill_price: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub algo_id: i64,
    /// `newClientOrderId` (normal orders) or `clientAlgoId` (algo orders).
    /// Generated and persisted *before* the first POST so 5xx / network
    /// recovery can query Binance by ID and decide whether the placement
    /// reached the matching engine. Empty string for orders surfaced by
    /// reconciliation (we did not place them ourselves).
    #[serde(default)]
    pub client_order_id: String,
}

// ---------------------------------------------------------------------------
// GeneratorBudget
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct GeneratorBudget {
    #[serde(default = "default_position_size_usdt")]
    pub position_size_usdt: f64,
    #[serde(default = "default_max_positions")]
    pub max_positions: usize,
}

fn default_position_size_usdt() -> f64 {
    100.0
}
fn default_max_positions() -> usize {
    3
}

impl Default for GeneratorBudget {
    fn default() -> Self {
        Self {
            position_size_usdt: default_position_size_usdt(),
            max_positions: default_max_positions(),
        }
    }
}

// ---------------------------------------------------------------------------
// LivePosition
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LivePosition {
    pub signal: Signal,
    pub position_id: String,
    #[serde(default)]
    pub strategy_id: String,
    #[serde(default = "default_position_status")]
    pub status: PositionStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entry_order: Option<ExchangeOrder>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tp_order: Option<ExchangeOrder>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sl_order: Option<ExchangeOrder>,
    #[serde(default)]
    pub fill_price: f64,
    #[serde(default)]
    pub quantity: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub opened_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exit_price: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pnl_pct: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gross_pnl_pct: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fee_drag_pct: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub closed_at: Option<DateTime<Utc>>,
}

fn default_position_status() -> PositionStatus {
    PositionStatus::PendingEntry
}

// ---------------------------------------------------------------------------
// LiveConfig
// ---------------------------------------------------------------------------

/// Production Binance USD-M futures REST base URL.
pub const PROD_BASE_URL: &str = "https://fapi.binance.com";

/// Testnet base URL per current Binance USD-M docs (April 2026).
///
/// The legacy URL `https://testnet.binancefuture.com` may still resolve in
/// some deployments; operators wanting it must set `BINANCE_BASE_URL`
/// explicitly or write `base_url` in the JSON config. The startup banner
/// emits a warning when the legacy URL is in use.
pub const TESTNET_BASE_URL: &str = "https://demo-fapi.binance.com";

/// Legacy testnet URL — kept only so the startup banner can recognise and
/// warn about it. Not used as a default anywhere.
pub const LEGACY_TESTNET_BASE_URL: &str = "https://testnet.binancefuture.com";

const ENV_API_KEY: &str = "BINANCE_API_KEY";
const ENV_API_SECRET: &str = "BINANCE_API_SECRET";
const ENV_TESTNET: &str = "BINANCE_TESTNET";
const ENV_BASE_URL: &str = "BINANCE_BASE_URL";
const ENV_POSITION_SIZE: &str = "BINANCE_POSITION_SIZE";
const ENV_MAX_POSITIONS: &str = "BINANCE_MAX_POSITIONS";
const ENV_ORDER_CHECK_INTERVAL: &str = "BINANCE_ORDER_CHECK_INTERVAL";

/// Env vars that used to live in the live runtime and have been pushed back
/// into the strategy. Setting any of these at engine startup is a hard error
/// — failing loud beats silently ignoring the operator's intent.
const REMOVED_ENV_VARS: &[&str] = &["BINANCE_MAX_HOLDING_HOURS"];

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Config file not found: {0}")]
    NotFound(PathBuf),

    #[error("Failed to read config file {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to parse config file {path}: {source}")]
    Parse {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },

    #[error("Invalid live config: {0}")]
    Invalid(String),

    #[error("Removed env var still set: {0}. {1}")]
    DeprecatedEnv(String, String),

    #[error(
        "No API credentials found. Set {ENV_API_KEY}/{ENV_API_SECRET} env vars or create {0}"
    )]
    NoCredentials(PathBuf),

    #[error("Env var {name}={value:?} could not be parsed as {kind}")]
    BadEnv {
        name: String,
        value: String,
        kind: &'static str,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LiveConfig {
    pub api_key: String,
    pub api_secret: String,
    #[serde(default = "default_base_url")]
    pub base_url: String,
    #[serde(default = "default_position_size_usdt")]
    pub position_size_usdt: f64,
    #[serde(default = "default_max_concurrent_positions")]
    pub max_concurrent_positions: usize,
    #[serde(default = "default_order_check_interval")]
    pub order_check_interval_seconds: f64,
    #[serde(default)]
    pub testnet: bool,
    /// On startup, after `load_state` and exchange reconciliation, attempt to
    /// re-place TP/SL for OPEN positions whose persisted brackets are missing
    /// or in a terminal state on the exchange. Tightly guarded: requires the
    /// exchange position still open, sane fill_price/quantity, and a
    /// definitive (non-Unknown) bracket-status query.
    #[serde(default = "default_recover_brackets_on_startup")]
    pub recover_brackets_on_startup: bool,
}

fn default_base_url() -> String {
    PROD_BASE_URL.to_string()
}
fn default_max_concurrent_positions() -> usize {
    3
}
fn default_order_check_interval() -> f64 {
    5.0
}
fn default_recover_brackets_on_startup() -> bool {
    true
}

impl LiveConfig {
    /// Validate invariants. Pure read of self.
    pub fn validate(&self) -> Result<(), String> {
        if self.api_key.is_empty() {
            return Err("api_key must not be empty".into());
        }
        if self.api_secret.is_empty() {
            return Err("api_secret must not be empty".into());
        }
        if self.base_url.is_empty() {
            return Err("base_url must not be empty".into());
        }
        if self.position_size_usdt <= 0.0 {
            return Err("position_size_usdt must be > 0".into());
        }
        if self.max_concurrent_positions == 0 {
            return Err("max_concurrent_positions must be > 0".into());
        }
        if self.order_check_interval_seconds <= 0.0 {
            return Err("order_check_interval_seconds must be > 0".into());
        }
        Ok(())
    }

    /// True when the resolved `base_url` matches the documented testnet URL
    /// (or the legacy testnet URL — both count). A bare `testnet=true` flag
    /// without the URL fixup is *not* sufficient on its own; we want operators
    /// to actually be hitting a non-prod endpoint before this returns true.
    pub fn is_testnet(&self) -> bool {
        self.base_url == TESTNET_BASE_URL || self.base_url == LEGACY_TESTNET_BASE_URL
    }

    /// Post-load normalization. Two responsibilities:
    ///
    /// 1. **Trailing-slash trim**. Without this, `BinanceFuturesClient`'s
    ///    `format!("{base_url}{path}")` produces `https://host//fapi/...`
    ///    when an operator supplies `https://demo-fapi.binance.com/`,
    ///    while `BinanceMarketClient::with_base_url` already trims its
    ///    own copy — the two clients disagree on the URL shape.
    ///    `is_testnet()` also stops recognising trailing-slash variants.
    ///    Normalising here once means every consumer reads a canonical URL.
    ///
    /// 2. **Testnet substitution**. Mirrors Python `__post_init__`: when
    ///    `testnet=true` and the URL is still production, swap to the
    ///    documented testnet endpoint. Operators who want the legacy URL
    ///    must set it explicitly.
    fn apply_testnet_fixup(&mut self) {
        let trimmed_len = self.base_url.trim_end_matches('/').len();
        if trimmed_len != self.base_url.len() {
            self.base_url.truncate(trimmed_len);
        }
        if self.testnet && self.base_url == PROD_BASE_URL {
            self.base_url = TESTNET_BASE_URL.to_string();
        }
    }

    /// Construct a new config with selected fields overridden. Mirrors Python
    /// `with_overrides(use_testnet=, position_size_usdt=, max_concurrent_positions=)`.
    /// `recover_brackets_on_startup` is preserved (no override hook) — it's
    /// not exposed via the operator CLI today.
    pub fn with_overrides(
        &self,
        use_testnet: bool,
        position_size_usdt: Option<f64>,
        max_concurrent_positions: Option<usize>,
    ) -> Self {
        let base_url = if use_testnet {
            TESTNET_BASE_URL.to_string()
        } else {
            self.base_url.clone()
        };
        let mut out = Self {
            api_key: self.api_key.clone(),
            api_secret: self.api_secret.clone(),
            base_url,
            position_size_usdt: position_size_usdt.unwrap_or(self.position_size_usdt),
            max_concurrent_positions: max_concurrent_positions
                .unwrap_or(self.max_concurrent_positions),
            order_check_interval_seconds: self.order_check_interval_seconds,
            testnet: use_testnet || self.testnet,
            recover_brackets_on_startup: self.recover_brackets_on_startup,
        };
        out.apply_testnet_fixup();
        out
    }

    /// Default config-file location: `$HOME/.claude_trader/live_config.json`.
    pub fn default_config_path() -> PathBuf {
        let home = std::env::var("HOME")
            .ok()
            .or_else(|| std::env::var("USERPROFILE").ok())
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));
        home.join(".claude_trader").join("live_config.json")
    }

    /// Three-tier resolution mirroring Python `LiveConfig.load`:
    ///   1. explicit `path` (errors if missing),
    ///   2. env vars (when both `BINANCE_API_KEY` and `BINANCE_API_SECRET` set),
    ///   3. `~/.claude_trader/live_config.json`.
    /// Otherwise returns `ConfigError::NoCredentials`.
    pub fn load(path: Option<&Path>) -> Result<Self, ConfigError> {
        if let Some(p) = path {
            if !p.exists() {
                return Err(ConfigError::NotFound(p.to_path_buf()));
            }
            return Self::load_from_json(p);
        }

        if let Some(cfg) = Self::load_from_env(|name| std::env::var(name).ok())? {
            return Ok(cfg);
        }

        let default = Self::default_config_path();
        if default.exists() {
            return Self::load_from_json(&default);
        }
        Err(ConfigError::NoCredentials(default))
    }

    /// Parse a JSON config file. Honors `deny_unknown_fields`, applies testnet
    /// fixup, and validates.
    pub fn load_from_json(path: &Path) -> Result<Self, ConfigError> {
        let raw = std::fs::read_to_string(path).map_err(|source| ConfigError::Io {
            path: path.to_path_buf(),
            source,
        })?;
        let mut cfg: Self =
            serde_json::from_str(&raw).map_err(|source| ConfigError::Parse {
                path: path.to_path_buf(),
                source,
            })?;
        cfg.apply_testnet_fixup();
        cfg.validate().map_err(ConfigError::Invalid)?;
        Ok(cfg)
    }

    /// Parse env vars via a caller-supplied lookup. Returns `Ok(None)` when
    /// API key/secret aren't both set (caller falls through to file path).
    /// Emits `ConfigError::DeprecatedEnv` if any removed env var is set.
    pub fn load_from_env<F>(env_lookup: F) -> Result<Option<Self>, ConfigError>
    where
        F: Fn(&str) -> Option<String>,
    {
        // Always check for removed env vars even if API_KEY isn't set — an
        // operator who sets BINANCE_MAX_HOLDING_HOURS expects engine behaviour
        // to honour it, and silently ignoring it would corrupt their intent.
        for &name in REMOVED_ENV_VARS {
            if env_lookup(name).is_some() {
                return Err(ConfigError::DeprecatedEnv(
                    name.to_string(),
                    "max_holding_hours belongs in the signal generator.".to_string(),
                ));
            }
        }

        let api_key = match env_lookup(ENV_API_KEY) {
            Some(v) if !v.is_empty() => v,
            _ => return Ok(None),
        };
        let api_secret = match env_lookup(ENV_API_SECRET) {
            Some(v) if !v.is_empty() => v,
            _ => return Ok(None),
        };

        let testnet = parse_bool_env(env_lookup(ENV_TESTNET).as_deref());
        let base_url = env_lookup(ENV_BASE_URL).unwrap_or_else(|| {
            if testnet {
                TESTNET_BASE_URL.to_string()
            } else {
                PROD_BASE_URL.to_string()
            }
        });
        let position_size_usdt = parse_env_f64(
            ENV_POSITION_SIZE,
            env_lookup(ENV_POSITION_SIZE).as_deref(),
            default_position_size_usdt(),
        )?;
        let max_concurrent_positions = parse_env_usize(
            ENV_MAX_POSITIONS,
            env_lookup(ENV_MAX_POSITIONS).as_deref(),
            default_max_concurrent_positions(),
        )?;
        let order_check_interval_seconds = parse_env_f64(
            ENV_ORDER_CHECK_INTERVAL,
            env_lookup(ENV_ORDER_CHECK_INTERVAL).as_deref(),
            default_order_check_interval(),
        )?;

        let mut cfg = Self {
            api_key,
            api_secret,
            base_url,
            position_size_usdt,
            max_concurrent_positions,
            order_check_interval_seconds,
            testnet,
            recover_brackets_on_startup: default_recover_brackets_on_startup(),
        };
        cfg.apply_testnet_fixup();
        cfg.validate().map_err(ConfigError::Invalid)?;
        Ok(Some(cfg))
    }
}

// ---------------------------------------------------------------------------
// Env-var parsing helpers
// ---------------------------------------------------------------------------

fn parse_bool_env(raw: Option<&str>) -> bool {
    matches!(
        raw.map(str::to_ascii_lowercase).as_deref(),
        Some("1" | "true" | "yes")
    )
}

fn parse_env_f64(name: &str, raw: Option<&str>, default: f64) -> Result<f64, ConfigError> {
    let Some(s) = raw else {
        return Ok(default);
    };
    s.parse::<f64>().map_err(|_| ConfigError::BadEnv {
        name: name.to_string(),
        value: s.to_string(),
        kind: "f64",
    })
}

fn parse_env_usize(
    name: &str,
    raw: Option<&str>,
    default: usize,
) -> Result<usize, ConfigError> {
    let Some(s) = raw else {
        return Ok(default);
    };
    s.parse::<usize>().map_err(|_| ConfigError::BadEnv {
        name: name.to_string(),
        value: s.to_string(),
        kind: "usize",
    })
}

#[cfg(test)]
mod config_tests {
    use super::*;
    use std::collections::HashMap;
    use std::io::Write;

    fn env_from<'a>(
        map: &'a HashMap<&'a str, &'a str>,
    ) -> impl Fn(&str) -> Option<String> + 'a {
        move |name: &str| map.get(name).map(|s| s.to_string())
    }

    #[test]
    fn env_loader_returns_none_without_credentials() {
        let env: HashMap<&str, &str> = HashMap::new();
        let cfg = LiveConfig::load_from_env(env_from(&env)).unwrap();
        assert!(cfg.is_none());
    }

    #[test]
    fn env_loader_picks_up_credentials_and_defaults() {
        let env: HashMap<&str, &str> = [(ENV_API_KEY, "k"), (ENV_API_SECRET, "s")]
            .iter()
            .copied()
            .collect();
        let cfg = LiveConfig::load_from_env(env_from(&env)).unwrap().unwrap();
        assert_eq!(cfg.api_key, "k");
        assert_eq!(cfg.api_secret, "s");
        assert_eq!(cfg.base_url, PROD_BASE_URL);
        assert!(!cfg.testnet);
        assert!(!cfg.is_testnet());
        assert!(cfg.recover_brackets_on_startup);
    }

    #[test]
    fn env_loader_applies_testnet_fixup() {
        let env: HashMap<&str, &str> = [
            (ENV_API_KEY, "k"),
            (ENV_API_SECRET, "s"),
            (ENV_TESTNET, "true"),
        ]
        .iter()
        .copied()
        .collect();
        let cfg = LiveConfig::load_from_env(env_from(&env)).unwrap().unwrap();
        assert_eq!(cfg.base_url, TESTNET_BASE_URL);
        assert!(cfg.is_testnet());
    }

    #[test]
    fn env_loader_respects_explicit_base_url() {
        let env: HashMap<&str, &str> = [
            (ENV_API_KEY, "k"),
            (ENV_API_SECRET, "s"),
            (ENV_TESTNET, "true"),
            (ENV_BASE_URL, LEGACY_TESTNET_BASE_URL),
        ]
        .iter()
        .copied()
        .collect();
        let cfg = LiveConfig::load_from_env(env_from(&env)).unwrap().unwrap();
        assert_eq!(cfg.base_url, LEGACY_TESTNET_BASE_URL);
        assert!(cfg.is_testnet()); // legacy URL still counts
    }

    /// Trailing slash on `BINANCE_BASE_URL` must be normalized at load time
    /// so `BinanceFuturesClient`'s `format!("{base}{path}")` doesn't produce
    /// `https://host//fapi/...` while the market client (which trims) hits
    /// `https://host/fapi/...`. Without normalization, signed and unsigned
    /// requests target different paths and `is_testnet()` returns false
    /// for `https://demo-fapi.binance.com/`.
    #[test]
    fn env_loader_normalizes_trailing_slash_on_base_url() {
        let env: HashMap<&str, &str> = [
            (ENV_API_KEY, "k"),
            (ENV_API_SECRET, "s"),
            (ENV_BASE_URL, "https://demo-fapi.binance.com/"),
        ]
        .iter()
        .copied()
        .collect();
        let cfg = LiveConfig::load_from_env(env_from(&env)).unwrap().unwrap();
        assert_eq!(cfg.base_url, "https://demo-fapi.binance.com");
        assert!(cfg.is_testnet());
    }

    #[test]
    fn env_loader_normalizes_multiple_trailing_slashes() {
        let env: HashMap<&str, &str> = [
            (ENV_API_KEY, "k"),
            (ENV_API_SECRET, "s"),
            (ENV_BASE_URL, "https://demo-fapi.binance.com///"),
        ]
        .iter()
        .copied()
        .collect();
        let cfg = LiveConfig::load_from_env(env_from(&env)).unwrap().unwrap();
        assert_eq!(cfg.base_url, "https://demo-fapi.binance.com");
        assert!(cfg.is_testnet());
    }

    #[test]
    fn json_loader_normalizes_trailing_slash_on_base_url() {
        let mut tmp = std::env::temp_dir();
        let pid = std::process::id();
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        tmp.push(format!("live_config_norm_{pid}_{nonce}.json"));
        let body = r#"{
            "api_key": "k",
            "api_secret": "s",
            "base_url": "https://demo-fapi.binance.com/"
        }"#;
        let mut f = std::fs::File::create(&tmp).unwrap();
        f.write_all(body.as_bytes()).unwrap();
        drop(f);
        let cfg = LiveConfig::load_from_json(&tmp).unwrap();
        assert_eq!(cfg.base_url, "https://demo-fapi.binance.com");
        assert!(cfg.is_testnet());
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn with_overrides_preserves_normalization() {
        let env: HashMap<&str, &str> = [
            (ENV_API_KEY, "k"),
            (ENV_API_SECRET, "s"),
            (ENV_BASE_URL, "https://example.com/api/"),
        ]
        .iter()
        .copied()
        .collect();
        let cfg = LiveConfig::load_from_env(env_from(&env)).unwrap().unwrap();
        assert_eq!(cfg.base_url, "https://example.com/api");
        // Override without flipping testnet — base_url should stay normalized.
        let out = cfg.with_overrides(false, Some(50.0), None);
        assert_eq!(out.base_url, "https://example.com/api");
    }

    #[test]
    fn env_loader_rejects_deprecated_env() {
        let env: HashMap<&str, &str> = [("BINANCE_MAX_HOLDING_HOURS", "48")]
            .iter()
            .copied()
            .collect();
        let err = LiveConfig::load_from_env(env_from(&env)).unwrap_err();
        assert!(matches!(err, ConfigError::DeprecatedEnv(ref n, _) if n == "BINANCE_MAX_HOLDING_HOURS"));
    }

    #[test]
    fn env_loader_rejects_bad_numeric() {
        let env: HashMap<&str, &str> = [
            (ENV_API_KEY, "k"),
            (ENV_API_SECRET, "s"),
            (ENV_POSITION_SIZE, "not-a-number"),
        ]
        .iter()
        .copied()
        .collect();
        let err = LiveConfig::load_from_env(env_from(&env)).unwrap_err();
        assert!(matches!(err, ConfigError::BadEnv { ref name, .. } if name == ENV_POSITION_SIZE));
    }

    #[test]
    fn json_loader_round_trip() {
        let mut tmp = tempfile_with_extension("json");
        let body = r#"{
            "api_key": "k",
            "api_secret": "s",
            "position_size_usdt": 250.0,
            "max_concurrent_positions": 5,
            "testnet": true
        }"#;
        tmp.as_file_mut().write_all(body.as_bytes()).unwrap();
        let cfg = LiveConfig::load_from_json(tmp.path()).unwrap();
        assert_eq!(cfg.api_key, "k");
        assert_eq!(cfg.position_size_usdt, 250.0);
        assert_eq!(cfg.max_concurrent_positions, 5);
        assert_eq!(cfg.base_url, TESTNET_BASE_URL); // testnet fixup applied
        assert!(cfg.recover_brackets_on_startup);
    }

    #[test]
    fn json_loader_rejects_unknown_keys() {
        let mut tmp = tempfile_with_extension("json");
        let body = r#"{
            "api_key": "k",
            "api_secret": "s",
            "BINANCE_MAX_HOLDING_HOURS": 48
        }"#;
        tmp.as_file_mut().write_all(body.as_bytes()).unwrap();
        let err = LiveConfig::load_from_json(tmp.path()).unwrap_err();
        // serde gives "unknown field" via Parse, not Invalid.
        assert!(matches!(err, ConfigError::Parse { .. }));
    }

    #[test]
    fn json_loader_validates() {
        let mut tmp = tempfile_with_extension("json");
        let body = r#"{
            "api_key": "k",
            "api_secret": "s",
            "position_size_usdt": -1.0
        }"#;
        tmp.as_file_mut().write_all(body.as_bytes()).unwrap();
        let err = LiveConfig::load_from_json(tmp.path()).unwrap_err();
        assert!(matches!(err, ConfigError::Invalid(_)));
    }

    #[test]
    fn with_overrides_applies_use_testnet() {
        let cfg = LiveConfig {
            api_key: "k".into(),
            api_secret: "s".into(),
            base_url: PROD_BASE_URL.into(),
            position_size_usdt: 100.0,
            max_concurrent_positions: 3,
            order_check_interval_seconds: 5.0,
            testnet: false,
            recover_brackets_on_startup: true,
        };
        let with = cfg.with_overrides(true, Some(50.0), Some(2));
        assert_eq!(with.base_url, TESTNET_BASE_URL);
        assert!(with.testnet);
        assert_eq!(with.position_size_usdt, 50.0);
        assert_eq!(with.max_concurrent_positions, 2);
        // Preserved
        assert_eq!(with.order_check_interval_seconds, 5.0);
        assert!(with.recover_brackets_on_startup);
    }

    #[test]
    fn with_overrides_preserves_explicit_base_url_when_no_testnet() {
        let cfg = LiveConfig {
            api_key: "k".into(),
            api_secret: "s".into(),
            base_url: LEGACY_TESTNET_BASE_URL.into(),
            position_size_usdt: 100.0,
            max_concurrent_positions: 3,
            order_check_interval_seconds: 5.0,
            testnet: true,
            recover_brackets_on_startup: true,
        };
        let with = cfg.with_overrides(false, None, None);
        // use_testnet=false: keep operator's chosen URL.
        assert_eq!(with.base_url, LEGACY_TESTNET_BASE_URL);
        assert!(with.testnet);
    }

    // tempfile-style helper without pulling the tempfile crate
    struct TmpFile {
        path: PathBuf,
        file: std::fs::File,
    }
    impl TmpFile {
        fn path(&self) -> &Path {
            &self.path
        }
        fn as_file_mut(&mut self) -> &mut std::fs::File {
            &mut self.file
        }
    }
    impl Drop for TmpFile {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(&self.path);
        }
    }

    fn tempfile_with_extension(ext: &str) -> TmpFile {
        let dir = std::env::temp_dir();
        let pid = std::process::id();
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = dir.join(format!("ct_live_cfg_{pid}_{nonce}.{ext}"));
        let file = std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .read(true)
            .open(&path)
            .unwrap();
        TmpFile { path, file }
    }
}
