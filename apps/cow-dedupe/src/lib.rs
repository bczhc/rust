#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
#![feature(slice_group_by)]

const IO_BUF_SIZE: usize = 4096;

use colored::Colorize;
use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::io::{Read, Write};

use digest::generic_array::GenericArray;
use digest::typenum::Unsigned;
use digest::{Digest, FixedOutput, OutputSizeUser};
use indicatif::{ProgressBar, ProgressStyle};

use crate::group::FileEntry;
use crate::hash::{FixedDigest, HashWriter};

pub mod cli;
pub mod dedupe;
pub mod errors;
pub mod group;
pub mod hash;

pub fn group_by_size(entries: &mut Vec<FileEntry>) -> Vec<Vec<FileEntry>> {
    entries.sort_by_key(|x| x.size);
    entries
        .group_by(|a, b| a.size == b.size)
        .map(Vec::from)
        .collect::<Vec<_>>()
}

/// 50 bytes after file start
/// 50 bytes before file end
/// middle 100 bytes
pub fn group_by_fragments(entries: &mut Vec<Vec<FileEntry>>) -> io::Result<()> {
    todo!()
}

pub fn group_by_content<H: FixedDigest>(
    entries: &mut Vec<Vec<FileEntry>>,
) -> io::Result<HashMap<[u8; H::OutputSize::USIZE], Vec<FileEntry>>>
where
    [(); H::OutputSize::USIZE]:,
    [u8; H::OutputSize::USIZE]: From<GenericArray<u8, H::OutputSize>>,
{
    let mut groups = HashMap::new();

    let total_file_size = entries
        .iter()
        .map(|x| x.iter().map(|x| x.size).sum::<u64>())
        .sum();
    let progress_bar = ProgressBar::new(total_file_size);
    progress_bar.set_style(
        ProgressStyle::default_bar()
            .template("{msg} {bar:50} {bytes}/{total_bytes}")
            .unwrap(),
    );
    progress_bar.set_message("Hashing files".cyan().bold().to_string());

    for g in entries {
        let mut vec: Vec<(FileEntry, [u8; H::OutputSize::USIZE])> = Vec::new();
        for x in g.iter() {
            let mut hasher = HashWriter(H::new());
            let mut file = File::open(&x.path)?;
            io_copy_with_progress(&mut file, &mut hasher, |s| {
                progress_bar.inc(s as u64);
            })?;

            let digest: [u8; H::OutputSize::USIZE] = hasher.0.finalize_fixed().into();
            vec.push((x.clone(), digest));
        }
        vec.sort_by_key(|x| x.1);
        for x in vec.group_by(|a, b| a.1 == b.1) {
            let hash = x[0].1;
            let group = x.iter().map(|x| x.0.clone()).collect::<Vec<_>>();
            groups.insert(hash, group);
        }
    }
    Ok(groups)
}

fn io_copy_with_progress<R, W, F>(mut reader: R, mut writer: W, progress: F) -> io::Result<()>
where
    R: Read,
    W: Write,
    F: Fn(usize),
{
    let mut buf = [0_u8; IO_BUF_SIZE];
    loop {
        let read_size = reader.read(&mut buf)?;
        if read_size == 0 {
            // EOF
            break;
        }
        writer.write_all(&buf[..read_size])?;
        progress(read_size);
    }
    Ok(())
}
