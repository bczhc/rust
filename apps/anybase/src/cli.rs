use clap::value_parser;

/// Convert data base, from stdin to stdout
#[derive(clap::Parser, Debug)]
pub struct CliConfig {
    /// Input base
    #[arg(short, long, value_parser = value_parser!(u16).range(2..=256), default_value = "256")]
    pub from_base: u16,
    /// Output base
    #[arg(short, long, value_parser = value_parser!(u16).range(2..=256))]
    pub to_base: u16,
    /// Input mapping table; maps bytes to characters
    ///
    /// The length of this string should be equal to `from_base`
    #[arg(long, alias = "ft")]
    pub from_table: Option<String>,
    /// Output mapping table; maps bytes to characters
    ///
    /// The length of this string should be equal to `to_base`
    #[arg(long, alias = "tt")]
    pub to_table: Option<String>,
}
