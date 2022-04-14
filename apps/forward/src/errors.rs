use std::any::Any;
use std::fmt::Debug;
use std::net::AddrParseError;
use std::num::ParseIntError;

use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0:?}")]
    ParseInt(#[from] ParseIntError),
    #[error("{0:?}")]
    AddressParse(#[from] AddrParseError),
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("others")]
    Others(Box<dyn Any + Send + 'static>),
    #[error("{0}")]
    Errno(#[from] errno::Errno),
}

impl From<Box<dyn Any + Send + 'static>> for Error {
    fn from(e: Box<dyn Any + Send + 'static>) -> Self {
        Self::Others(e)
    }
}
