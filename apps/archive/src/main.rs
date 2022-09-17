use std::path::{Path, PathBuf};
use std::sync::Mutex;

use clap::{Arg, Command};
use once_cell::sync::Lazy;

use archive::archive::Archive;
use archive::compressors::{Compress, GzipCompressor, NoCompressor};
use archive::errors::{Error, Result};
use archive::Compressor;
use bczhc_lib::mutex_lock;

static CONFIGS: Lazy<Mutex<Configs>> = Lazy::new(|| Mutex::new(Configs::default()));

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
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
        )
        .about("An index-able, compress-able archive format for data backups")
        .get_matches();

    let mut paths = matches.values_of("path").unwrap();
    let output = matches.value_of("output").unwrap();
    let base_dir = matches.value_of("base-dir").unwrap();
    let compressor_name = matches.value_of("compress").unwrap();
    let compress_level = matches.value_of("level").unwrap();

    let (compressor_type, compressor) = resolve_compressor(compressor_name, compress_level)?;

    mutex_lock!(CONFIGS)
        .compressor_type
        .replace(compressor_type);

    let mut archive = Archive::new(output, compressor)?;

    for path in paths {
        push_path(&mut archive, base_dir, path)?;
    }

    Ok(())
}

fn push_path(archive: &mut Archive, base_dir: &str, path: &str) -> Result<()> {
    let compressor_type = mutex_lock!(CONFIGS).compressor_type.unwrap();

    let walk_dir = {
        let mut buf = PathBuf::new();
        buf.push(base_dir);
        buf.push(path);
        buf
    };

    let entries = walkdir::WalkDir::new(&walk_dir);
    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        archive.add_path(Path::new(base_dir), path, compressor_type)?;
    }

    Ok(())
}

fn resolve_compressor(name: &str, level: &str) -> Result<(Compressor, Box<dyn Compress>)> {
    let parse = name.parse::<Compressor>();
    if parse.is_err() {
        return Err(Error::InvalidCompressor);
    }
    let compressor_type = parse.unwrap();

    let level_num = match level {
        "best" => compressor_type.best_level(),
        _ => level.parse::<u32>().map_err(|_| Error::InvalidCompressor)?,
    };

    let compressor: Box<dyn Compress> = match compressor_type {
        Compressor::Gzip => Box::new(GzipCompressor::new(level_num)),
        Compressor::Xz => {
            todo!()
        }
        Compressor::Zstd => {
            todo!()
        }
        Compressor::None => Box::new(NoCompressor::new()),
        Compressor::External => {
            // ensured by clap
            unreachable!()
        }
    };

    Ok((compressor_type, compressor))
}

#[derive(Default)]
struct Configs {
    compressor_type: Option<Compressor>,
}
