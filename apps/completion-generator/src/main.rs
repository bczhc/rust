use clap_complete::Shell;
use completion_generator::{print_completions, BIN_NAMES, CLI_BUILDERS};

fn main() {
    let matches = completion_generator::cli::build_cli().get_matches();
    let app_name = matches.get_one::<String>("app-name").unwrap();
    let shell = matches.get_one::<Shell>("shell").unwrap().clone();

    let index = BIN_NAMES.iter().position(|x| *x == app_name).unwrap();
    let command = CLI_BUILDERS[index]();

    print_completions(shell, command);
}
