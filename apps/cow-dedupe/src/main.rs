use bytesize::ByteSize;
use std::collections::HashMap;
use std::fs::{remove_file, File};
use std::io;
use std::io::BufReader;
use std::path::Path;
use std::str::FromStr;

use clap::{Arg, Command};
use cow_dedupe::errors::*;
use reflink::reflink;
use sha2::digest::Digest;
use sha2::Sha256;
use walkdir::WalkDir;

fn main() {
    let matches = Command::new("cow-dedupe")
        .about("A simple file-based deduplication tool using CoW semantic (reflink)")
        .arg(
            Arg::new("path")
                .required(true)
                .multiple_values(true)
                .help("Path to directory"),
        )
        .arg(
            Arg::new("min-size")
                .long("min-size")
                .short('m')
                .help("Minimum size filter")
                .takes_value(true)
                .required(false),
        )
        .get_matches();

    let mut map = HashMap::new();

    let paths = matches.values_of("path").unwrap();
    let min_size: u64 = matches
        .value_of("min-size")
        .map(|x| ByteSize::from_str(x).unwrap().0)
        .unwrap_or(0);

    let entries = paths.flat_map(WalkDir::new);
    for entry in entries {
        let entry = entry.unwrap();
        let metadata = entry.metadata().unwrap();
        if !metadata.is_file() {
            continue;
        }

        let size = metadata.len();
        if size == 0 || size < min_size {
            continue;
        }

        println!("Indexing {:?}", (size, entry.path()));
        map.entry(size)
            .or_insert_with(Vec::new)
            .push(entry.path().to_path_buf());
    }

    let dup_files = map
        .into_iter()
        .filter(|x| x.1.len() > 1)
        .map(|x| x.1)
        .collect::<Vec<_>>();
    let total: usize = dup_files.iter().map(|x| x.len()).sum();
    let mut count = 1;

    let mut hash_buf = [0_u8; 32];
    let mut dups_vec = Vec::new();
    for dups in dup_files.iter() {
        let mut map = HashMap::new();
        for path in dups {
            println!("Hashing ({}/{}) {:?}", count, total, path);
            count += 1_usize;
            file_hash(path, &mut hash_buf).unwrap();
            map.entry(hash_buf)
                .or_insert_with(Vec::new)
                .push(path.clone());
        }
        for x in map.into_iter().filter(|x| x.1.len() > 1) {
            dups_vec.push(x.1);
        }
    }

    let total = dups_vec.iter().map(|x| x.len()).sum::<usize>() - dups_vec.len();
    let mut count = 1;
    for dups in dups_vec {
        assert!(dups.len() > 1);
        let first = &dups[0];
        for x in dups.iter().skip(1) {
            println!("Reflinking: ({}/{}) {:?}", count, total, x);
            count += 1_usize;
            remove_file(x).unwrap();
            reflink(first, x).unwrap();
        }
    }
}

fn file_hash<P: AsRef<Path>>(path: P, buf: &mut [u8]) -> Result<()> {
    let mut sha256 = Sha256::new();

    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    io::copy(&mut reader, &mut sha256)?;
    let array = sha256.finalize();
    for x in array.iter().enumerate() {
        buf[x.0] = *(x.1);
    }
    Ok(())
}
