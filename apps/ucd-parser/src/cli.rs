use clap::{Arg, Command};

pub fn build_cli() -> Command {
    Command::new("ucd-parser").arg(Arg::new("output").required(true))
}
