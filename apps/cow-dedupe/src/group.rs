use crate::cli::GroupArgs;
use crate::{group_by_middle_content, group_by_size};
use std::fs::FileType;
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
    eprintln!("Grouping by middle content...");
    group_by_middle_content(&mut files)?;
    eprintln!("File entries: {}", files.len());

    Ok(())
}

/// Returns (path, file size)
fn collect_file(paths: &Vec<String>) -> io::Result<Vec<(PathBuf, u64)>> {
    let mut files_vec = Vec::new();
    for path in paths {
        let files = walkdir::WalkDir::new(path);
        for entry in files {
            let entry = entry?;
            if !entry.file_type().is_file() {
                continue;
            }
            files_vec.push((entry.path().into(), entry.metadata()?.len()));
        }
    }
    Ok(files_vec)
}
