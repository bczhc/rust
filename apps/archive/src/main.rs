use clap::{Arg, ArgAction, Command};

fn main() -> Result<(), String> {
    let matches = Command::new("archive")
        .version("1.0.0")
        .subcommand(
            Command::new("create")
                .alias("c")
                .arg(Arg::new("output").required(true))
                .arg(Arg::new("path").action(ArgAction::Append).required(true))
                .arg(
                    Arg::new("base-dir")
                        .short('C')
                        .long("base-directory")
                        .default_value("."),
                )
                .arg(
                    Arg::new("compress")
                        .short('c')
                        .long("compress")
                        .value_name("method")
                        .value_parser(["gzip", "xz", "zstd", "bzip2", "no"])
                        .help("Compression method used for each file, \"no\" means no compression")
                        .default_value("no")
                        .ignore_case(true)
                        .conflicts_with("data-filter-cmd"),
                )
                .arg(
                    Arg::new("level")
                        .short('l')
                        .long("level")
                        .help("Compression level")
                        .default_value("best")
                        .conflicts_with("data-filter-cmd"),
                )
                .arg(
                    Arg::new("data-filter-cmd")
                        .long("data-filter")
                        .action(ArgAction::Append)
                        .num_args(1..)
                        .allow_hyphen_values(true)
                        .help(
                            "External data filter command line (for compression and decompression)",
                        ),
                ),
        )
        .subcommand(
            Command::new("list")
                .alias("l")
                .arg(Arg::new("archive").help("Archive file path").required(true)),
        )
        .subcommand(
            Command::new("extract")
                .alias("e")
                .arg(Arg::new("archive").help("Archive file path").required(true))
                .arg(
                    Arg::new("base-dir")
                        .short('C')
                        .long("base-directory")
                        .default_value("."),
                )
                .arg(
                    Arg::new("data-filter-cmd")
                        .long("data-filter")
                        .action(ArgAction::Append)
                        .num_args(1..)
                        .allow_hyphen_values(true)
                        .help(
                            "External data filter command line (for compression and decompression)",
                        ),
                ),
        )
        .subcommand(
            Command::new("test")
                .alias("t")
                .arg(Arg::new("archive").help("Archive file path").required(true))
                .about("Check the archive integrity"),
        )
        .subcommand_required(true)
        .about("An archive format for data backups with indexing and compression capabilities")
        .get_matches();

    let result = if let Some(matches) = matches.subcommand_matches("create") {
        archive::create::main(matches)
    } else if let Some(matches) = matches.subcommand_matches("list") {
        archive::list::main(matches)
    } else if let Some(matches) = matches.subcommand_matches("extract") {
        archive::extract::main(matches)
    } else if let Some(matches) = matches.subcommand_matches("test") {
        archive::test::main(matches)
    } else {
        unreachable!()
    };
    if let Err(e) = result {
        return Err(e.to_string());
    }

    Ok(())
}
