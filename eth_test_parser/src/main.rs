use anyhow::Context;
use arg_parsing::ProgArgs;
use clap::Parser;
use eth_test_parsing::{parse_test_directories, parse_test_directories_forced};
use eth_tests_fetching::update_eth_tests_upstream;
use stale_test_scanning::determine_which_tests_need_reparsing;

mod arg_parsing;
mod eth_test_parsing;
mod eth_tests_fetching;
mod stale_test_scanning;
mod utils;

pub(crate) struct ProgState {
    forced_regen: bool,
}

impl ProgState {
    fn new(p_args: ProgArgs) -> Self {
        Self {
            forced_regen: p_args.force_regen_local,
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let p_args = ProgArgs::try_parse().with_context(|| "Parsing program arguments")?;
    let state = ProgState::new(p_args);

    run(state).await
}

async fn run(state: ProgState) -> anyhow::Result<()> {
    match state.forced_regen {
        false => {
            update_eth_tests_upstream().with_context(|| "Updating the Ethereum test repository")?;

            let tests_needing_reparse = determine_which_tests_need_reparsing()
                .await
                .with_context(|| "Determining which Ethereum tests are stale")?;

            parse_test_directories(tests_needing_reparse)
                .await
                .with_context(|| parse_err_context_msg("Reparsing stale tests"))?;
        }
        true => {
            parse_test_directories_forced()
                .await
                .with_context(|| parse_err_context_msg("forced parsing"))?;
        }
    }

    Ok(())
}

fn parse_err_context_msg(extra_info_str: &str) -> String {
    format!("Parsing tests ({})", extra_info_str)
}
