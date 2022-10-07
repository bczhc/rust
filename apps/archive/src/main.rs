use std::path::{Path, PathBuf};
use std::sync::Mutex;

use clap::{Arg, ArgAction, Command};
use once_cell::sync::Lazy;

use archive::archive::Archive;
use archive::compressors::{Compress, GzipCompressor, NoCompressor};
use archive::errors::{Error, Result};
use archive::Compressor;
use bczhc_lib::mutex_lock;

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
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
                        .value_parser(["gzip", "xz", "zstd", "no"])
                        .help("Compression method used for each file, \"no\" means no compression")
                        .default_value("no")
                        .ignore_case(true),
                )
                .arg(
                    Arg::new("level")
                        .short('l')
                        .long("level")
                        .help("Compression level")
                        .default_value("best"),
                ),
        )
        .subcommand(
            Command::new("list")
                .alias("l")
                .arg(Arg::new("archive").help("Archive file path").required(true)),
        )
        .subcommand_required(true)
        .about("An archive format for data backups with indexing and compression capabilities")
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("create") {
        archive::create::main(matches)
    } else if let Some(matches) = matches.subcommand_matches("list") {
        archive::list::main(matches)
    } else {
        unreachable!()
    }?;

    Ok(())
}
