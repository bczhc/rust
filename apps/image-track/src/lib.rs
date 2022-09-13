use std::error::Error;
use std::fs::File;
use std::io;
use std::io::{Read, Write};
use std::path::Path;

use byteorder::{LittleEndian, ReadBytesExt};
use serde::{Deserialize, Serialize};

pub mod add;
pub mod create;
pub mod info;
pub mod new;

pub type AnyResult<T> = Result<T, Box<dyn Error>>;

pub const SECTOR_SIZE: u32 = 2048;
pub const HEADER: &[u8; 17] = b"bczhc image-track";
pub const HEAD_INFO_SIZE: u64 = 32 * SECTOR_SIZE as u64;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ImageInfo {
    sector_size: u32,
    tracks: Vec<Track>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Track {
    number: u32,
    start_sector: u64,
    end_sector: u64,
    used_sectors: u64,
    content_size: u64,
}

/// pad the byte size to sectors that will take place in the image
pub fn pad_sector(size: u64) -> u64 {
    let m = size % SECTOR_SIZE as u64;
    if m == 0 {
        size / SECTOR_SIZE as u64
    } else {
        (size / SECTOR_SIZE as u64) + 1
    }
}

pub fn open_image<P: AsRef<Path>>(path: P) -> AnyResult<(File, ImageInfo)> {
    let mut file = File::open(path)?;

    const INVALID_FILE_TYPE_ERR: &str = "Invalid file type";
    let mut header_buf = [0_u8; HEADER.len()];
    let result = file.read_exact(&mut header_buf);
    if let Err(ref e) = result {
        if e.kind() == io::ErrorKind::UnexpectedEof {
            return Err(INVALID_FILE_TYPE_ERR.into());
        } else {
            result?;
        }
    }
    if &header_buf != HEADER {
        return Err(INVALID_FILE_TYPE_ERR.into());
    }

    let json_len = file.read_u32::<LittleEndian>()?;
    let mut json_buf = vec![0_u8; json_len as usize];
    file.read_exact(&mut json_buf)?;

    let json = String::from_utf8(json_buf)?;
    let info: ImageInfo = serde_json::from_str(&json)?;

    Ok((file, info))
}

impl ImageInfo {
    fn new(sector_size: u32, tracks: Vec<Track>) -> ImageInfo {
        Self {
            sector_size,
            tracks,
        }
    }
}

trait PadZero<W>
where
    W: Write,
{
    fn pad_zero(&mut self, size: u64) -> io::Result<()>;
}

impl<W> PadZero<W> for W
where
    W: Write,
{
    fn pad_zero(&mut self, size: u64) -> io::Result<()> {
        for _ in 0..size {
            self.write_all(&[0])?;
        }
        Ok(())
    }
}
