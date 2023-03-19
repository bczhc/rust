use clap::{Arg, ArgAction, Command};

pub fn build_cli() -> Command {
    Command::new("x11-event-monitor").arg(
        Arg::new("json")
            .action(ArgAction::SetTrue)
            .help("Print in JSON Lines format")
            .short('j')
            .long("json")
            .required(false),
    )
}
