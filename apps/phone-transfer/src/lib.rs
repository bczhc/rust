use std::io::{stdin, Read};

pub mod errors;
pub mod receive;
pub mod send;

use errors::*;

pub const HEADER: &[u8; 8] = b"bczhc\0\0\0";

pub enum Mark {
    File = 1,
    Text = 2,
    Tar = 3,
}

