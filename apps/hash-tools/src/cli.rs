use clap::value_parser;

#[derive(Debug, clap::Parser)]
#[command(about = "A generic hash tool", author = "bczhc <bczhc0@126.com>")]
pub struct Args {
    #[command(subcommand)]
    pub subcommand: Subcommand,
    #[arg(
        short,
        long,
        help = "Output raw bytes instead of hex string",
        default_value = "false"
    )]
    pub raw: bool,
    #[arg(
        short,
        long,
        help = "Iteration count (hash multiple times)",
        default_value = "1",
        value_parser = value_parser!(u64).range(1..)
    )]
    pub iter_count: u64,
}

#[derive(Debug, clap::Subcommand)]
pub enum Subcommand {
    Md5,
    Sha1,
    Sha256,
    Sha512,
    Blake3(Blake3Args),
}

#[derive(Debug, clap::Args)]
pub struct Blake3Args {
    #[command(flatten)]
    pub xof_args: XofDigestArgs,
}

#[derive(Debug, clap::Args)]
pub struct XofDigestArgs {
    #[arg(short, long, help = "Output length in bytes", default_value = "32")]
    pub length: usize,
}
