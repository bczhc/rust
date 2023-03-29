use clap::Parser;
use cow_dedupe::cli::{CliConfig, Subcommands};

fn main() -> anyhow::Result<()> {
    let args = CliConfig::parse();

    match args.command {
        Subcommands::Group(args) => {
            cow_dedupe::group::main(&args)?;
        }
        Subcommands::Dedupe(args) => {
            cow_dedupe::dedupe::main(&args)?;
        }
    }
    Ok(())
}
