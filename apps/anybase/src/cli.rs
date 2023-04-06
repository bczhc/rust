use clap::value_parser;

/// Convert data base, from stdin to stdout
#[derive(clap::Parser, Debug)]
pub struct CliConfig {
    /// Output base
    #[arg(value_parser = value_parser!(u16).range(2..=256))]
    pub base: u16,
    /// Map result bytes to characters according to this table
    /// Note the length of this string should be equal to `base`
    #[arg(short = 't', long)]
    pub alphabet_table: Option<String>,
    /// Decode mode
    #[arg(short, long)]
    pub decode: bool,
}
