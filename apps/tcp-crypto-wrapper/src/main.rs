use tcp_crypto_wrapper::cli::build_cli;
use tcp_crypto_wrapper::errors::Result;

fn main() -> Result<()> {
    let command = build_cli();
    let matches = command.get_matches();

    if let Some(matches) = matches.subcommand_matches("server") {
        tcp_crypto_wrapper::server::main(matches)
    } else if let Some(matches) = matches.subcommand_matches("client") {
        tcp_crypto_wrapper::client::main(matches)
    } else {
        unreachable!()
    }
}
