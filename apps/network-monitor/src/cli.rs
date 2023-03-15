use clap::{value_parser, Arg, ArgAction, Command};

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
        .arg(
            Arg::new("interval")
                .short('d')
                .long("interval")
                .action(ArgAction::Set)
                .default_value("1")
                .value_parser(value_parser!(u32))
                .required(false),
        )
}
