pub type Result<R> = std::result::Result<R, Error>;

use quick_error::quick_error;

quick_error! {
    #[derive(Debug)]
    pub enum Error {
        BmpError(err: bmp::BmpError) { from() }
        IoError(err: std::io::Error) { from() }
        ImageError(err: image::ImageError) { from() }
    }
}
