/// A simple file-based deduplication tool using CoW semantics (reflink)
#[derive(clap::Parser, Debug)]
#[command(author, version, about)]
pub struct CliConfig {
    #[command(subcommand)]
    pub command: Subcommands,
}

#[derive(clap::Subcommand, Debug)]
pub enum Subcommands {
    /// List duplicated files
    #[command(alias = "g")]
    Group(GroupArgs),
    /// Do file deduplication
    #[command(alias = "d")]
    Dedupe(DedupeArgs),
}

#[derive(clap::Args, Debug)]
pub struct GroupArgs {
    /// Don't do anything; just print the size of duplicated files
    #[arg(short, long)]
    pub dry_run: bool,
    #[command(flatten)]
    pub common: CommonArgs,
}

#[derive(clap::Args, Debug)]
pub struct DedupeArgs {
    #[command(flatten)]
    pub common: CommonArgs,
}

#[derive(clap::Args, Debug)]
pub struct CommonArgs {
    /// Minimum size filter
    #[arg(short, long)]
    pub min_size: Option<u64>,
    /// Paths
    #[arg(required = true)]
    pub path: Vec<String>,
    /// Hashing algorithm to be used
    #[arg(long)]
    pub hash_fn: HashFn,
}

#[derive(clap::ValueEnum, Debug, Clone, Copy)]
pub enum HashFn {
    B3_256,
    B3_512,
    B3_1024,
    Sha256,
    Sha512,
}
