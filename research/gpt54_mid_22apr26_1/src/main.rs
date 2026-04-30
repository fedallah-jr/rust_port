use claude_trader_research_runtime::{parse_run_config, run_evaluation};
use ct_research_gpt54_mid_22apr26_1::Gpt54Mid22apr261;

fn main() {
    let config = match parse_run_config(std::env::args()) {
        Ok(c) => c,
        Err(msg) => {
            eprintln!("{msg}");
            std::process::exit(2);
        }
    };

    run_evaluation(&Gpt54Mid22apr261, &config);
}
