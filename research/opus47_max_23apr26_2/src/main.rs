use ct_research_opus47_max_23apr26_2::Opus47Max23apr262;
use claude_trader_research_runtime::{parse_run_config, run_evaluation};

fn main() {
    let config = match parse_run_config(std::env::args()) {
        Ok(c) => c,
        Err(msg) => {
            eprintln!("{msg}");
            std::process::exit(2);
        }
    };

    run_evaluation(&Opus47Max23apr262, &config);
}
