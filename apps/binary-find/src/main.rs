use std::fs::File;
use std::io;
use std::io::{stdin, BufReader, Read};
use std::process::exit;

use anyhow::anyhow;
use clap::Parser;

use bin_find::{Args, IterWindow};

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let pattern = hex::decode(&args.hex_string)
        .map_err(|_| anyhow!("Invalid hex string: {}", args.hex_string))?;
    let found = match args.file {
        None => search(stdin().lock(), &pattern)?,
        Some(path) => {
            let reader = BufReader::new(File::open(path)?);
            search(reader, &pattern)?
        }
    };

    if !found {
        exit(1);
    }
    Ok(())
}

fn search<R: Read>(reader: R, pattern: &[u8]) -> io::Result<bool> {
    let mut windows = reader.bytes().map(Result::unwrap).windows(pattern.len());
    let iter = windows.iter();
    let mut found = false;
    for (offset, x) in iter.enumerate() {
        if x == pattern {
            println!("Found at offset: {}", offset);
            found = true;
        }
    }

    Ok(found)
}
