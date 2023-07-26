#[derive(Debug, clap::Parser)]
#[command(
    author = "bczhc <bczhc0@126.com>",
    about = "Some Bitcoin-related tools\n\
Note: For security considerations, this tool is supposed to be run on a cold wallet side"
)]
pub struct Args {
    #[command(subcommand)]
    pub command: Subcommands,
}

#[derive(Debug, clap::Subcommand)]
pub enum Subcommands {
    #[command(alias = "ga")]
    GetAddress(GetAddressArgs),
    #[command(alias = "gvw")]
    GenerateVanityWallet(GenerateVanityWalletArgs),
}

#[derive(Debug, clap::Args)]
pub struct GetAddressArgs {
    #[arg(help = "Private key in WIF format")]
    pub private_key: String,
}

#[derive(Debug, clap::Args)]
pub struct GenerateVanityWalletArgs {
    #[arg()]
    pub substring: String,
}
