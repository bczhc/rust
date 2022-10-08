extern crate core;
extern crate crc as crc_lib;

use num_derive::FromPrimitive;
use std::io;
use std::io::{Read, Write};
use std::mem::{size_of, size_of_val};
use std::str::FromStr;

use bczhc_lib::field_size;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use cfg_if::cfg_if;
use crc_lib::{Algorithm, Crc, Digest, Width};
use once_cell::sync::Lazy;

use bczhc_lib::io::duplicator::StreamDuplicator;
use errors::Result;

use crate::crc::DigestWriter;
use crate::errors::Error;

pub mod archive;
pub mod compressors;
pub mod crc;
pub mod create;
pub mod errors;
pub mod list;

#[derive(Debug)]
pub struct Entry {
    magic_number: [u8; ENTRY_MAGIC.len()],
    path_length: u16,
    path: Vec<u8>,
    file_type: FileType,
    compression_method: Compressor,
    stored_size: u64,
    original_size: u64,
    owner_id: u16,
    group_id: u16,
    permission_mode: u16,
    modification_time: u64,
    content_checksum: u64,
}

#[derive(Copy, Clone, FromPrimitive, Debug)]
pub enum Compressor {
    Gzip = 0,
    Xz = 1,
    Zstd = 2,
    None = 3,
    External = 4,
}

#[derive(Copy, Clone, FromPrimitive, Debug)]
pub enum FileType {
    Regular = 0,
    Link = 1,
    Symlink = 2,
    Fifo = 3,
    Directory = 4,
}

trait FixedStoredSize {
    const SIZE: usize;
}

pub struct Header {
    magic_number: [u8; FILE_MAGIC.len()],
    version: u16,
    content_offset: u64,
}

impl FixedStoredSize for Header {
    const SIZE: usize = FILE_MAGIC.len() + 2 + 8;
}

trait WriteTo {
    fn write_to<W: Write>(&self, writer: &mut W) -> io::Result<()>;
}

trait ReadFrom {
    type Item;

    fn read_from<R: Read>(reader: &mut R) -> Result<Self::Item>;
}

impl ReadFrom for Header {
    type Item = Self;

    fn read_from<R: Read>(reader: &mut R) -> Result<Self::Item> {
        let mut magic_number = [0_u8; FILE_MAGIC.len()];
        reader.read_exact(&mut magic_number)?;
        let version = reader.read_u16::<LittleEndian>()?;
        let content_offset = reader.read_u64::<LittleEndian>()?;

        Ok(Self {
            magic_number,
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

impl WriteTo for Entry {
    fn write_to<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(&self.magic_number)?;
        writer.write_u16::<LittleEndian>(self.path_length)?;
        writer.write_all(&self.path)?;
        writer.write_u8(self.file_type as u8)?;
        writer.write_u8(self.compression_method as u8)?;
        writer.write_u64::<LittleEndian>(self.stored_size)?;
        writer.write_u64::<LittleEndian>(self.original_size)?;
        writer.write_u16::<LittleEndian>(self.owner_id)?;
        writer.write_u16::<LittleEndian>(self.group_id)?;
        writer.write_u16::<LittleEndian>(self.permission_mode)?;
        writer.write_u64::<LittleEndian>(self.modification_time)?;
        writer.write_u64::<LittleEndian>(self.content_checksum)?;
        Ok(())
    }
}

impl ReadFrom for Entry {
    type Item = Self;

    fn read_from<R: Read>(reader: &mut R) -> Result<Self::Item> {
        let mut magic = [0_u8; ENTRY_MAGIC.len()];
        reader.read_exact(&mut magic)?;
        if &magic != ENTRY_MAGIC {
            return Err(Error::InvalidEntryHeader);
        }

        let path_length = reader.read_u16::<LittleEndian>()?;
        let mut path = vec![0_u8; path_length as usize];
        reader.read_exact(&mut path)?;

        let file_type =
            num_traits::FromPrimitive::from_u8(reader.read_u8()?).ok_or(Error::UnknownFileType)?;
        let compression_method = num_traits::FromPrimitive::from_u8(reader.read_u8()?)
            .ok_or(Error::UnknownCompressionMethod)?;
        let stored_size = reader.read_u64::<LittleEndian>()?;
        let original_size = reader.read_u64::<LittleEndian>()?;
        let owner_id = reader.read_u16::<LittleEndian>()?;
        let group_id = reader.read_u16::<LittleEndian>()?;
        let permission_mode = reader.read_u16::<LittleEndian>()?;
        let modification_time = reader.read_u64::<LittleEndian>()?;
        let content_checksum = reader.read_u64::<LittleEndian>()?;

        Ok(Self {
            magic_number: magic,
            path_length,
            path,
            file_type,
            compression_method,
            stored_size,
            original_size,
            owner_id,
            group_id,
            permission_mode,
            modification_time,
            content_checksum,
        })
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

trait GetStoredSize {
    /// get the dynamic written size after serialization
    fn stored_size(&self) -> usize;
}

impl GetStoredSize for Entry {
    fn stored_size(&self) -> usize {
        size_of_val(&self.magic_number)
            + 2
            + self.path_length as usize
            + 1
            + 1
            + 8
            + 8
            + 2
            + 2
            + 2
            + 8
            + 8
    }
}

pub const FILE_MAGIC: &[u8; 13] = b"bczhc archive";
pub const ENTRY_MAGIC: &[u8; 5] = b"Entry";
pub const VERSION: u16 = 1;

pub const FILE_CRC_64: Algorithm<u64> = crc_lib::CRC_64_XZ;
pub const HEADER_CRC_32: Algorithm<u32> = crc_lib::CRC_32_CKSUM;
