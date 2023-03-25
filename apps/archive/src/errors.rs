use std::fmt::Debug;
use std::io;
use std::string::FromUtf8Error;

use thiserror::Error;

use crate::Entry;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    Io(#[from] io::Error),
    #[error("Invalid base directory")]
    InvalidBaseDir,
    #[error("{0}")]
    WalkDir(#[from] walkdir::Error),
    #[error("Unknown compressor (name or level)")]
    InvalidCompressor,
    #[error("Invalid file type")]
    InvalidFileType,
    #[error("Invalid entry header")]
    InvalidEntryHeader,
    #[error("Unknown file type")]
    UnknownFileType,
    #[error("Unknown compression method")]
    UnknownCompressionMethod,
    #[error("Checksum error for entry: {0:?}")]
    Checksum(Entry),
    #[cfg(unix)]
    #[error("Unix errno: {0}")]
    Errno(#[from] nix::Error),
    #[error("External compressor non-zero exit code: {0}")]
    FilterNonZeroExit(i32),
    #[error("External decompressor not provided")]
    MissingDecompressor,
    #[error("Only use relative paths")]
    AbsolutePath,
    #[error("{0}")]
    FromUtf8(#[from] FromUtf8Error),
    #[error("Invalid info json")]
    InvalidInfoJson,
    #[error("Invalid time")]
    InvalidTime(TimeError),
    #[error("Error from compressor: {0}")]
    CompressorError(String),
    #[error("Error from decompressor: {0}")]
    DecompressorError(String),
    #[error("{0}")]
    Others(String),
}

#[derive(Error, Debug)]
pub enum TimeError {
    #[error("No such local time")]
    None,
    #[error("Ambiguous local time, range: {0:?}")]
    Ambiguous(String),
}

impl From<&str> for Error {
    fn from(msg: &str) -> Self {
        Self::Others(msg.into())
    }
}
