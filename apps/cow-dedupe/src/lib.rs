use std::io;
use std::path::Path;

pub mod cli;
pub mod dedupe;
pub mod errors;
pub mod group;
pub mod hash;

pub fn group_by_size<P: AsRef<Path>>(path: &mut Vec<(P, u64)>) {
    path.sort_by_key(|x| x.1);
    path.dedup_by_key(|x| x.1);
}

pub fn group_by_middle_content<P: AsRef<Path>>(path: &mut Vec<(P, u64)>) -> io::Result<()> {
    Ok(())
}
