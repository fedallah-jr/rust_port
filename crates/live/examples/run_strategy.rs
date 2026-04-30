//! Live-trading example runner.
//!
//! Mirrors `live/run.py` in spirit but ships a **demo strategy** by default
//! that returns no signals — running this example will not place any orders
//! unless the operator wires in a real `LiveSignalGenerator`. This is by
//! design: someone running `cargo run --example run_strategy` shouldn't be
//! able to accidentally start trading.
//!
//! Usage:
//!   $ cargo run --example run_strategy
//!     # → reads BINANCE_API_KEY / BINANCE_API_SECRET from env or
//!     #   ~/.claude_trader/live_config.json. If credentials are absent,
//!     #   prints a helpful message and exits 1.
//!     # → runs `DemoStrategy` which emits no signals; the engine reconciles
//!     #   exchange state and polls indefinitely until SIGINT.
//!
//! Replace `DemoStrategy::new()` with a real strategy impl in your own
//! binary, or copy this file as a template.

use std::sync::Arc;

use chrono::{DateTime, Utc};
use claude_trader_live::auth_client::BinanceFuturesClient;
use claude_trader_live::engine::LiveEngine;
use claude_trader_live::error::Result;
use claude_trader_live::market_client::BinanceMarketClient;
use claude_trader_live::signal_generator::{FatalSignalError, LiveSignalGenerator};
use claude_trader_models::{LiveConfig, Signal};

/// No-trade demo strategy. Always returns an empty signal vec, so the engine
/// runs through its full lifecycle (load_state → reconcile → recover_brackets
/// → setup → poll loop) without ever placing an order.
struct DemoStrategy {
    symbols: Vec<String>,
}

impl DemoStrategy {
    fn new() -> Self {
        // Trade nothing — empty symbol set ensures we don't even reconcile
        // against any specific tickers.
        Self { symbols: Vec::new() }
    }
}

impl LiveSignalGenerator for DemoStrategy {
    fn strategy_id(&self) -> &str {
        "demo"
    }
    fn symbols(&self) -> &[String] {
        &self.symbols
    }
    fn analysis_interval(&self) -> &str {
        "1h"
    }
    fn poll(&mut self) -> std::result::Result<Vec<Signal>, FatalSignalError> {
        Ok(Vec::new())
    }
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    eprintln!("claude-trader live runner — demo strategy");
    eprintln!(
        "WARNING: this example uses a no-trade DemoStrategy. To trade for real, \
         replace DemoStrategy::new() with your own LiveSignalGenerator."
    );

    let config = match LiveConfig::load(None) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("\nNo live config found: {e}");
            eprintln!(
                "\nProvide credentials via either:\n  \
                  - BINANCE_API_KEY / BINANCE_API_SECRET environment variables, or\n  \
                  - ~/.claude_trader/live_config.json\n\n\
                Set BINANCE_TESTNET=1 to use the demo-fapi.binance.com testnet."
            );
            std::process::exit(1);
        }
    };
    eprintln!(
        "Config loaded: base_url={} testnet={} size={} max_pos={}",
        config.base_url, config.testnet, config.position_size_usdt, config.max_concurrent_positions,
    );
    if config.is_testnet() {
        eprintln!("Running against TESTNET — no real money at risk.");
    } else {
        eprintln!("Running against PRODUCTION — orders will be placed with real money.");
    }

    let client = Arc::new(BinanceFuturesClient::new(config.clone())?);
    let market = Arc::new(BinanceMarketClient::new());
    let strategy: Box<dyn LiveSignalGenerator> = Box::new(DemoStrategy::new());

    let mut engine = LiveEngine::new_single(config, client, market, strategy)?;
    let _ = preview_now(); // drop a startup time on stderr for log correlation
    engine.start()
}

fn preview_now() -> DateTime<Utc> {
    let now = Utc::now();
    eprintln!("Engine starting at {}", now.format("%Y-%m-%d %H:%M:%S UTC"));
    now
}
