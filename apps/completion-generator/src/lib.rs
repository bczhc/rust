use std::io::stdout;

use clap::Command;
use clap_complete::Generator;
use once_cell::sync::Lazy;

pub mod cli;

pub static CLI_BUILDERS: [fn() -> Command; 1] = [archive::build_cli];

pub static BIN_NAMES: Lazy<[&str; 1]> = Lazy::new(|| {
    CLI_BUILDERS.map(|x| Box::leak(String::from(x().get_name()).into_boxed_str()) as &'static str)
});

pub fn print_completions<G: Generator>(generator: G, mut command: Command) {
    let bin_name = String::from(command.get_name());
    clap_complete::generate(generator, &mut command, bin_name, &mut stdout());
}
