pub type Result<T> = std::result::Result<T, Error>;

use std::num::ParseIntError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    ParseInt(#[from] ParseIntError),
    #[error("{0}")]
    AddressParse(#[from] std::net::AddrParseError),
    #[error("{0}")]
    Poll(#[from] errno::Errno),
}
