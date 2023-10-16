//! # Archive
//!
//! ## Usage steps:
//! 1. add path records, initialize the file entries
//! 2. write files and update file entries
//! 3. finalize: write entries to the starting part of the output file

use std::collections::HashMap;
use std::ffi::{OsStr, OsString};
use std::fs::File;
use std::io::{BufReader, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::{fs, io};

use byteorder::{LittleEndian, WriteBytesExt};
use cfg_if::cfg_if;
use chrono::{DateTime, Utc};
use crc_lib::Crc;

use bczhc_lib::field_size;
use bczhc_lib::str::GenericOsStrExt;

use crate::compressors::Compress;
use crate::crc::write::CrcFilter;
use crate::errors::Result;
use crate::{
    CalcCrcChecksum, Compression, Entry, EntryChecksum, FileType, FixedStoredSize, GetStoredSize,
    Header, Info, OsStrExt, Timestamp, WriteTo, ENTRY_MAGIC, FILE_CRC_64, FILE_MAGIC, VERSION,
};

pub struct Archive<'a, W>
where
    W: Write + Seek,
{
    writer: W,
    compressor: Box<dyn Compress + 'a>,
    /// the first tuple element in the vector, `PathBuf`,
    /// is the path to the file to be added,
    /// and should be opened in `write_files` method
    entries: Vec<(PathBuf, Entry)>,
    content_offset: u64,
    last_content_offset: u64,
    header: Header,
    // for identifying hard links
    inode_map: HashMap<u64, OsString>,
}

impl<'a, W> Archive<'a, W>
where
    W: Write + Seek,
{
    pub fn new(
        writer: W,
        compressor: Box<dyn Compress + 'a>,
        compression_type: Compression,
    ) -> Result<Self> {
        let header = Header {
            magic_number: *FILE_MAGIC,
            version: VERSION,
            content_offset: 0,
            compression: compression_type,
            creation_time: Utc::now().timestamp_millis(),
            entry_count: 0,         /* placeholder */
            info_json_length: 2,    /* default */
            info_json: "{}".into(), /* default */
        };

        let archive = Self {
            writer,
            compressor,
            entries: Vec::new(),
            content_offset: 0,
            last_content_offset: 0,
            header,
            inode_map: HashMap::new(),
        };
        Ok(archive)
    }

    /// add a path record
    ///
    /// `path` will be stored in the archive directly,
    /// while `file_path` is the actual path of the file to be added
    ///
    /// For example, now we are at the location '/tmp', and if we do
    /// ```bash
    /// tar -C /home -c user > ./tar
    /// ```
    /// the stored paths in this tarball will like this:
    /// - user/a.txt
    /// - user/b.txt
    /// - ...
    ///
    /// But they're not relative to our current location '/tmp', so
    /// we also need paths to these files to be added, just using absolute paths:
    /// - /home/user/a.txt
    /// - /home/user/b.txt
    /// - ...
    ///
    /// like `tar` utility
    pub fn add_path<P: AsRef<Path>>(&mut self, path: &OsStr, file_path: P) -> Result<()> {
        #[cfg(unix)]
        use std::os::unix::fs::{FileTypeExt, MetadataExt, PermissionsExt};

        let file_path = file_path.as_ref();
        let metadata = file_path.symlink_metadata()?;

        let Some(path_bytes) = path.to_bytes() else {
            panic!("Invalid path name meets");
        };

        // record inodes and detect hard links
        cfg_if! {
            if #[cfg(unix)] {
                let inode = metadata.ino();
                if let std::collections::hash_map::Entry::Vacant(e) = self.inode_map.entry(inode) {
                    e.insert(path.into());
                } else {
                    // is a hardlink, just record the linked path, and no more fields needed
                    let linked_path = self.inode_map.get(&inode).unwrap();
                    let linked_path_bytes = linked_path.as_os_str().to_bytes().unwrap();
                    let entry = Entry {
                        magic_number: *ENTRY_MAGIC,
                        path_length: path_bytes.len() as u16,
                        path: path_bytes,
                        file_type: FileType::Link,
                        linked_path_length: linked_path_bytes.len() as u16,
                        linked_path: linked_path_bytes,
                        stored_size: 0,
                        original_size: 0,
                        owner_id: 0,
                        group_id: 0,
                        permission_mode: 0,
                        modification_time: Timestamp::zero(),
                        content_checksum: 0,
                        offset: 0,
                    };
                    self.entries.push((file_path.into(), entry));
                    return Ok(());
                }
            }
        }

        let file_type = metadata.file_type();
        cfg_if! {
            if #[cfg(unix)] {
                if file_type.is_socket() {
                    eprintln!("{}: socket ignored", path.escape());
                    return Ok(());
                }
            }
        }
        let Ok(file_type) = FileType::try_from(file_type) else {
            panic!("Unknown file type: {:?}", path);
        };

        cfg_if! {
            // unix-specific attributes
            if #[cfg(unix)] {
                let file_mode = metadata.permissions().mode() as u16;

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
            .map(|x| {
                let time = DateTime::<Utc>::from(x);
                let seconds = time.timestamp();
                let nanos = time.timestamp_subsec_nanos();
                Timestamp::new(seconds, nanos)
            })
            .unwrap_or_else(|_| Timestamp::zero());

        let linked_path = if file_type == FileType::Symlink {
            // here it should be a symbolic link
            let Some(path_bytes) = ({ fs::read_link(file_path).unwrap().as_os_str().to_bytes() })
            else {
                panic!("Invalid path name meets")
            };
            path_bytes
        } else {
            vec![]
        };

        let entry = Entry {
            magic_number: *ENTRY_MAGIC,
            path_length: path_bytes.len() as u16,
            path: path_bytes,
            file_type,
            linked_path_length: linked_path.len() as u16,
            linked_path,
            stored_size: 0, /* placeholder */
            original_size: metadata.len(),
            owner_id: owner_id as u16,
            group_id: group_id as u16,
            permission_mode: file_mode,
            modification_time,
            content_checksum: 0, /* placeholder */
            offset: 0,           /* placeholder */
        };

        self.entries.push((file_path.into(), entry));
        Ok(())
    }

    pub fn set_info(&mut self, info: &Info) {
        self.header.info_json = serde_json::to_string(info).unwrap();
        self.header.info_json_length = self.header.info_json.len() as u32;
    }

    fn content_offset(&self) -> usize {
        let entries_size_sum = self
            .entries
            .iter()
            .map(|x| {
                x.1.stored_size() + EntryChecksum::SIZE /* checksum of entry, following after each entry */
            })
            .sum::<usize>();
        self.header.stored_size() + entries_size_sum
    }

    fn write_header(&mut self) -> io::Result<()> {
        self.header.entry_count = self.entries.len() as u64;

        self.header.write_to(&mut self.writer)
    }

    fn change_content_offset(&mut self, offset: u64) -> Result<()> {
        let position = self.writer.stream_position()?;
        self.writer.seek(SeekFrom::Start(
            (FILE_MAGIC.len() + field_size!(Header, version)) as u64,
        ))?;

        self.writer.write_u64::<LittleEndian>(offset)?;

        self.writer.seek(SeekFrom::Start(position))?;
        Ok(())
    }

    pub fn write(&mut self) -> Result<()> {
        self.writer.seek(SeekFrom::Start(0))?;

        self.write_header()?;
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
            let stored_path = OsStr::from_bytes(&entry.path);
            println!("{}", stored_path.escape());

            if entry.file_type != FileType::Regular {
                continue;
            }

            let file = File::open(path)?;
            let mut file_reader = BufReader::new(file);

            let crc = Crc::<u64>::new(&FILE_CRC_64);
            let mut digest = crc.digest();
            let mut crc_filter = CrcFilter::new(&mut digest, &mut self.writer);

            let compressed_size = self
                .compressor
                .compress_to(&mut file_reader, &mut crc_filter)?;

            crc_filter.flush()?;
            let content_checksum = digest.finalize();

            entry.stored_size = compressed_size;
            entry.content_checksum = content_checksum;

            entry.offset = self.last_content_offset;
            self.last_content_offset += entry.stored_size;
        }

        Ok(())
    }

    fn write_entries(&mut self) -> Result<()> {
        self.writer
            .seek(SeekFrom::Start(self.header.stored_size() as u64))?;

        for (_, entry) in &self.entries {
            let checksum = entry.crc_checksum();
            entry.write_to(&mut self.writer)?;
            self.writer.write_u32::<LittleEndian>(checksum)?;
        }

        let result = self.writer.stream_position();
        assert_eq!(self.content_offset, result?);

        Ok(())
    }

    pub fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}

impl<'a, W> Drop for Archive<'a, W>
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
            if #[cfg(unix)] {
                use std::os::unix::ffi::OsStrExt;
                return Some(Vec::from(self.as_bytes()));
            } else {
                let Some(s) = self.to_str() else { return None };
                Some(Vec::from(s.as_bytes()))
            }
        }
    }
}
