extern crate core;
extern crate crc as crc_lib;

use std::io;
use std::io::{Read, Write};
use std::mem::{size_of, size_of_val};
use std::str::FromStr;

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use cfg_if::cfg_if;
use crc_lib::{Algorithm, Crc, Digest, Width};

use bczhc_lib::io::duplicator::StreamDuplicator;
use errors::Result;

use crate::crc::DigestWriter;

pub mod archive;
pub mod compressors;
pub mod crc;
pub mod create;
pub mod errors;
pub mod list;

pub struct Entry<'a> {
    magic_number: [u8; 5],
    path_length: u16,
    path: &'a [u8],
    file_type: FileType,
    compression_method: Compressor,
    stored_size: u64,
    original_size: u64,
    owner_id: u16,
    group_id: u16,
    permission_mode: u16,
    modification_time: u64,
    content_checksum: u64,
    entry_checksum: u32,
}

#[derive(Copy, Clone)]
pub enum Compressor {
    Gzip = 0,
    Xz = 1,
    Zstd = 2,
    None = 3,
    External = 4,
}

#[derive(Copy, Clone)]
pub enum FileType {
    Regular = 0,
    Link = 1,
    Symlink = 2,
    Fifo = 3,
    Directory = 4,
}

pub struct Header {
    magic_number: [u8; 13],
    version: u16,
    content_offset: u64,
}

trait WriteTo {
    fn write_to<W: Write>(&self, writer: &mut W) -> io::Result<()>;
}

trait ReadFrom {
    type Item;

    fn read_from<R: Read>(reader: &mut R) -> io::Result<Self::Item>;
}

impl ReadFrom for Header {
    type Item = Self;

    fn read_from<R: Read>(reader: &mut R) -> io::Result<Self::Item> {
        let mut mgc_num_buf = [0_u8; FILE_MAGIC.len()];
        reader.read_exact(&mut mgc_num_buf)?;
        let version = reader.read_u16::<LittleEndian>()?;
        let content_offset = reader.read_u64::<LittleEndian>()?;

        Ok(Self {
            magic_number: mgc_num_buf,
            version,
            content_offset,
        })
    }
}

impl WriteTo for Header {
    fn write_to<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(&self.magic_number)?;
        writer.write_u16::<LittleEndian>(self.version)?;
        writer.write_u64::<LittleEndian>(self.content_offset)?;
        Ok(())
    }
}

pub struct Options {}

impl FromStr for Compressor {
    type Err = ();

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let compressor = match s.to_lowercase().as_str() {
            "gzip" => Compressor::Gzip,
            "xz" => Compressor::Xz,
            "zstd" => Compressor::Zstd,
            "no" => Compressor::None,
            _ => {
                return Err(());
            }
        };
        Ok(compressor)
    }
}

impl Compressor {
    pub fn best_level(&self) -> u32 {
        match self {
            Compressor::Gzip => 9,
            Compressor::Xz => 9,
            Compressor::Zstd => 22,
            Compressor::None => 0,
            Compressor::External => panic!("Unexpected method"),
        }
    }
}

impl<'a> WriteTo for Entry<'a> {
    fn write_to<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        let crc = Crc::<u64>::new(&FILE_CRC_64);
        let mut digest = crc.digest();
        let mut digest_writer = DigestWriter::<u64>::new(&mut digest);
        let mut crc_writer = StreamDuplicator::new(writer, &mut digest_writer);

        crc_writer.write_all(&self.magic_number)?;
        crc_writer.write_u16::<LittleEndian>(self.path_length)?;
        crc_writer.write_all(self.path)?;
        crc_writer.write_u8(self.file_type as u8)?;
        crc_writer.write_u8(self.compression_method as u8)?;
        crc_writer.write_u64::<LittleEndian>(self.stored_size)?;
        crc_writer.write_u64::<LittleEndian>(self.original_size)?;
        crc_writer.write_u16::<LittleEndian>(self.owner_id)?;
        crc_writer.write_u16::<LittleEndian>(self.group_id)?;
        crc_writer.write_u16::<LittleEndian>(self.permission_mode)?;
        crc_writer.write_u64::<LittleEndian>(self.modification_time)?;

        crc_writer.flush()?;

        let digest = digest.finalize();

        // checksum for content (placeholder)
        writer.write_u64::<LittleEndian>(0)?;
        // checksum for this entry
        writer.write_u64::<LittleEndian>(digest)?;

        Ok(())
    }
}

impl TryFrom<std::fs::FileType> for FileType {
    type Error = ();

    fn try_from(t: std::fs::FileType) -> std::result::Result<Self, Self::Error> {
        let option = t
            .is_file()
            .then(|| FileType::Regular)
            .or_else(|| t.is_symlink().then(|| FileType::Symlink))
            .or_else(|| t.is_dir().then(|| FileType::Directory));

        if let Some(t) = option {
            Ok(t)
        } else {
            cfg_if! {
                if #[cfg(unix)] {
                    use std::os::unix::fs::FileTypeExt;
                    if t.is_fifo() {
                        return Ok(FileType::Fifo);
                    }
                }
            }
            Err(())
        }
    }
}

#[derive(Default)]
struct Configs {
    compressor_type: Option<Compressor>,
}

pub const FILE_MAGIC: &[u8; 13] = b"bczhc archive";
pub const ENTRY_MAGIC: &[u8; 5] = b"Entry";
pub const VERSION: u16 = 1;

pub const FILE_CRC_64: Algorithm<u64> = crc_lib::CRC_64_XZ;
pub const HEADER_CRC_32: Algorithm<u32> = crc_lib::CRC_32_CKSUM;
