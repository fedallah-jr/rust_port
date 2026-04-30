//! Live runner for the opus46 strategy.
//!
//! Wires `Opus46Live` into `LiveEngine::new_single` using:
//!   - `BinanceFuturesClient` for the signed account/order surface
//!   - `BinanceMarketClient` for unsigned (no-cache) kline fetches
//!   - `LiveConfig::load(...)` to resolve credentials from `--config`, env
//!     vars, or `~/.claude_trader/live_config.json`
//!
//! Refuses to start if credentials are absent. Set `BINANCE_TESTNET=1` to
//! point at the Binance demo-fapi testnet for validation runs before
//! flipping production credentials.

use std::{path::PathBuf, sync::Arc};

use chrono::Utc;
use claude_trader_live::auth_client::BinanceFuturesClient;
use claude_trader_live::engine::LiveEngine;
use claude_trader_live::error::{LiveError, Result};
use claude_trader_live::market_client::BinanceMarketClient;
use claude_trader_live::signal_generator::LiveSignalGenerator;
use claude_trader_live_opus46::Opus46Live;
use claude_trader_models::LiveConfig;

#[derive(Debug)]
enum Cli {
    Run { config_path: Option<PathBuf> },
    Help,
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = parse_cli(std::env::args()).map_err(LiveError::State)?;
    let config_path = match cli {
        Cli::Run { config_path } => config_path,
        Cli::Help => {
            println!("{}", usage());
            return Ok(());
        }
    };

    eprintln!("claude-trader live runner — opus46_max_16apr26_1_v24");

    let config = match LiveConfig::load(config_path.as_deref()) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("\nNo live config found: {e}");
            eprintln!(
                "\nProvide credentials via one of:\n  \
                  - --config /path/to/live_config.json\n  \
                  - BINANCE_API_KEY / BINANCE_API_SECRET environment variables, or\n  \
                  - ~/.claude_trader/live_config.json\n\n\
                Set BINANCE_TESTNET=1 to use the demo-fapi.binance.com testnet."
            );
            std::process::exit(1);
        }
    };
    if let Some(path) = &config_path {
        eprintln!("Config file: {}", path.display());
    }
    eprintln!(
        "Config loaded: base_url={} testnet={} size={} max_pos={} order_check={}s",
        config.base_url,
        config.testnet,
        config.position_size_usdt,
        config.max_concurrent_positions,
        config.order_check_interval_seconds,
    );
    if config.is_testnet() {
        eprintln!("Running against TESTNET — no real money at risk.");
    } else {
        eprintln!("Running against PRODUCTION — orders will be placed with real money.");
    }

    let client = Arc::new(BinanceFuturesClient::new(config.clone())?);
    // Route market data + funding through the same base_url as signed
    // orders so testnet runs use demo-fapi for everything (and any custom
    // BINANCE_BASE_URL is honored on both sides).
    let market = Arc::new(BinanceMarketClient::with_base_url(&config.base_url));
    let strategy: Box<dyn LiveSignalGenerator> = Box::new(Opus46Live::new());

    eprintln!(
        "Engine starting at {} — strategy_id={}",
        Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
        strategy.strategy_id()
    );

    let mut engine = LiveEngine::new_single(config, client, market, strategy)?;
    engine.start()
}

fn usage() -> String {
    "Usage: live-opus46 [--config <path>]\n\nOptions:\n  -c, --config <path>   Load live config from an explicit JSON file\n  -h, --help            Show this help".to_string()
}

fn parse_cli<I>(args: I) -> std::result::Result<Cli, String>
where
    I: IntoIterator<Item = String>,
{
    let mut config_path: Option<PathBuf> = None;
    let mut iter = args.into_iter();
    let _program = iter.next();

    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "-h" | "--help" => return Ok(Cli::Help),
            "-c" | "--config" => {
                let value = iter
                    .next()
                    .ok_or_else(|| format!("missing value for {arg}\n\n{}", usage()))?;
                set_config_path(&mut config_path, value)?;
            }
            _ if arg.starts_with("--config=") => {
                let value = arg
                    .strip_prefix("--config=")
                    .expect("prefix checked")
                    .to_string();
                if value.is_empty() {
                    return Err(format!("missing value for --config\n\n{}", usage()));
                }
                set_config_path(&mut config_path, value)?;
            }
            _ => return Err(format!("unknown argument: {arg}\n\n{}", usage())),
        }
    }

    Ok(Cli::Run { config_path })
}

fn set_config_path(slot: &mut Option<PathBuf>, value: String) -> std::result::Result<(), String> {
    if slot.is_some() {
        return Err(format!("--config supplied more than once\n\n{}", usage()));
    }
    if value.is_empty() {
        return Err(format!("missing value for --config\n\n{}", usage()));
    }
    *slot = Some(PathBuf::from(value));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(args: &[&str]) -> std::result::Result<Cli, String> {
        parse_cli(args.iter().map(|s| s.to_string()))
    }

    #[test]
    fn parse_accepts_config_space_form() {
        match parse(&["live-opus46", "--config", "/tmp/live.json"]).unwrap() {
            Cli::Run { config_path } => {
                assert_eq!(config_path.unwrap(), PathBuf::from("/tmp/live.json"));
            }
            Cli::Help => panic!("expected run"),
        }
    }

    #[test]
    fn parse_accepts_config_equals_form() {
        match parse(&["live-opus46", "--config=/tmp/live.json"]).unwrap() {
            Cli::Run { config_path } => {
                assert_eq!(config_path.unwrap(), PathBuf::from("/tmp/live.json"));
            }
            Cli::Help => panic!("expected run"),
        }
    }

    #[test]
    fn parse_accepts_short_config_form() {
        match parse(&["live-opus46", "-c", "/tmp/live.json"]).unwrap() {
            Cli::Run { config_path } => {
                assert_eq!(config_path.unwrap(), PathBuf::from("/tmp/live.json"));
            }
            Cli::Help => panic!("expected run"),
        }
    }

    #[test]
    fn parse_help_exits_without_config() {
        assert!(matches!(
            parse(&["live-opus46", "--help"]).unwrap(),
            Cli::Help
        ));
    }

    #[test]
    fn parse_rejects_missing_config_value() {
        let err = parse(&["live-opus46", "--config"]).unwrap_err();
        assert!(err.contains("missing value"));
    }
}
