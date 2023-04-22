use bytesize::ByteSize;
use colored::Colorize;
use std::io;
use std::path::PathBuf;
use std::sync::Mutex;

use bczhc_lib::mutex_lock;
use digest::generic_array::GenericArray;
use digest::typenum::Unsigned;
use once_cell::sync::Lazy;
use sha2::{Sha256, Sha512};
use sha3::{Sha3_256, Sha3_512};

use crate::cli::{GroupArgs, HashFn};
use crate::hash::{FixedDigest, B3_1024, B3_128, B3_160, B3_2048, B3_256, B3_512};
use crate::{group_by_content, group_by_size};

static ARGS: Lazy<Mutex<Option<GroupArgs>>> = Lazy::new(|| Mutex::new(None));

pub fn main(args: GroupArgs) -> anyhow::Result<()> {
    mutex_lock!(ARGS).replace(args.clone());

    let min_size = match &args.common.min_size.parse::<ByteSize>() {
        Ok(s) => s.0,
        Err(e) => return Err(anyhow::anyhow!("Invalid min size: {}", e)),
    };

    let paths = &args.common.path;
    eprintln!("Collecting files...");
    let mut entries = collect_file(paths, min_size)?;
    eprintln!("File entries: {}", entries.len());
    eprintln!("Grouping by size...");
    let mut groups = group_by_size(&mut entries);
    eprintln!("Group count: {}", groups.len());

    match args.common.hash_fn {
        HashFn::B3_128 => generic_group_files_by_hash::<B3_128>(&mut groups),
        HashFn::B3_160 => generic_group_files_by_hash::<B3_160>(&mut groups),
        HashFn::B3_256 => generic_group_files_by_hash::<B3_256>(&mut groups),
        HashFn::B3_512 => generic_group_files_by_hash::<B3_512>(&mut groups),
        HashFn::B3_1024 => generic_group_files_by_hash::<B3_1024>(&mut groups),
        HashFn::B3_2048 => generic_group_files_by_hash::<B3_2048>(&mut groups),
        HashFn::Sha256 => generic_group_files_by_hash::<Sha256>(&mut groups),
        HashFn::Sha512 => generic_group_files_by_hash::<Sha512>(&mut groups),
        HashFn::Sha3_256 => generic_group_files_by_hash::<Sha3_256>(&mut groups),
        HashFn::Sha3_512 => generic_group_files_by_hash::<Sha3_512>(&mut groups),
    }?;

    Ok(())
}

fn generic_group_files_by_hash<H: FixedDigest>(
    files: &mut Vec<Vec<FileEntry>>,
) -> anyhow::Result<()>
where
    [(); H::OutputSize::USIZE]:,
    [u8; H::OutputSize::USIZE]: From<GenericArray<u8, H::OutputSize>>,
{
    eprintln!("Grouping by file fragments");
    // group_by_fragments(files)?;
    // eprintln!("File entries: {}", files.len());
    eprintln!("Grouping by file content...");
    let groups = group_by_content::<H>(files)?;
    eprintln!("Group count: {}", groups.len());
    let duplicated_file_group_count = groups.iter().filter(|x| x.1.len() >= 2).count();
    eprintln!("Duplicated file groups: {}", duplicated_file_group_count);

    let compact_hash = mutex_lock!(ARGS).as_ref().unwrap().compact_hash;

    // print out
    for x in groups.iter().filter(|x| x.1.len() >= 2) {
        let file_count = x.1.len();

        let hash_string = if compact_hash {
            hex::encode(&x.0[..20])
        } else {
            hex::encode(x.0)
        };
        println!(
            "{}",
            format!(
                "{}, {} * {}",
                hash_string,
                bytesize::to_string(x.1[0].size, true),
                file_count
            )
            .yellow()
        );
        for x in x.1 {
            println!("{}", x.path.display());
        }
        println!()
    }

    Ok(())
}

#[derive(Clone, Debug)]
pub struct FileEntry {
    pub path: PathBuf,
    pub size: u64,
}

/// Returns (path, file size)
fn collect_file(paths: &Vec<String>, min_size: u64) -> io::Result<Vec<FileEntry>> {
    let mut files_vec = Vec::new();
    for path in paths {
        let files = walkdir::WalkDir::new(path);
        for entry in files {
            let entry = entry?;
            if !entry.file_type().is_file() {
                continue;
            }
            let metadata = entry.metadata()?;
            let file_size = metadata.len();
            if file_size >= min_size {
                files_vec.push(FileEntry {
                    path: entry.path().into(),
                    size: file_size,
                });
            }
        }
    }
    Ok(files_vec)
}
