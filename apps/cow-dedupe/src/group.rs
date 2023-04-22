use std::io;
use std::path::PathBuf;

use digest::generic_array::GenericArray;
use digest::typenum::Unsigned;
use sha2::{Sha256, Sha512};

use crate::cli::{GroupArgs, HashFn};
use crate::hash::{FixedDigest, B3_1024, B3_256, B3_512};
use crate::{group_by_content, group_by_size};

pub fn main(args: &GroupArgs) -> anyhow::Result<()> {
    let paths = &args.common.path;
    eprintln!("Collecting files...");
    let mut entries = collect_file(paths)?;
    eprintln!("File entries: {}", entries.len());
    eprintln!("Grouping by size...");
    let mut groups = group_by_size(&mut entries);
    eprintln!("File entries: {}", entries.len());

    match args.common.hash_fn {
        HashFn::B3_256 => generic_group_files_by_hash::<B3_256>(&mut groups),
        HashFn::B3_512 => generic_group_files_by_hash::<B3_512>(&mut groups),
        HashFn::B3_1024 => generic_group_files_by_hash::<B3_1024>(&mut groups),
        HashFn::Sha256 => generic_group_files_by_hash::<Sha256>(&mut groups),
        HashFn::Sha512 => generic_group_files_by_hash::<Sha512>(&mut groups),
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
    eprintln!("File entries: {}", files.len());
    eprintln!("Grouping by file content...");
    let groups = group_by_content::<H>(files)?;
    eprintln!("File entries: {}", files.len());

    // print out
    for x in groups.iter().filter(|x| x.1.len() >= 2) {
        println!(
            "{}, {}",
            hex::encode(x.0),
            bytesize::to_string(x.1[0].size, true)
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
fn collect_file(paths: &Vec<String>) -> io::Result<Vec<FileEntry>> {
    let mut files_vec = Vec::new();
    for path in paths {
        let files = walkdir::WalkDir::new(path);
        for entry in files {
            let entry = entry?;
            if !entry.file_type().is_file() {
                continue;
            }
            files_vec.push(FileEntry {
                path: entry.path().into(),
                size: entry.metadata()?.len(),
            });
        }
    }
    Ok(files_vec)
}
