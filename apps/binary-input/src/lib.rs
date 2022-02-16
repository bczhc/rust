#[macro_use]
extern crate quick_error;

mod errors;
pub use errors::*;

#[cfg(unix)]
pub mod unix;
