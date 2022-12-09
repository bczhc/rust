use clap::{value_parser, Arg, Command};

pub fn build_cli() -> Command {
    Command::new("tcp-wrapper")
        .about("A TCP stream wrapper with cryptography")
        .subcommand(
            Command::new("server")
                .arg(Arg::new("address").help("Target address").required(true))
                .arg(
                    Arg::new("port")
                        .help("Port to listen on")
                        .value_parser(value_parser!(u16))
                        .required(true),
                ),
        )
        .subcommand(
            Command::new("client")
                .arg(Arg::new("server").help("Server address").required(true))
                .arg(
                    Arg::new("port")
                        .help("Local mapped port to listen on")
                        .value_parser(value_parser!(u16))
                        .required(true),
                ),
        )
        .subcommand_required(true)
}
