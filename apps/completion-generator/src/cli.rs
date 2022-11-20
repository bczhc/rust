use clap::{value_parser, Arg, Command};
use clap_complete::Shell;

use crate::BIN_NAMES;

pub fn build_cli() -> Command {
    Command::new("generate-completions")
        .arg(Arg::new("app-name").required(true).value_parser(*BIN_NAMES))
        .arg(
            Arg::new("shell")
                .value_parser(value_parser!(Shell))
                .required(true),
        )
}
