use crate::archive::Archive;
use crate::compressors::{Compress, GzipCompressor, NoCompressor};
use crate::{Compressor, Configs};
use bczhc_lib::mutex_lock;
use clap::ArgMatches;
use once_cell::sync::Lazy;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use crate::errors::*;

static CONFIGS: Lazy<Mutex<Configs>> = Lazy::new(|| Mutex::new(Configs::default()));

pub fn main(matches: &ArgMatches) -> Result<()> {
    let mut paths = matches.get_many::<String>("path").unwrap();
    let output = matches.get_one::<String>("output").unwrap();
    let base_dir = matches.get_one::<String>("base-dir").unwrap();
    let compressor_name = matches.get_one::<String>("compress").unwrap();
    let compress_level = matches.get_one::<String>("level").unwrap();

    let (compressor_type, compressor) = resolve_compressor(compressor_name, compress_level)?;

    mutex_lock!(CONFIGS)
        .compressor_type
        .replace(compressor_type);

    let mut archive = Archive::new(output, compressor)?;

    for path in paths {
        push_path(&mut archive, base_dir, path)?;
    }
    archive.write_files()?;

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
