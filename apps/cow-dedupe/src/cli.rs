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

#[derive(clap::Args, Debug, Clone)]
pub struct GroupArgs {
    /// Print the whole hash of each file. Otherwise it will print the only first 20 bytes.
    #[arg(long, default_value = "false")]
    pub full_hash: bool,
    /// Output format
    #[arg(short = 'f', long, default_value = "default")]
    pub output_format: OutputFormat,
    #[command(flatten)]
    pub common: CommonArgs,
}

#[derive(clap::Args, Debug)]
pub struct DedupeArgs {
    /// Don't do anything; just print the size of duplicated files
    #[arg(short, long)]
    pub dry_run: bool,
    #[arg(long, default_value = "yes")]
    pub use_cp_cmd: YesNoChoice,
    #[command(flatten)]
    pub common: CommonArgs,
}

#[derive(clap::Args, Debug, Clone)]
pub struct CommonArgs {
    /// Minimum size filter
    #[arg(short, long, default_value = "1B")]
    pub min_size: String,
    /// Paths
    #[arg(required = true)]
    pub path: Vec<String>,
    /// Hashing algorithm to be used
    #[arg(long, default_value = "b3-512")]
    pub hash_fn: HashFn,
    /// JSON or binary input file
    #[arg(short, long)]
    pub input_file: Option<String>,
}

#[derive(clap::ValueEnum, Debug, Clone, Copy)]
pub enum HashFn {
    B3_128,
    B3_160,
    B3_256,
    B3_512,
    B3_1024,
    B3_2048,
    Sha256,
    Sha512,
    Sha3_256,
    Sha3_512,
}

#[derive(clap::ValueEnum, Debug, Clone, Copy)]
pub enum OutputFormat {
    Default,
    Json,
    Binary,
}

#[derive(clap::ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum YesNoChoice {
    Yes,
    No,
}

impl YesNoChoice {
    #[inline]
    pub fn yes(&self) -> bool {
        *self == Self::Yes
    }

    #[inline]
    pub fn no(&self) -> bool {
        !self.yes()
    }
}
