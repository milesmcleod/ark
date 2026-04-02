use anyhow::Result;
use clap::CommandFactory;
use clap_complete::generate;

use crate::cli::Cli;

pub fn run(shell: clap_complete::Shell) -> Result<()> {
    generate(shell, &mut Cli::command(), "ark", &mut std::io::stdout());
    Ok(())
}
