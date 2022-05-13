use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    AddressParse(#[from] std::net::AddrParseError),
    #[error("UnsupportedFileType")]
    UnsupportedFileType,
    #[error("{0}")]
    ReadDir(#[from] walkdir::Error),
    #[error("{0}")]
    Utf8(#[from] std::str::Utf8Error),
    #[error("InvalidCharset")]
    InvalidCharset,
    #[error("UnknownFilename")]
    UnknownFilename,
    #[error("InvalidPort")]
    InvalidPort,
    #[error("InvalidHeader")]
    InvalidHeader,
    #[error("InvalidMark {0}")]
    InvalidMark(u8),
    #[error("InvalidSelect")]
    InvalidSelect,
}
