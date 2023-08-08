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
        value_parser = clap::value_parser!(u64).range(1..)
    )]
    pub iter_count: u64,
    #[arg(
        short,
        long,
        help = "Also duplicate stdin to stdout; hash result will be in stderr"
    )]
    pub pipe_input: bool,
}

#[derive(Debug, clap::Subcommand)]
pub enum Subcommand {
    Md5,
    Sha1,
    Sha256,
    Sha512,
    Sha3_256,
    Sha3_512,
    Ripemd128,
    Ripemd160,
    Ripemd256,
    Blake2b512,
    Blake2s256,
    Blake3(XofDigestArgs),
}

#[derive(Debug, clap::Args)]
pub struct XofDigestArgs {
    #[arg(short, long, help = "Output length in bytes", default_value = "32")]
    pub length: usize,
}
