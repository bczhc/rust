use bczhc_lib::mutex_lock;
use tcp_crypto_tunnel::cli::build_cli;
use tcp_crypto_tunnel::errors::Result;
use tcp_crypto_tunnel::{derive_key, prompt_password, KEY};

fn main() -> Result<()> {
    let command = build_cli();
    let matches = command.get_matches();

    let password = prompt_password()?;
    let key = derive_key::<32>(&password);
    unsafe { KEY.copy_from_slice(&key); }

    if let Some(matches) = matches.subcommand_matches("server") {
        tcp_crypto_tunnel::server::main(matches)
    } else if let Some(matches) = matches.subcommand_matches("client") {
        tcp_crypto_tunnel::client::main(matches)
    } else {
        unreachable!()
    }
}
