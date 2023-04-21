#![feature(generic_const_exprs)]

use std::fs::File;
use std::io;

use digest::generic_array::GenericArray;
use digest::typenum::Unsigned;
use digest::{Digest, FixedOutput, OutputSizeUser};

use crate::group::FileEntry;
use crate::hash::{FixedDigest, HashWriter};

pub mod cli;
pub mod dedupe;
pub mod errors;
pub mod group;
pub mod hash;

pub fn group_by_size(entries: &mut Vec<FileEntry>) {
    entries.sort_by_key(|x| x.size);
    entries.dedup_by_key(|x| x.size);
}

/// 50 bytes after file start
/// 50 bytes before file end
/// middle 100 bytes
pub fn group_by_fragments(entries: &mut Vec<FileEntry>) -> io::Result<()> {
    Ok(())
}

pub fn group_by_content<H: FixedDigest>(entries: &mut Vec<FileEntry>) -> io::Result<()>
where
    [(); H::OutputSize::USIZE]:,
    [u8; H::OutputSize::USIZE]: From<GenericArray<u8, H::OutputSize>>,
{
    let mut vec: Vec<(&FileEntry, [u8; H::OutputSize::USIZE])> = Vec::new();
    for x in entries {
        let mut hasher = HashWriter(H::new());
        let mut file = File::open(&x.path)?;
        io::copy(&mut file, &mut hasher).unwrap();
        let digest: [u8; H::OutputSize::USIZE] = hasher.0.finalize_fixed().into();
        vec.push((x, digest));
    }
    vec.sort_by_key(|x| x.1);
    vec.dedup_by_key(|x| x.1);
    Ok(())
}
