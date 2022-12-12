use std::collections::HashMap;
use std::fs::remove_file;
use std::str::FromStr;

use anyhow::anyhow;
use bytesize::ByteSize;
use reflink::reflink;
use walkdir::WalkDir;

use cow_dedupe::cli::build_cli;
use cow_dedupe::file_hash;

/// the hash length to be used; in bytes
const HASH_LENGTH: usize = 64;

fn main() -> anyhow::Result<()> {
    let matches = build_cli().get_matches();
    let paths = matches.get_many::<String>("path").unwrap();
    let min_size = matches
        .get_one::<String>("min-size")
        .map(|x| x.as_str())
        .unwrap_or("0");
    let min_size = ByteSize::from_str(min_size)
        .map_err(|e| anyhow!("Invalid size: {}", e))?
        .0;
    let dry_run = matches.get_flag("dry-run");

    let mut map = HashMap::new();

    let entries = paths.flat_map(WalkDir::new);
    for entry in entries {
        let entry = entry?;
        let metadata = entry.metadata()?;
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

    let mut hash_buf = [0_u8; HASH_LENGTH];
    let mut dups_vec = Vec::new();
    for dups in dup_files.iter() {
        let mut map = HashMap::new();
        for path in dups {
            println!("Hashing ({}/{}) {:?}", count, total, path);
            count += 1_usize;
            file_hash(path, &mut hash_buf)?;
            map.entry(hash_buf)
                .or_insert_with(Vec::new)
                .push(path.clone());
        }
        for x in map.into_iter().filter(|x| x.1.len() > 1) {
            dups_vec.push(x.1);
        }
    }

    if dry_run {
        let mut dupes_sum = 0_u64;
        for dupe in &dups_vec {
            dupes_sum += dupe[0].metadata()?.len();
        }

        let mut reflink_sum = 0_u64;
        for (mut dupe_count, first) in dups_vec.iter().map(|x| (x.len(), &x[0])) {
            let first_size = first.metadata()?.len();
            if dupe_count >= 1 {
                dupe_count -= 1;
            }
            reflink_sum += dupe_count as u64 * first_size;
        }

        println!(
            "Size of unique duplicated files: {}",
            ByteSize(dupes_sum).to_string_as(true)
        );
        println!(
            "Size of all duplicated files: {}",
            ByteSize(dupes_sum + reflink_sum).to_string_as(true)
        );
        println!(
            "Size of files to be reflinked: {}",
            ByteSize(reflink_sum).to_string_as(true)
        );
    } else {
        let total = dups_vec.iter().map(|x| x.len()).sum::<usize>() - dups_vec.len();
        for (i, dups) in dups_vec.iter().enumerate() {
            assert!(dups.len() > 1);
            // pick the first file as reflinking source, since these files are all identical
            let first = &dups[0];
            for x in dups.iter().skip(1) {
                println!("Reflinking: ({}/{}) {:?}", i + 1, total, x);
                remove_file(x)?;
                reflink(first, x)?;
                if !x.exists() {
                    panic!("Checking failed: {:?} doesn't exist", x);
                }
            }
        }
    }

    Ok(())
}
