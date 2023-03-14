use clap::{Arg, ArgAction, Command};

pub fn build_cli() -> Command {
    Command::new("network-monitor")
        .arg(
            Arg::new("interfaces")
                .short('I')
                .long("interfaces")
                .required(false)
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("interface")
                .short('i')
                .long("interface")
                .action(ArgAction::Set)
                .required_unless_present("interfaces"),
        )
}
