use std::fmt::{Debug, Display, Formatter};
use std::io;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Io(#[from] io::Error),
    #[error("Password doesn't match")]
    PasswordNotMatch,
    #[error("{0}")]
    PasswordHash(#[from] PasswordHashError),
    #[error("Invalid salt")]
    InvalidSalt
}

#[repr(transparent)]
#[derive(Debug)]
pub struct PasswordHashError(argon2::password_hash::Error);

impl Display for PasswordHashError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{:?}", self))
    }
}

impl std::error::Error for PasswordHashError {}

impl From<argon2::password_hash::Error> for PasswordHashError {
    fn from(value: argon2::password_hash::Error) -> Self {
        Self(value)
    }
}

impl From<argon2::password_hash::Error> for Error {
    fn from(value: argon2::password_hash::Error) -> Self {
        Self::PasswordHash(value.into())
    }
}
