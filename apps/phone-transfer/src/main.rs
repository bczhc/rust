use clap::{Arg, ArgGroup, Command};
use phone_transfer::errors::*;

fn main() -> Result<()> {
    let matches = Command::new("phone-transfer")
        .subcommand(
            Command::new("send")
                .alias("s")
                .arg(Arg::new("ip").required(true))
                .arg(Arg::new("port").required(true))
                .arg(
                    Arg::new("file-mode")
                        .short('f')
                        .long("file")
                        .help("file mode"),
                )
                .arg(
                    Arg::new("text-mode")
                        .short('t')
                        .long("text")
                        .help("text mode"),
                )
                .group(
                    ArgGroup::new("pipe-send-mode")
                        .args(&["file-mode", "text-mode"])
                        .required(false)
                        .requires("pipe-mode"),
                )
                .arg(
                    Arg::new("pipe-mode")
                        .short('p')
                        .long("pipe")
                        .help("Read from stdin. This can be used with file-mode and text-mode")
                        .requires("pipe-send-mode")
                        .required(false),
                )
                .arg(
                    Arg::new("file")
                        .required_unless_present("pipe-mode")
                        .conflicts_with("pipe-mode"),
                ),
        )
        .subcommand(
            Command::new("receive")
                .alias("r")
                .arg(
                    Arg::new("output-dir")
                        .short('o')
                        .long("output-dir")
                        .default_value(".")
                        .required(false)
                        .takes_value(true)
                        .help("Output directory (auto create if not exists)"),
                )
                .arg(Arg::new("port").required(true).help("Listen port")),
        )
        .subcommand_required(true)
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("send") {
        return phone_transfer::send::main(matches);
    }
    if let Some(matches) = matches.subcommand_matches("receive") {
        return phone_transfer::receive::main(matches);
    };

    Ok(())
}
