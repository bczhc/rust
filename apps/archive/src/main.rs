use std::io;

use archive::cli::build_cli;
use clap::{value_parser, Arg, ArgAction, Command};
use clap_complete::{Generator, Shell};

fn main() -> Result<(), String> {
    let matches = build_cli().get_matches();

    let result = if let Some(matches) = matches.subcommand_matches("create") {
        archive::create::main(matches)
    } else if let Some(matches) = matches.subcommand_matches("list") {
        archive::list::main(matches)
    } else if let Some(matches) = matches.subcommand_matches("extract") {
        archive::extract::main(matches)
    } else if let Some(matches) = matches.subcommand_matches("test") {
        archive::test::main(matches)
    } else if let Some(matches) = matches.subcommand_matches("info") {
        archive::info::main(matches)
    } else {
        unreachable!()
    };
    if let Err(e) = result {
        return Err(e.to_string());
    }

    Ok(())
}

fn print_completions<G: Generator>(generator: G, cmd: &mut Command) {
    clap_complete::generate(generator, cmd, String::from(cmd.get_name()), &mut io::stdout());
}
