use crate::cli::{GroupArgs, HashFn};
use crate::hash::{FixedDigest, B3_1024, B3_256, B3_512};
use crate::{group_by_content, group_by_fragments, group_by_size};
use digest::generic_array::GenericArray;
use digest::typenum::Unsigned;
use sha2::{Sha256, Sha512};
use std::io;
use std::path::PathBuf;

pub fn main(args: &GroupArgs) -> anyhow::Result<()> {
    let paths = &args.common.path;
    eprintln!("Collecting files...");
    let mut files = collect_file(paths)?;
    eprintln!("File entries: {}", files.len());
    eprintln!("Grouping by size...");
    group_by_size(&mut files);
    eprintln!("File entries: {}", files.len());
    eprintln!("Grouping by file fragments");
    group_by_fragments(&mut files)?;
    eprintln!("File entries: {}", files.len());
    eprintln!("Grouping by file content...");
    match args.common.hash_fn {
        HashFn::B3_256 => generic_group_by_content::<B3_256>(&mut files),
        HashFn::B3_512 => generic_group_by_content::<B3_512>(&mut files),
        HashFn::B3_1024 => generic_group_by_content::<B3_1024>(&mut files),
        HashFn::Sha256 => generic_group_by_content::<Sha256>(&mut files),
        HashFn::Sha512 => generic_group_by_content::<Sha512>(&mut files),
    }?;
    eprintln!("File entries: {}", files.len());

    // print these entries out

    Ok(())
}

fn generic_group_by_content<H: FixedDigest>(files: &mut Vec<FileEntry>) -> anyhow::Result<()>
where
    [(); H::OutputSize::USIZE]:,
    [u8; H::OutputSize::USIZE]: From<GenericArray<u8, H::OutputSize>>,
{
    group_by_content::<H>(files)?;
    Ok(())
}

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
