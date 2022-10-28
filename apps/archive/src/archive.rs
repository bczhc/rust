//! # Archive
//!
//! ## Usage steps:
//! 1. add path records, initialize the file entries
//! 2. write files and update file entries
//! 3. finalize: write entries to the starting part of the output file

use std::ffi::{OsStr, OsString};
use std::fs::{File, OpenOptions};
use std::io;
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::mem::size_of_val;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::UNIX_EPOCH;

use byteorder::{LittleEndian, WriteBytesExt};
use cfg_if::cfg_if;

use crate::compressors::Compress;
use crate::errors::{Error, Result};
use crate::{
    CalcCrcChecksum, Compressor, Entry, FileType, FixedStoredSize, GenericOsStrExt, GetStoredSize,
    Header, WriteTo, ENTRY_MAGIC, FILE_MAGIC, VERSION,
};

pub struct Archive<W>
where
    W: Write + Seek,
{
    writer: W,
    compressor: Box<dyn Compress>,
    entries: Vec<(PathBuf, Entry)>,
    content_offset: u64,
    last_content_offset: u64,
}

impl<W> Archive<W>
where
    W: Write + Seek,
{
    pub fn new(writer: W, compressor: Box<dyn Compress>) -> Result<Self> {
        let mut archive = Self {
            writer,
            compressor,
            entries: Vec::new(),
            content_offset: 0,
            last_content_offset: 0,
        };
        archive.init_header()?;
        Ok(archive)
    }

    /// add a path record
    /// the path diff between `path` and `base_path` will be recorded in the archive file
    /// like `tar` utility
    pub fn add_path<P: AsRef<Path>>(
        &mut self,
        base_path: P,
        path: P,
        compressor_type: Compressor,
    ) -> Result<()> {
        let relative_path = pathdiff::diff_paths(&path, &base_path).ok_or(Error::InvalidBaseDir)?;
        let metadata = path.as_ref().symlink_metadata()?;

        let path_bytes = relative_path.as_os_str().to_bytes();
        if path_bytes.is_none() {
            panic!("Invalid path name meets");
        }
        let path_bytes = path_bytes.unwrap();

        let file_type = metadata.file_type();
        cfg_if! {
            if #[cfg(unix)] {
                use std::os::unix::fs::FileTypeExt;
                if file_type.is_socket() {
                    eprintln!("{}: socket ignored", relative_path.as_os_str().to_string());
                    return Ok(());
                }
            }
        }
        let file_type = FileType::try_from(file_type);
        if file_type.is_err() {
            panic!("Unknown file type: {:?}", relative_path);
        }
        let file_type = file_type.unwrap();

        cfg_if! {
            // unix-specific attributes
            if #[cfg(unix)] {
                use std::os::unix::fs::PermissionsExt;
                let file_mode = metadata.permissions().mode() as u16;

                use std::os::unix::fs::MetadataExt;
                let owner_id = metadata.uid();
                let group_id = metadata.gid();
            } else {
                let mut file_mode = 0;
                let mut owner_id = 0;
                let mut group_id = 0;
            }
        }

        let modification_time = metadata
            .modified()
            .map(|x| x.duration_since(UNIX_EPOCH).unwrap().as_millis() as u64)
            .unwrap_or(0);

        let entry = Entry {
            magic_number: *ENTRY_MAGIC,
            path_length: path_bytes.len() as u16,
            path: path_bytes,
            file_type,
            compression_method: compressor_type,
            stored_size: 0, /* placeholder */
            original_size: metadata.len(),
            owner_id: owner_id as u16,
            group_id: group_id as u16,
            permission_mode: file_mode,
            modification_time,
            content_checksum: 0, /* placeholder */
            offset: 0,           /* placeholder */
        };

        self.entries.push((relative_path, entry));
        Ok(())
    }

    fn content_offset(&self) -> usize {
        let entries_size_sum = self
            .entries
            .iter()
            .map(|x| {
                x.1.stored_size() + 4 /* checksum of entry, following after each entry */
            })
            .sum::<usize>();
        Header::SIZE + entries_size_sum
    }

    fn init_header(&mut self) -> Result<()> {
        let header = Header {
            magic_number: *FILE_MAGIC,
            version: VERSION,
            content_offset: 0,
        };
        header.write_to(&mut self.writer)?;
        Ok(())
    }

    fn change_content_offset(&mut self, offset: u64) -> Result<()> {
        let position = self.writer.stream_position()?;
        self.writer
            .seek(SeekFrom::Start((FILE_MAGIC.len() + 2) as u64))?;

        self.writer.write_u64::<LittleEndian>(offset)?;

        self.writer.seek(SeekFrom::Start(position))?;
        Ok(())
    }

    pub fn write(&mut self) -> Result<()> {
        self.write_files()?;
        self.write_entries()?;
        Ok(())
    }

    fn write_files(&mut self) -> Result<()> {
        let content_offset = self.content_offset() as u64;
        self.change_content_offset(content_offset)?;
        self.writer.seek(SeekFrom::Start(content_offset))?;

        self.content_offset = content_offset;
        self.last_content_offset = 0;

        for (path, entry) in self.entries.iter_mut() {
            println!(
                "./{}{}",
                path.as_os_str().to_string(),
                if path.is_dir() { "/" } else { "" }
            );
            if !path.is_file() {
                continue;
            }

            let file = File::open(&path)?;
            let mut reader = BufReader::new(file);
            let compressed_size = self.compressor.compress_to(&mut reader, &mut self.writer)?;
            entry.stored_size = compressed_size;

            entry.offset = self.last_content_offset;
            self.last_content_offset += entry.stored_size;
        }

        Ok(())
    }

    fn write_entries(&mut self) -> Result<()> {
        self.writer.seek(SeekFrom::Start(Header::SIZE as u64))?;

        for (_, entry) in &self.entries {
            let checksum = entry.crc_checksum();
            entry.write_to(&mut self.writer)?;
            self.writer.write_u32::<LittleEndian>(checksum)?;
        }

        assert_eq!(self.content_offset, self.writer.stream_position()?);

        Ok(())
    }

    pub fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}

impl<W> Drop for Archive<W>
where
    W: Write + Seek,
{
    /// finalize the archive
    fn drop(&mut self) {}
}

trait ToBytes {
    fn to_bytes(&self) -> Option<Vec<u8>>;
}

impl ToBytes for OsStr {
    fn to_bytes(&self) -> Option<Vec<u8>> {
        cfg_if! {
            if #[cfg(windows)] {
                let option = self.to_str();
                match option {
                    None => {
                        return None;
                    }
                    Some(s) => {
                        Some(Vec::from(s.as_bytes()))
                    }
                }
            } else {
                use std::os::unix::ffi::OsStrExt;
                return Some(Vec::from(self.as_bytes()));
            }
        }
    }
}
