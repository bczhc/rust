pub type Result<T> = std::result::Result<T, Error>;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("{0}")]
    Io(#[from] std::io::Error),
}
