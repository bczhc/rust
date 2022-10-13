pub type Result<T> = std::result::Result<T, Error>;

use crate::Entry;
use std::io;
use thiserror::Error;

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
    #[error("{0}")]
    Others(String),
}

impl From<&str> for Error {
    fn from(msg: &str) -> Self {
        Self::Others(msg.into())
    }
}
