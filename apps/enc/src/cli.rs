use clap::{Arg, ArgAction, Command};

pub fn build_cli() -> Command {
    Command::new("enc")
        .about("AES-256 encryption and decryption")
        .arg(
            Arg::new("decrypt")
                .short('d')
                .long("decrypt")
                .help("Decryption mode")
                .required(false)
                .action(ArgAction::SetTrue),
        )
}
