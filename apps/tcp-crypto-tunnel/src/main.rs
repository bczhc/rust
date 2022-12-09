use tcp_crypto_tunnel::cli::build_cli;
use tcp_crypto_tunnel::errors::Result;

fn main() -> Result<()> {
    let command = build_cli();
    let matches = command.get_matches();

    if let Some(matches) = matches.subcommand_matches("server") {
        tcp_crypto_tunnel::server::main(matches)
    } else if let Some(matches) = matches.subcommand_matches("client") {
        tcp_crypto_tunnel::client::main(matches)
    } else {
        unreachable!()
    }
}
