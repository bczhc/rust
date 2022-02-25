#[derive(Debug)]
pub enum Error {
    IoError(std::io::Error),
    FormatError(FormatError),
    WavError(hound::Error),
    UsageError(String),
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::IoError(e)
    }
}

impl From<hound::Error> for Error {
    fn from(e: hound::Error) -> Self {
        match e {
            hound::Error::IoError(io_err) => Error::IoError(io_err),
            _ => Error::WavError(e),
        }
    }
}

#[derive(Debug)]
pub enum FormatError {
    NoEnoughSamples,
}

impl From<FormatError> for Error {
    fn from(e: FormatError) -> Self {
        Error::FormatError(e)
    }
}
