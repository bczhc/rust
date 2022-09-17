use byteorder::{LittleEndian, WriteBytesExt};
use std::io;
use std::io::{Read, Write};
use std::mem::{size_of, size_of_val};

pub mod archive;
pub mod errors;

pub struct Entry<'a> {
    magic_number: [u8; 5],
    filename_length: u16,
    filename: &'a str,
    compression_method: CompMethod,
    stored_size: u64,
    original_size: u64,
    owner_id: u16,
    group_id: u16,
    mode: u16,
    modification_time: u64,
    content_checksum: u64,
    entry_checksum: u32,
}

pub enum CompMethod {
    Gzip,
    Xz,
    Zstd,
}

pub enum FileType {
    Regular,
    Link,
    Symlink,
    Fifo,
    Directory,
}

pub struct Header {
    magic_number: [u8; 13],
    version: u16,
    content_offset: u64,
}

trait WriteTo {
    fn write_to<W: Write>(&self, writer: &mut W) -> io::Result<()>;
}

impl WriteTo for Header {
    fn write_to<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(&self.magic_number)?;
        writer.write_u16::<LittleEndian>(self.version)?;
        writer.write_u64::<LittleEndian>(self.content_offset)?;
        Ok(())
    }
}

pub trait Compress: Read + Write {
    fn new() {}
}

pub struct Compressor {}

pub struct Options {}

pub const FILE_MAGIC: &[u8; 13] = b"bczhc archive";
pub const ENTRY_MAGIC: &[u8; 5] = b"Entry";
pub const VERSION: u16 = 1;
