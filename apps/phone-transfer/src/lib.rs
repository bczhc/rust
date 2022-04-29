use std::io::{stdin, Read};
pub mod errors;
pub mod receive;
pub mod send;

pub const HEADER: &[u8; 8] = b"bczhc\0\0\0";

use num_derive::FromPrimitive;
#[derive(FromPrimitive)]
pub enum Mark {
    File = 1,
    Text = 2,
    Tar = 3,
}
