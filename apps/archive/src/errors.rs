pub type Result<T> = std::result::Result<T, Error>;

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
}
