use clap::{value_parser, Arg, Command, ValueHint};

pub fn build_cli() -> Command {
    Command::new("send-email")
        .arg(
            Arg::new("config")
                .help("Configuration file")
                .short('c')
                .long("config")
                .value_hint(ValueHint::FilePath)
                .required(true),
        )
        .arg(Arg::new("to").short('o').long("to").required(true))
        .arg(
            Arg::new("subject")
                .short('s')
                .long("subject")
                .required(false),
        )
        .arg(Arg::new("message").required(true))
        .arg(
            Arg::new("timeout")
                .short('t')
                .long("timeout")
                .help("Timeout for sending email (in milliseconds)")
                .value_parser(value_parser!(u32)),
        )
}
