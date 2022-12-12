use std::fs::File;
use std::io;
use std::io::{BufReader, Read};
use std::path::Path;

use errors::*;

pub mod cli;
pub mod errors;

pub fn file_hash<P: AsRef<Path>>(path: P, buf: &mut [u8]) -> Result<()> {
    let mut hasher = blake3::Hasher::new();
    let mut reader = BufReader::new(File::open(path)?);
    io::copy(&mut reader, &mut hasher)?;

    let mut output_reader = hasher.finalize_xof();
    // there's no way this can encounter an IO error
    output_reader.read_exact(buf).unwrap();

    Ok(())
}
