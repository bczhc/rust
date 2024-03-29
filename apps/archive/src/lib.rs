#![feature(const_size_of_val)]
#![feature(variant_count)]

extern crate core;
extern crate crc as crc_lib;

use std::ffi::OsStr;
use std::fmt::{Debug, Display, Formatter};
use std::io::{Read, Write};
use std::str::FromStr;
use std::{io, mem};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use cfg_if::cfg_if;
use chrono::{Local, LocalResult, TimeZone};
use crc_lib::{Algorithm, Crc, Width};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

use bczhc_lib::field_size;
pub use cli::build_cli;
use errors::Result;

use crate::crc::DigestWriter;
use crate::errors::{Error, TimeError};

pub mod archive;
pub mod cli;
pub mod compressors;
pub mod crc;
pub mod create;
pub mod errors;
pub mod extract;
pub mod info;
pub mod line_progress;
pub mod list;
pub mod reader;
pub mod test;

#[derive(Debug, Clone)]
pub struct Entry {
    magic_number: [u8; ENTRY_MAGIC.len()],
    path_length: u16,
    path: Vec<u8>,
    file_type: FileType,
    linked_path_length: u16,
    linked_path: Vec<u8>,
    stored_size: u64,
    original_size: u64,
    owner_id: u16,
    group_id: u16,
    permission_mode: u16,
    modification_time: Timestamp,
    content_checksum: u64,
    // the offset in respect with the content area (0 for the first file)
    offset: u64,
}

#[repr(transparent)]
pub struct EntryChecksum(u32);

#[repr(transparent)]
pub struct ContentChecksum(u64);

impl FixedStoredSize for EntryChecksum {
    const SIZE: usize = field_size!(Self, 0);
}

impl FixedStoredSize for ContentChecksum {
    const SIZE: usize = field_size!(Self, 0);
}

#[derive(Copy, Clone, FromPrimitive, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum Compression {
    External = 0,
    None = 1,
    Gzip = 2,
    Xz = 3,
    Zstd = 4,
    Bzip2 = 5,
    Brotli = 6,
    Bzip3 = 7,
}

const COMPRESSION_COUNT: usize = mem::variant_count::<Compression>();

pub static COMPRESSIONS: Lazy<[Compression; COMPRESSION_COUNT]> = Lazy::new(|| {
    let mut index = 0;
    [(); COMPRESSION_COUNT].map(|_| {
        // the ordinal should be continuous
        index += 1;
        Compression::from_u8(index as u8 - 1).unwrap()
    })
});

pub static COMPRESSION_NAMES: Lazy<[&str; COMPRESSION_COUNT]> =
    Lazy::new(|| (*COMPRESSIONS).map(|x| x.as_str()));

impl Display for Compression {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Compression::External => "external",
            _ => self.as_str(),
        };
        f.write_str(name)
    }
}

#[derive(Copy, Clone, FromPrimitive, Debug, Eq, PartialEq)]
#[repr(u8)]
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

#[derive(Serialize, Deserialize)]
pub struct Info {
    compression_method: String,
    comment: Option<String>,
}

impl Display for Info {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Compression method: {}", self.compression_method)?;
        if let Some(ref s) = self.comment {
            writeln!(f)?;
            write!(f, "Comment: {}", s)?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct Header {
    magic_number: [u8; FILE_MAGIC.len()],
    version: u16,
    content_offset: u64,
    compression: Compression,
    creation_time: i64,
    entry_count: u64,
    info_json_length: u32,
    info_json: String,
}

impl Display for Header {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Version: {}", self.version)?;
        writeln!(f, "Content offset: {}", self.content_offset)?;
        writeln!(
            f,
            "Creation time: {}",
            Local.timestamp_millis_opt(self.creation_time).unwrap(),
        )?;
        write!(f, "Entry count: {}", self.entry_count)?;

        Ok(())
    }
}

impl GetStoredSize for Header {
    fn stored_size(&self) -> usize {
        field_size!(
            Self,
            magic_number,
            content_offset,
            version,
            compression,
            creation_time,
            entry_count,
            info_json_length
        ) + self.info_json.len()
    }
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
        let compression = reader.read_u8()?;
        let creation_time = reader.read_i64::<LittleEndian>()?;
        let entry_count = reader.read_u64::<LittleEndian>()?;
        let info_json_length = reader.read_u32::<LittleEndian>()?;
        let mut info_json_buf = vec![0_u8; info_json_length as usize];
        reader.read_exact(&mut info_json_buf)?;
        let info_json = String::from_utf8(info_json_buf)?;

        // checks
        let compression =
            FromPrimitive::from_u8(compression).ok_or(Error::UnknownCompressionMethod)?;

        Ok(Self {
            magic_number,
            version,
            content_offset,
            compression,
            creation_time,
            entry_count,
            info_json_length,
            info_json,
        })
    }
}

impl WriteTo for Header {
    fn write_to<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(&self.magic_number)?;
        writer.write_u16::<LittleEndian>(self.version)?;
        writer.write_u64::<LittleEndian>(self.content_offset)?;
        writer.write_u8(self.compression as u8)?;
        writer.write_i64::<LittleEndian>(self.creation_time)?;
        writer.write_u64::<LittleEndian>(self.entry_count)?;
        writer.write_u32::<LittleEndian>(self.info_json_length)?;
        writer.write_all(self.info_json.as_bytes())?;
        Ok(())
    }
}

pub struct Options {}

impl FromStr for Compression {
    type Err = ();

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let name = s.to_lowercase();
        match COMPRESSIONS.into_iter().find(|x| x.as_str() == name) {
            None => Err(()),
            Some(c) => Ok(c),
        }
    }
}

impl Compression {
    pub fn best_level(&self) -> u32 {
        // TODO: maybe as a property in the trait
        match self {
            Compression::Gzip => flate2::Compression::best().level(),
            Compression::Xz => 9,
            Compression::Zstd => 22,
            Compression::None => 0,
            Compression::Bzip2 => bzip2::Compression::best().level(),
            Compression::Brotli => 11,
            Compression::External => panic!("Unexpected method"),
            Compression::Bzip3 => 9,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Compression::External => "external",
            Compression::Gzip => "gzip",
            Compression::Xz => "xz",
            Compression::Zstd => "zstd",
            Compression::Bzip2 => "bzip2",
            Compression::None => "none",
            Compression::Brotli => "brotli",
            Compression::Bzip3 => "bzip3",
        }
    }
}

impl WriteTo for Entry {
    fn write_to<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(&self.magic_number)?;
        writer.write_u16::<LittleEndian>(self.path_length)?;
        writer.write_all(&self.path)?;
        writer.write_u8(self.file_type as u8)?;
        writer.write_u16::<LittleEndian>(self.linked_path.len() as u16)?;
        writer.write_all(&self.linked_path)?;
        writer.write_u64::<LittleEndian>(self.stored_size)?;
        writer.write_u64::<LittleEndian>(self.original_size)?;
        writer.write_u16::<LittleEndian>(self.owner_id)?;
        writer.write_u16::<LittleEndian>(self.group_id)?;
        writer.write_u16::<LittleEndian>(self.permission_mode)?;
        self.modification_time.write_to(writer)?;
        writer.write_u64::<LittleEndian>(self.content_checksum)?;
        writer.write_u64::<LittleEndian>(self.offset)?;
        Ok(())
    }
}

impl ReadFrom for Entry {
    type Item = Self;

    fn read_from<R: Read>(reader: &mut R) -> Result<Self::Item> {
        let mut magic_buf = [0_u8; ENTRY_MAGIC.len()];
        reader.read_exact(&mut magic_buf)?;

        let path_length = reader.read_u16::<LittleEndian>()?;
        let mut path = vec![0_u8; path_length as usize];
        reader.read_exact(&mut path)?;

        let file_type = reader.read_u8()?;

        let linked_path_length = reader.read_u16::<LittleEndian>()?;
        let mut linked_path = vec![0_u8; linked_path_length as usize];
        reader.read_exact(&mut linked_path)?;

        let stored_size = reader.read_u64::<LittleEndian>()?;
        let original_size = reader.read_u64::<LittleEndian>()?;
        let owner_id = reader.read_u16::<LittleEndian>()?;
        let group_id = reader.read_u16::<LittleEndian>()?;
        let permission_mode = reader.read_u16::<LittleEndian>()?;
        let modification_time = Timestamp::read_from(reader)?;
        let content_checksum = reader.read_u64::<LittleEndian>()?;
        let offset = reader.read_u64::<LittleEndian>()?;

        // checks
        if &magic_buf != ENTRY_MAGIC {
            return Err(Error::InvalidEntryHeader);
        }
        let file_type = FromPrimitive::from_u8(file_type).ok_or(Error::UnknownFileType)?;

        Ok(Self {
            magic_number: magic_buf,
            path_length,
            path,
            file_type,
            linked_path_length,
            linked_path,
            stored_size,
            original_size,
            owner_id,
            group_id,
            permission_mode,
            modification_time,
            content_checksum,
            offset,
        })
    }
}

impl TryFrom<std::fs::FileType> for FileType {
    type Error = ();

    fn try_from(t: std::fs::FileType) -> std::result::Result<Self, Self::Error> {
        let option = t
            .is_file()
            .then_some(FileType::Regular)
            .or_else(|| t.is_symlink().then_some(FileType::Symlink))
            .or_else(|| t.is_dir().then_some(FileType::Directory));

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
    compressor_type: Option<Compression>,
}

trait GetStoredSize {
    /// get the dynamic written size after serialization
    fn stored_size(&self) -> usize;
}

impl GetStoredSize for Entry {
    fn stored_size(&self) -> usize {
        // TODO: avoid manually adding these size values
        field_size!(Self, magic_number, path_length)
            + self.path_length as usize
            + field_size!(
                Self,
                file_type,
                stored_size,
                original_size,
                owner_id,
                group_id,
                permission_mode,
                content_checksum,
                offset
            )
            + Timestamp::SIZE
            + field_size!(Self, linked_path_length)
            + self.linked_path.len()
    }
}

impl<T> GetStoredSize for T
where
    T: FixedStoredSize,
{
    fn stored_size(&self) -> usize {
        T::SIZE
    }
}

trait CalcCrcChecksum<W>
where
    W: Width,
{
    fn crc_checksum(&self) -> W;
}

impl CalcCrcChecksum<u32> for Entry {
    fn crc_checksum(&self) -> u32 {
        let crc = Crc::<u32>::new(&ENTRY_CRC_32);
        let mut digest = crc.digest();
        let mut crc_writer = DigestWriter::<u32>::new(&mut digest);

        self.write_to(&mut crc_writer).unwrap();

        digest.finalize()
    }
}

pub struct StreamPipe<'a, W>
where
    W: Write,
{
    writer: &'a mut W,
}
impl<'a, W> StreamPipe<'a, W>
where
    W: Write,
{
    pub fn new(writer: &'a mut W) -> Self {
        Self { writer }
    }
}
impl<'a, W> Write for StreamPipe<'a, W>
where
    W: Write,
{
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.writer.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}

#[derive(Debug, Clone)]
pub struct Timestamp {
    seconds: i64,
    nanoseconds: u32,
}

impl FixedStoredSize for Timestamp {
    const SIZE: usize = field_size!(Self, seconds, nanoseconds);
}

impl Timestamp {
    fn new(seconds: i64, nanoseconds: u32) -> Timestamp {
        Self {
            seconds,
            nanoseconds,
        }
    }

    fn zero() -> Timestamp {
        Timestamp::new(0, 0)
    }
}

impl ReadFrom for Timestamp {
    type Item = Self;

    fn read_from<R: Read>(reader: &mut R) -> Result<Self::Item> {
        let seconds = reader.read_i64::<LittleEndian>()?;
        let nanoseconds = reader.read_u32::<LittleEndian>()?;
        Ok(Self {
            seconds,
            nanoseconds,
        })
    }
}

impl WriteTo for Timestamp {
    fn write_to<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_i64::<LittleEndian>(self.seconds)?;
        writer.write_u32::<LittleEndian>(self.nanoseconds)?;
        Ok(())
    }
}

pub const FILE_MAGIC: &[u8; 13] = b"bczhc archive";
pub const ENTRY_MAGIC: &[u8; 5] = b"Entry";
pub const VERSION: u16 = 0;

pub const FILE_CRC_64: Algorithm<u64> = crc_lib::CRC_64_XZ;
pub const ENTRY_CRC_32: Algorithm<u32> = crc_lib::CRC_32_CKSUM;

#[cfg(test)]
pub mod unit_test {
    use std::io::{Cursor, Seek};

    use crate::{
        Compression, Entry, FileType, GetStoredSize, Header, Timestamp, WriteTo, ENTRY_MAGIC,
        FILE_MAGIC,
    };

    fn test_size<T>(x: &T)
    where
        T: WriteTo + GetStoredSize,
    {
        let mut cursor = Cursor::new(Vec::new());
        x.write_to(&mut cursor).unwrap();
        assert_eq!(cursor.stream_position().unwrap(), x.stored_size() as u64);
    }

    #[test]
    pub fn header_size() {
        let header = Header {
            magic_number: *FILE_MAGIC,
            version: 0,
            content_offset: 0,
            compression: Compression::None,
            creation_time: 0,
            entry_count: 0,
            info_json_length: 2,
            info_json: "{}".to_string(),
        };
        test_size(&header);
    }

    pub fn entry_size() {
        let entry = Entry {
            magic_number: *ENTRY_MAGIC,
            path_length: 4,
            path: (*b"test").into(),
            file_type: FileType::Regular,
            linked_path_length: 4,
            linked_path: (*b"test").into(),
            stored_size: 0,
            original_size: 0,
            owner_id: 0,
            group_id: 0,
            permission_mode: 0,
            modification_time: Timestamp {
                seconds: 0,
                nanoseconds: 0,
            },
            content_checksum: 0,
            offset: 0,
        };
        test_size(&entry);
    }
}

pub trait LocalResultExt<T>
where
    T: Debug,
{
    fn check(self) -> Result<T>;
}

impl<T> LocalResultExt<T> for LocalResult<T>
where
    T: Debug,
{
    fn check(self) -> Result<T> {
        match self {
            LocalResult::None => Err(Error::InvalidTime(TimeError::None)),
            LocalResult::Single(d) => Ok(d),
            LocalResult::Ambiguous(d1, d2) => {
                let range_string = format!("{:?}", (d1, d2));
                Err(Error::InvalidTime(TimeError::Ambiguous(range_string)))
            }
        }
    }
}

pub trait OsStrExt {
    /// Convert raw bytes to OsStr
    ///
    /// # Panics
    ///
    /// On windows, when the data is not UTF-8 encoded, this panics!
    fn from_bytes(_: &[u8]) -> &OsStr;
}

impl OsStrExt for OsStr {
    fn from_bytes(bytes: &[u8]) -> &OsStr {
        cfg_if! {
            if #[cfg(unix)] {
                std::os::unix::ffi::OsStrExt::from_bytes(bytes)
            } else {
                let str = std::str::from_utf8(bytes).expect("Invalid UTF-8 meets");
                OsStr::new(str)
            }
        }
    }
}
