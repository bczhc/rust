use clap::{Arg, Command};

pub fn build_cli() -> Command {
    Command::new("tcp-wrapper")
        .about("A TCP stream wrapper with cryptography")
        .subcommand(
            Command::new("server")
                .arg(Arg::new("address").help("Target address"))
                .arg(Arg::new("port").help("Port to listen on")),
        )
        .subcommand(
            Command::new("client")
                .arg(Arg::new("server").help("Server address"))
                .arg(Arg::new("port").help("Local mapped port to listen on")),
        )
}
