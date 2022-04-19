use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("IllegalArgument")]
    IllegalArgument,
    #[error("{0}")]
    Serial(#[from] serialport::Error),
}
