#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
#![feature(slice_group_by)]

extern crate core;

use std::any::TypeId;
use std::fs::File;
use std::io;
use std::io::{Read, Seek, SeekFrom, Write};
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
        .filter(|x| x.len() >= 2)
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
/// 1. some portion from file start
/// 1. some portion from file end
/// 1. some portion bytes through file middle
///
/// :)
pub struct FileFragmentsHasher;

impl FileFragmentsHasher {
    const HEAD_SIZE: usize = 50;
    const TAIL_SIZE: usize = 50;
    const MIDDLE_SIZE: usize = 100;

    const BUF_SIZE: usize = max(max(Self::HEAD_SIZE, Self::TAIL_SIZE), Self::MIDDLE_SIZE);
    const TOTAL_SIZE: usize = Self::HEAD_SIZE + Self::TAIL_SIZE + Self::MIDDLE_SIZE;
}

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

impl<H> FileHash<H> for FileFragmentsHasher
where
    H: FixedDigest,
{
    fn hash<P: AsRef<Path>, F>(p: P, progress: F) -> io::Result<[u8; H::OutputSize::USIZE]>
    where
        [(); H::OutputSize::USIZE]:,
        [u8; H::OutputSize::USIZE]: From<GenericArray<u8, H::OutputSize>>,
        F: Fn(usize),
    {
        // everytime for this call, the final data to be hashed has the size
        // HEAD_SIZE + TAIL_SIZE + MIDDLE_SIZE;
        // for files smaller than the needed read size, pad with zeros.
        let mut buf = [0_u8; Self::BUF_SIZE];
        let mut hasher = H::new();

        let mut file = File::open(p)?;
        let file_len = file.metadata()?.len();

        // read the first 50 bytes; coerced to <file_size> if needed
        let read_size = (Self::HEAD_SIZE as u64).min(file_len);
        file.read_exact(&mut buf[..read_size as usize])?;
        buf[..Self::HEAD_SIZE][read_size as usize..].fill(0);
        hasher.update(&buf[..Self::HEAD_SIZE]);

        // read the last 50 bytes; coerced to <file_size> if needed
        let read_size = (Self::TAIL_SIZE as u64).min(file_len);
        file.seek(SeekFrom::End(-(read_size as i64)))?;
        file.read_exact(&mut buf[..read_size as usize])?;
        buf[..Self::TAIL_SIZE][read_size as usize..].fill(0);
        hasher.update(&buf[..Self::TAIL_SIZE]);

        // read 100 bytes at file middle; coerced to <file_size> if needed
        let read_size = (Self::MIDDLE_SIZE as u64).min(file_len);
        let start = if file_len < Self::MIDDLE_SIZE as u64 {
            0_u64
        } else {
            file_len / 2 - Self::MIDDLE_SIZE as u64 / 2
        };

        file.seek(SeekFrom::Start(start))?;
        file.read_exact(&mut buf[..read_size as usize])?;
        buf[..Self::MIDDLE_SIZE][read_size as usize..].fill(0);
        hasher.update(&buf[..Self::MIDDLE_SIZE]);

        progress(Self::TOTAL_SIZE);

        Ok(hasher.finalize_fixed().into())
    }
}

pub fn group_by_hash<'a, H, FH: FileHash<H> + 'static, G, I>(
    entries_iter_getter: G,
) -> io::Result<Vec<([u8; H::OutputSize::USIZE], Vec<FileEntry>)>>
where
    H: FixedDigest,
    [(); H::OutputSize::USIZE]:,
    [u8; H::OutputSize::USIZE]: From<GenericArray<u8, H::OutputSize>>,
    G: Fn() -> I,
    I: Iterator<Item = &'a [FileEntry]>,
{
    let total_file_size = if TypeId::of::<FH>() == TypeId::of::<FileFragmentsHasher>() {
        entries_iter_getter()
            .map(|x| x.len() as u64 * FileFragmentsHasher::TOTAL_SIZE as u64)
            .sum::<u64>()
    } else {
        entries_iter_getter()
            .map(|x| x.iter().map(|x| x.size).sum::<u64>())
            .sum::<u64>()
    };

    let mut groups = Vec::new();

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
            if group.len() >= 2 {
                groups.push((hash, group));
            }
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

const fn max(a: usize, b: usize) -> usize {
    [a, b][(a < b) as usize]
}

pub fn unique_by_hardlinks(entries: &[FileEntry]) -> Vec<FileEntry> {
    let mut inode_none = Vec::from_iter(
        entries
            .iter()
            .filter(|x| x.inode.is_none())
            .map(Clone::clone),
    );
    let mut inode_some = Vec::from_iter(
        entries
            .iter()
            .filter(|x| x.inode.is_some())
            .map(Clone::clone),
    );
    inode_some.par_sort_by_key(|x| x.inode.unwrap());
    inode_some.dedup_by_key(|x| x.inode.unwrap());
    inode_none.extend(inode_some);
    inode_none
}

pub fn group_redundant_size(groups: &[Group]) -> u64 {
    groups
        .iter()
        .map(|x| x.file_size * (x.files.len() as u64 - 1))
        .sum::<u64>()
}

pub fn print_redundant_size(groups: &[Group]) {
    eprintln!(
        "Redundant size: {}",
        bytesize::to_string(group_redundant_size(groups), true)
    );
}
