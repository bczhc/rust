use btc_tools::cli::{Args, Subcommands};
use clap::Parser;

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    match args.command {
        Subcommands::GetAddress(a) => btc_tools::get_address::main(a),
        Subcommands::GenerateVanityWallet(a) => btc_tools::vanity_wallet::main(a),
    }
}
