use clap::{Arg, ArgAction, Command, ValueHint};

pub fn build_cli() -> Command {
    Command::new("cow-dedupe")
        .about("A simple file-based deduplication tool using CoW semantic (reflink)")
        .arg(
            Arg::new("path")
                .required(true)
                .action(ArgAction::Append)
                .value_hint(ValueHint::DirPath)
                .help("Path to directory"),
        )
        .arg(
            Arg::new("min-size")
                .long("min-size")
                .short('m')
                .help("Minimum size filter")
                .action(ArgAction::Set)
                .required(false),
        )
        .arg(
            Arg::new("dry-run")
                .long("dry-run")
                .short('d')
                .help("Don't do reflinking; just print the size of duplicated files")
                .required(false)
                .action(ArgAction::SetTrue),
        )
}
