use bmp::BmpError;

pub type Result<R> = std::result::Result<R, Error>;

#[derive(Debug)]
pub enum Error {
    BmpError(bmp::BmpError),
    IoError(std::io::Error),
}

impl From<BmpError> for Error {
    fn from(e: BmpError) -> Self {
        Error::BmpError(e)
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::IoError(e)
    }
}
