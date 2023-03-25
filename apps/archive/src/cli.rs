use clap::{Arg, ArgAction, Command, ValueHint};

use crate::{Compression, COMPRESSIONS};

pub fn build_cli() -> Command {
    let compression_names = COMPRESSIONS
        .into_iter()
        .filter(|x| *x != Compression::External)
        .map(|x| Box::leak(String::from(x.as_str()).into_boxed_str()) as &'static str)
        .collect::<Vec<_>>();

    Command::new("archive")
        .version("1.0.0")
        .subcommand(
            Command::new("create")
                .about("Create an archive")
                .alias("c")
                .arg(
                    Arg::new("output")
                        .required(true)
                        .value_hint(ValueHint::FilePath),
                )
                .arg(
                    Arg::new("path")
                        .action(ArgAction::Append)
                        .required(true)
                        .value_hint(ValueHint::AnyPath),
                )
                .arg(
                    Arg::new("base-dir")
                        .short('C')
                        .long("base-directory")
                        .default_value(".")
                        .value_hint(ValueHint::DirPath),
                )
                .arg(
                    Arg::new("comment")
                        .short('m')
                        .long("comment")
                        .help("Comment for this archive")
                        .value_hint(ValueHint::Other),
                )
                .arg(
                    Arg::new("compress")
                        .short('c')
                        .long("compress")
                        .value_name("method")
                        .value_parser(compression_names)
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
                        .conflicts_with("data-filter-cmd")
                        .value_hint(ValueHint::Other),
                )
                .arg(
                    Arg::new("data-filter-cmd")
                        .long("data-filter")
                        .action(ArgAction::Append)
                        .num_args(1..)
                        .allow_hyphen_values(true)
                        .value_hint(ValueHint::Other)
                        .help(
                            "External data filter command line (for compression and decompression)",
                        ),
                ),
        )
        .subcommand(
            Command::new("list")
                .about("List archive content")
                .alias("l")
                .arg(
                    Arg::new("archive")
                        .help("Archive file path")
                        .required(true)
                        .value_hint(ValueHint::FilePath),
                ),
        )
        .subcommand(
            Command::new("extract")
                .about("Extract files from archive")
                .alias("e")
                .arg(
                    Arg::new("archive")
                        .help("Archive file path")
                        .required(true)
                        .value_hint(ValueHint::FilePath),
                )
                .arg(
                    Arg::new("base-dir")
                        .short('C')
                        .long("base-directory")
                        .default_value(".")
                        .value_hint(ValueHint::DirPath),
                )
                .arg(
                    Arg::new("paths")
                        .action(ArgAction::Append)
                        .help("File paths to be extracted")
                        .value_hint(ValueHint::Other),
                )
                .arg(
                    Arg::new("pipe")
                        .short('p')
                        .long("pipe")
                        .action(ArgAction::SetTrue)
                        .help("Pipe the data to stdout"),
                )
                .arg(
                    Arg::new("data-filter-cmd")
                        .long("data-filter")
                        .action(ArgAction::Append)
                        .num_args(1..)
                        .allow_hyphen_values(true)
                        .value_hint(ValueHint::Other)
                        .help(
                            "External data filter command line (for compression and decompression)",
                        ),
                ),
        )
        .subcommand(
            Command::new("test")
                .alias("t")
                .arg(
                    Arg::new("archive")
                        .help("Archive file path")
                        .required(true)
                        .value_hint(ValueHint::FilePath),
                )
                .about("Check the archive integrity"),
        )
        .subcommand(
            Command::new("info")
                .alias("i")
                .arg(
                    Arg::new("archive")
                        .help("Archive file path")
                        .required(true)
                        .value_hint(ValueHint::FilePath),
                )
                .about("Show the information of archive"),
        )
        .subcommand_required(true)
        .about("An archive format for data backups with indexing and compression capabilities")
}
