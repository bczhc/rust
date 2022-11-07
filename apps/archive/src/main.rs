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
                    Arg::new("comment")
                        .short('m')
                        .long("comment")
                        .help("Comment for this archive"),
                )
                .arg(
                    Arg::new("compress")
                        .short('c')
                        .long("compress")
                        .value_name("method")
                        .value_parser(["gzip", "xz", "zstd", "bzip2", "brotli", "none"])
                        .help("Compression method used for each file, \"none\" for not used")
                        .default_value("none")
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
                    Arg::new("paths")
                        .action(ArgAction::Append)
                        .help("File paths to be extracted"),
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
        .subcommand(
            Command::new("info")
                .alias("i")
                .arg(Arg::new("archive").help("Archive file path").required(true))
                .about("Show the information of archive"),
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
    } else if let Some(matches) = matches.subcommand_matches("info") {
        archive::info::main(matches)
    } else {
        unreachable!()
    };
    if let Err(e) = result {
        return Err(e.to_string());
    }

    Ok(())
}
