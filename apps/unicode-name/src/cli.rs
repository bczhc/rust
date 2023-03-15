use clap::{Arg, ArgAction, Command};

pub fn build_cli() -> Command {
    Command::new("unicode-name")
        .arg(Arg::new("text").required(false))
        .arg(
            Arg::new("grapheme")
                .action(ArgAction::SetTrue)
                .short('g')
                .long("grapheme"),
        )
        .arg(
            Arg::new("ucd db")
                .action(ArgAction::Set)
                .short('d')
                .long("database")
                .required(false)
                .default_value("/usr/local/share/ucd.db"),
        )
}
