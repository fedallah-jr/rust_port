//! Scaffolder for new research experiments.
//!
//! Usage: cargo run -p ct-scaffold -- <name>
//!
//! Creates: rust_port/research/<name>/{Cargo.toml, src/lib.rs, src/main.rs, results.tsv}

use std::fs;
use std::path::PathBuf;

use claude_trader_research_runtime::output::RESULTS_TSV_HEADER;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 || args[1] == "--help" || args[1] == "-h" {
        eprintln!("Usage: ct-scaffold <experiment_name>");
        eprintln!();
        eprintln!("Creates a new research experiment under rust_port/research/<name>/");
        std::process::exit(if args.len() != 2 { 2 } else { 0 });
    }

    let name = &args[1];

    // Validate name: alphanumeric + underscores
    if !name.chars().all(|c| c.is_alphanumeric() || c == '_') {
        eprintln!("Error: name must be alphanumeric with underscores only");
        std::process::exit(1);
    }

    // Find research directory
    let cwd = std::env::current_dir().expect("cannot get cwd");
    let research_dir = find_research_dir(&cwd).unwrap_or_else(|| {
        eprintln!("Error: cannot find rust_port/research/ from cwd");
        std::process::exit(1);
    });

    let experiment_dir = research_dir.join(name);
    if experiment_dir.exists() {
        eprintln!("Error: {} already exists", experiment_dir.display());
        std::process::exit(1);
    }

    let src_dir = experiment_dir.join("src");
    fs::create_dir_all(&src_dir).expect("cannot create directories");

    // Crate name: ct-research-<name_with_hyphens>
    let crate_name = format!("ct-research-{}", name.replace('_', "-"));

    // Cargo.toml
    let cargo_toml = format!(
        r#"[package]
name = "{crate_name}"
version.workspace = true
edition.workspace = true

[[bin]]
name = "{crate_name}"
path = "src/main.rs"

[dependencies]
claude-trader-research-runtime.workspace = true
claude-trader-models.workspace = true
claude-trader-indicators.workspace = true
chrono.workspace = true
serde_json.workspace = true
"#
    );
    fs::write(experiment_dir.join("Cargo.toml"), cargo_toml).expect("write Cargo.toml");

    // src/lib.rs
    let struct_name = to_pascal_case(name);
    let lib_rs = format!(
        r#"//! {name} — research experiment.
//!
//! Edit this file to implement your strategy logic.

use std::collections::{{BTreeMap, HashMap}};

use chrono::{{DateTime, Utc}};
use claude_trader_indicators::{{compute_indicators, OhlcvFrame}};
use claude_trader_models::{{
    Candle, ContextKey, ContextMap, CooldownSpec, HtfData, MarketType, PositionType, Signal,
}};
use claude_trader_research_runtime::ResearchStrategy;

pub struct {struct_name};

// ---------------------------------------------------------------------------
// Configuration — edit these
// ---------------------------------------------------------------------------

const SYMBOLS: &[&str] = &["BTCUSDT", "ETHUSDT", "SOLUSDT"];
const INDICATOR_COLUMNS: &[&str] = &["rsi_14", "atr_14", "ema_20"];
const ANALYSIS_INTERVAL: &str = "1h";

// ---------------------------------------------------------------------------
// Strategy implementation
// ---------------------------------------------------------------------------

impl ResearchStrategy for {struct_name} {{
    fn name(&self) -> &str {{
        "{name}"
    }}

    // Human-readable hypothesis / what changed vs. the previous version.
    // Written to the `strategy_description` column of results.tsv on every
    // run. Keep it concise — one sentence that captures the idea.
    fn description(&self) -> String {{
        "{name} — describe the hypothesis here".to_string()
    }}

    fn symbols(&self) -> Vec<String> {{
        SYMBOLS.iter().map(|s| s.to_string()).collect()
    }}

    fn indicator_columns(&self) -> &[&str] {{
        INDICATOR_COLUMNS
    }}

    // The runtime enforces cooldown globally — emit every candidate and let
    // the runtime filter. Swap in CooldownSpec::symbol, ::symbol_pattern,
    // or CooldownKey::custom(...) if you need a different grouping.
    fn cooldown_spec(&self, signal: &Signal) -> CooldownSpec {{
        CooldownSpec::symbol_side(signal, 12.0)
    }}

    fn analysis_interval(&self) -> &str {{
        ANALYSIS_INTERVAL
    }}

    // Declare point-in-time context dependencies. The runtime auto-derives
    // fetch requirements and warmup from this list. Read values via
    // ctx.context_at(&key, t). Examples:
    //   vec![ContextKey::BtcStructure]
    //   vec![ContextKey::KeyLevels("BTCUSDT".into())]
    //   vec![ContextKey::Funding("BTCUSDT".into())]
    fn required_context(&self) -> Vec<ContextKey> {{
        Vec::new()
    }}

    // Override additional_intervals() and indicator_columns_per_interval()
    // if the strategy uses multi-timeframe data. The runtime pre-computes
    // indicators on additional intervals and delivers them via
    // htf.additional_indicators.

    fn generate_signals(
        &self,
        candles: &BTreeMap<String, &[Candle]>,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        _active_params: &HashMap<String, serde_json::Value>,
        _ctx: &ContextMap,
        _htf: &HtfData,
    ) -> Vec<Signal> {{
        let warmup = claude_trader_indicators::required_warmup(INDICATOR_COLUMNS);
        let mut signals = Vec::new();

        for (symbol, &symbol_candles) in candles {{
            if symbol_candles.len() < warmup + 3 {{
                continue;
            }}

            let ohlcv = OhlcvFrame {{
                open: symbol_candles.iter().map(|c| c.open).collect(),
                high: symbol_candles.iter().map(|c| c.high).collect(),
                low: symbol_candles.iter().map(|c| c.low).collect(),
                close: symbol_candles.iter().map(|c| c.close).collect(),
                volume: symbol_candles.iter().map(|c| c.volume).collect(),
                taker_buy_volume: symbol_candles.iter().map(|c| c.taker_buy_volume).collect(),
            }};
            let ind = compute_indicators(&ohlcv, INDICATOR_COLUMNS)
                .expect("invalid INDICATOR_COLUMNS");

            let get = |col: &str, i: usize, default: f64| -> f64 {{
                ind.get(col)
                    .and_then(|v| v.get(i).copied())
                    .map(|v| if v.is_nan() {{ default }} else {{ v }})
                    .unwrap_or(default)
            }};

            for i in warmup.max(3)..symbol_candles.len() {{
                let close_time = symbol_candles[i].close_time;
                if close_time < start || close_time >= end {{
                    continue;
                }}

                // -----------------------------------------------------------
                // YOUR STRATEGY LOGIC HERE
                // -----------------------------------------------------------
                let rsi = get("rsi_14", i, 50.0);

                if rsi < 30.0 {{
                    let mut metadata = HashMap::new();
                    metadata.insert("rsi".to_string(), serde_json::json!(format!("{{rsi:.1}}")));

                    signals.push(Signal {{
                        signal_date: close_time,
                        position_type: PositionType::Long,
                        ticker: symbol.clone(),
                        pattern: "rsi_oversold".to_string(),
                        tp_pct: Some(3.0),
                        sl_pct: Some(1.5),
                        tp_price: None,
                        sl_price: None,
                        leverage: 1.0,
                        market_type: MarketType::Futures,
                        taker_fee_rate: 0.0005,
                        entry_price: None,
                        fill_timeout_seconds: 3600,
                        entry_delay_seconds: None,
                        max_holding_hours: 72,
                        size_multiplier: 1.0,
                        metadata,
                    }});
                }}
            }}
        }}

        signals.sort_by_key(|s| s.signal_date);
        signals
    }}
}}
"#
    );
    fs::write(src_dir.join("lib.rs"), lib_rs).expect("write lib.rs");

    // src/main.rs
    let mod_name = format!("ct_research_{}", name.replace('-', "_"));
    let main_rs = format!(
        r#"use {mod_name}::{struct_name};
use claude_trader_research_runtime::{{parse_run_config, run_evaluation}};

fn main() {{
    let config = match parse_run_config(std::env::args()) {{
        Ok(c) => c,
        Err(msg) => {{
            eprintln!("{{msg}}");
            std::process::exit(2);
        }}
    }};

    run_evaluation(&{struct_name}, &config);
}}
"#
    );
    fs::write(src_dir.join("main.rs"), main_rs).expect("write main.rs");

    // results.tsv — just the header; the runtime appends rows per eval run.
    fs::write(
        experiment_dir.join("results.tsv"),
        format!("{RESULTS_TSV_HEADER}\n"),
    )
    .expect("write results.tsv");

    println!("Created experiment: {}", experiment_dir.display());
    println!();
    println!("============================================================");
    println!("  ACTION REQUIRED: register the crate in the workspace");
    println!("============================================================");
    println!("Add this line to the `members` array in rust_port/Cargo.toml");
    println!("(do NOT add it to `default-members` — research crates are");
    println!("excluded from default builds on purpose):");
    println!();
    println!("    \"research/{name}\",");
    println!();
    println!("Until you do, `cargo run -p {crate_name}` will fail.");
    println!("============================================================");
    println!();
    println!("Next steps:");
    println!("  1. Register the crate (see above)");
    println!("  2. Edit src/lib.rs — implement your strategy logic");
    println!("  3. cargo run -p {crate_name} -- eval --windows dev");
    println!("  4. cargo run -p {crate_name} -- validate --windows dev");
}

fn find_research_dir(start: &std::path::Path) -> Option<PathBuf> {
    let mut dir = start.to_path_buf();
    loop {
        let candidate = dir.join("rust_port").join("research");
        if candidate.is_dir() {
            return Some(candidate);
        }
        let candidate = dir.join("research");
        if candidate.is_dir() && dir.join("Cargo.toml").is_file() {
            return Some(candidate);
        }
        if !dir.pop() {
            return None;
        }
    }
}

fn to_pascal_case(s: &str) -> String {
    s.split('_')
        .map(|part| {
            let mut c = part.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().to_string() + &c.as_str().to_lowercase(),
            }
        })
        .collect()
}
