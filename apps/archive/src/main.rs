use archive::archive::Archive;
use archive::errors::Error;
use clap::{Arg, Command};
use std::path::{Path, PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new("archive")
        .arg(Arg::new("output").required(true))
        .arg(Arg::new("path").required(true).multiple_values(true))
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
                .possible_values(&["gzip", "xz", "zstd", "no"])
                .help("Compression method used for each file, \"no\" means no compression"),
        )
        .arg(
            Arg::new("level")
                .short('l')
                .long("level")
                .help("Compression level"),
        )
        .about("An index-able, compress-able archive format for data backups")
        .get_matches();

    let paths = matches.values_of("path").unwrap();
    let output = matches.value_of("output").unwrap();
    let base_dir = matches.value_of("base-dir").unwrap();

    let mut archive = Archive::new(output)?;

    let walk_dir = {
        let mut buf = PathBuf::new();
        buf.push(base_dir);
        // buf.push(path);
        buf
    };

    let entries = walkdir::WalkDir::new(&walk_dir);
    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        archive.add_path(Path::new(base_dir), path)?;
    }

    Ok(())
}
