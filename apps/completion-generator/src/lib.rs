use std::io::stdout;

use clap::Command;
use clap_complete::Generator;
use once_cell::sync::Lazy;

macro_rules! count {
    () => (0_usize);
    ($x:expr) => (1_usize);
    ( $x:expr, $($xs:expr),* ) => (1usize + count!($($xs),*));
}

macro_rules! cli_builders {
    ( $($f:expr),+ $(,)? ) => {
        pub const CLI_BUILDERS: [CliBuilderFn; count![$($f),*]] = [$($f),*];
    };
}

pub mod cli;

type CliBuilderFn = fn() -> Command;

/// Can only define self-cli-name via this. Because
/// when using `crate::cli::build_cli().get_name()`, this
/// function needs "self-cli-name" field, which causes
/// a loop.
const SELF_CRATE_NAME: &str = env!("CARGO_PKG_NAME");

use clap::CommandFactory;
cli_builders![
    archive::build_cli,
    fourier_series::cli::build_cli,
    cow_dedupe::cli::CliConfig::command,
    enc::cli::build_cli,
    send_email::cli::build_cli,
    tcp_crypto_tunnel::cli::build_cli,
    task_notify::cli::build_cli,
    network_monitor::cli::build_cli,
    ucd_parser::cli::build_cli,
    x11_event_monitor::cli::build_cli,
    btc_tools::cli::Args::command,
    anybase::cli::CliConfig::command,
    image_stegano_bg::cli::CliArgs::command,
];

const CLI_BUILDERS_LEN: usize = CLI_BUILDERS.len();

pub static BIN_NAMES: Lazy<[&str; CLI_BUILDERS_LEN + 1]> = Lazy::new(|| {
    let app_names = CLI_BUILDERS
        .map(|x| Box::leak(String::from(x().get_name()).into_boxed_str()) as &'static str);
    let mut include_self = [SELF_CRATE_NAME; CLI_BUILDERS_LEN + 1];
    include_self[1..].copy_from_slice(&app_names);
    include_self
});

pub fn print_completions<G: Generator>(generator: G, mut command: Command) {
    let bin_name = String::from(command.get_name());
    clap_complete::generate(generator, &mut command, bin_name, &mut stdout());
}
