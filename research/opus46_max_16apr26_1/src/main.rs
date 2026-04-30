use ct_research_opus46_max_16apr26_1::Opus46Max16apr261;
use claude_trader_research_runtime::{parse_run_config, run_evaluation};

fn main() {
    let config = match parse_run_config(std::env::args()) {
        Ok(c) => c,
        Err(msg) => {
            eprintln!("{msg}");
            std::process::exit(2);
        }
    };

    run_evaluation(&Opus46Max16apr261, &config);
}
