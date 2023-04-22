#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
#![feature(slice_group_by)]

extern crate core;

use std::fs::File;
use std::io;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use ::serde::{Deserialize, Serialize};
use anyhow::anyhow;
use colored::Colorize;
use digest::generic_array::GenericArray;
use digest::typenum::Unsigned;
use digest::Digest;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::ParallelSliceMut;

use crate::group::FileEntry;
use crate::hash::{FixedDigest, HashWriter};
use crate::serde::Output;

const IO_BUF_SIZE: usize = 4096;

pub mod cli;
pub mod dedupe;
pub mod errors;
pub mod group;
pub mod hash;
pub mod serde;

pub fn group_by_size(entries: &mut Vec<FileEntry>) -> Vec<Vec<FileEntry>> {
    entries.par_sort_by_key(|x| x.size);
    entries
        .group_by(|a, b| a.size == b.size)
        .map(Vec::from)
        .collect::<Vec<_>>()
}

pub trait FileHash<H>
where
    H: FixedDigest,
{
    fn hash<P: AsRef<Path>, F>(p: P, progress: F) -> io::Result<[u8; H::OutputSize::USIZE]>
    where
        [(); H::OutputSize::USIZE]:,
        [u8; H::OutputSize::USIZE]: From<GenericArray<u8, H::OutputSize>>,
        F: Fn(usize);
}

/// Hash the whole file
pub struct FileFullHasher;
/// 1. 50 bytes from file start
/// 1. 50 bytes from file end
/// 1. 100 bytes through file middle
pub struct FileFragmentsHasher;

impl<H> FileHash<H> for FileFullHasher
where
    H: FixedDigest,
{
    fn hash<P: AsRef<Path>, F>(p: P, progress: F) -> io::Result<[u8; H::OutputSize::USIZE]>
    where
        [(); H::OutputSize::USIZE]:,
        [u8; H::OutputSize::USIZE]: From<GenericArray<u8, H::OutputSize>>,
        F: Fn(usize),
    {
        let mut file = File::open(p)?;

        let mut hasher = HashWriter(H::new());
        io_copy_with_progress(&mut file, &mut hasher, progress)?;
        Ok(hasher.0.finalize_fixed().into())
    }
}

pub fn group_by_hash<'a, H, FH: FileHash<H>, G, I>(
    entries_iter_getter: G,
) -> io::Result<Vec<([u8; H::OutputSize::USIZE], Vec<FileEntry>)>>
where
    H: FixedDigest,
    [(); H::OutputSize::USIZE]:,
    [u8; H::OutputSize::USIZE]: From<GenericArray<u8, H::OutputSize>>,
    G: Fn() -> I,
    I: Iterator<Item = &'a [FileEntry]>,
{
    let mut groups = Vec::new();

    let total_file_size = entries_iter_getter()
        .map(|x| x.iter().map(|x| x.size).sum::<u64>())
        .sum();
    let progress_bar = ProgressBar::new(total_file_size);
    progress_bar.set_style(
        ProgressStyle::default_bar()
            .template("{msg} {bar:50} {bytes}/{total_bytes}")
            .unwrap(),
    );
    progress_bar.set_message("Hashing files".cyan().bold().to_string());

    for g in entries_iter_getter() {
        let mut vec: Vec<(FileEntry, [u8; H::OutputSize::USIZE])> = Vec::new();
        for x in g.iter() {
            let digest = FH::hash(&x.path, |s| {
                progress_bar.inc(s as u64);
            })?;
            vec.push((x.clone(), digest));
        }
        vec.par_sort_by_key(|x| x.1);
        for x in vec.group_by(|a, b| a.1 == b.1) {
            let hash = x[0].1;
            let group = x.iter().map(|x| x.0.clone()).collect::<Vec<_>>();
            groups.push((hash, group));
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

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Group {
    pub file_size: u64,
    pub hash: String,
    pub files: Vec<PathBuf>,
}

pub fn parse_input_file<P: AsRef<Path>>(input: P) -> anyhow::Result<Vec<Group>> {
    let mut data = Vec::new();
    File::open(input)?.read_to_end(&mut data)?;
    if data.is_empty() {
        return Err(anyhow!("Empty input file"));
    }
    if data[0] == b'{' {
        // treat as json
        let json_str = std::str::from_utf8(&data)?;
        let output: Output = serde_json::from_str(json_str)?;
        return Ok(output.groups);
    }

    // binary format
    let output: Output = bincode::deserialize(&data)?;
    Ok(output.groups)
}
