use crate::archive::Archive;
use crate::compressors::{create_compressor, Compress, ExternalFilter, Level};
use crate::{Compressor, Configs};
use bczhc_lib::mutex_lock;

use clap::ArgMatches;
use once_cell::sync::Lazy;

use std::fs::File;
use std::io::{BufWriter, Seek, Write};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::Mutex;

use crate::errors::*;

static CONFIGS: Lazy<Mutex<Configs>> = Lazy::new(|| Mutex::new(Configs::default()));

pub fn main(matches: &ArgMatches) -> Result<()> {
    let paths = matches.get_many::<String>("path").unwrap();
    let output = matches.get_one::<String>("output").unwrap();
    let base_dir = matches.get_one::<String>("base-dir").unwrap();

    let data_filter_cmd = matches
        .get_many::<String>("data-filter-cmd")
        .map(|values| values.map(|x| x.to_owned()).collect::<Vec<_>>());

    let compressor_type;
    let compressor: Box<dyn Compress> = match data_filter_cmd {
        None => {
            // use built-in compressors
            let compressor_name = matches.get_one::<String>("compress").unwrap();
            let compress_level = matches.get_one::<String>("level").unwrap();

            let compressor_name = compressor_name
                .parse::<Compressor>()
                .map_err(|_| Error::InvalidCompressor)?;
            let compress_level =
                Level::from_str(compress_level).map_err(|_| Error::InvalidCompressor)?;

            compressor_type = compressor_name;
            create_compressor(compressor_name, compress_level)
        }
        Some(ref cmd) => {
            // external compressor
            compressor_type = Compressor::External;
            Box::new(ExternalFilter::new(cmd))
        }
    };

    mutex_lock!(CONFIGS)
        .compressor_type
        .replace(compressor_type);

    let mut archive = create_archive(output, compressor)?;

    println!("Indexing...");
    for path in paths {
        if Path::new(path).is_absolute() {
            return Err(Error::AbsolutePath);
        }
        add_path(&mut archive, base_dir, path)?;
    }
    println!("Writing files...");
    archive.write()?;

    archive.flush()?;

    Ok(())
}

fn create_archive<'a, P: AsRef<Path>>(
    path: P,
    compressor: Box<dyn Compress + 'a>,
) -> Result<Archive<impl Write + Seek>> {
    let file = File::options()
        .truncate(true)
        .create(true)
        .read(true)
        .write(true)
        .open(path)?;
    let writer = BufWriter::new(file);
    Archive::new(writer, compressor)
}

fn add_path(archive: &mut Archive<impl Write + Seek>, base_dir: &str, path: &str) -> Result<()> {
    let compressor_type = mutex_lock!(CONFIGS).compressor_type.unwrap();

    let walk_dir = {
        let mut buf = PathBuf::new();
        buf.push(base_dir);
        buf.push(path);
        buf
    };

    let entries = walkdir::WalkDir::new(walk_dir);
    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        archive.add_path(Path::new(base_dir), path, compressor_type)?;
    }

    Ok(())
}
