use ct_research_opus47_max_20apr26_1::Opus47Max20apr261;
use claude_trader_research_runtime::{parse_run_config, run_evaluation};

fn main() {
    let config = match parse_run_config(std::env::args()) {
        Ok(c) => c,
        Err(msg) => {
            eprintln!("{msg}");
            std::process::exit(2);
        }
    };

    run_evaluation(&Opus47Max20apr261, &config);
}
