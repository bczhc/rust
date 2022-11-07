use crate::errors::*;
use crate::{CalcCrcChecksum, Entry, GetStoredSize, Header, ReadFrom, ENTRY_MAGIC, FILE_MAGIC};
use byteorder::{LittleEndian, ReadBytesExt};

use bczhc_lib::io::Skip;
use std::fs::File;
use std::io;
use std::io::{Read, Seek, SeekFrom, Take};
use std::path::Path;

pub struct ArchiveReader {
    file: File,
    pub header: Header,
}

impl ArchiveReader {
    pub fn new<P: AsRef<Path>>(archive: P) -> Result<Self> {
        let mut file = File::open(archive)?;

        let header = Header::read_from(&mut file)?;
        if &header.magic_number != FILE_MAGIC {
            return Err(Error::InvalidFileType);
        }
        Ok(Self { file, header })
    }

    pub fn entries(&self) -> Entries {
        Entries::new(self)
    }

    /// `offset`: offset to `content_offset`
    /// absolute offset = `offset` + `content_offset`
    pub fn retrieve_content(&mut self, offset: u64, size: u64) -> ContentReader {
        ContentReader::new(&mut self.file, self.header.content_offset, offset, size)
    }
}

pub struct ContentReader<'a> {
    file: &'a mut File,
    left_size: u64,
    position: u64,
}

impl<'a> Read for ContentReader<'a> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.file.stream_position()? != self.position {
            self.file.seek(SeekFrom::Start(self.position))?;
        }
        let read_size = self.file.take(self.left_size).read(buf)?;
        self.left_size -= read_size as u64;
        self.position += read_size as u64;

        Ok(read_size)
    }
}

impl<'a> ContentReader<'a> {
    fn new(file: &'a mut File, content_offset: u64, offset: u64, size: u64) -> Self {
        Self {
            file,
            left_size: size,
            position: content_offset + offset,
        }
    }
}

pub struct Entries {
    position: u64,
    count: u64,
    total_count: u64,
    file: File,
}

impl Entries {
    fn new(outer: &ArchiveReader) -> Self {
        let entries_start_pos = outer.header.stored_size() as u64;
        let file = outer.file.try_clone().unwrap();

        Self {
            position: entries_start_pos,
            count: 0,
            total_count: outer.header.entry_count,
            file,
        }
    }
}

impl Iterator for Entries {
    type Item = Result<Entry>;

    fn next(&mut self) -> Option<Self::Item> {
        fn try_next(s: &mut Entries) -> Result<Entry> {
            if s.file.stream_position()? != s.position {
                s.file.seek(SeekFrom::Start(s.position))?;
            }
            let result = Entry::read_from(&mut s.file);
            let checksum = s.file.read_u32::<LittleEndian>()?;
            s.position = s.file.stream_position()?;

            let entry = result?;
            if entry.crc_checksum() != checksum {
                return Err(Error::Checksum(entry));
            }

            Ok(entry)
        }

        if self.count == self.total_count {
            return None;
        }

        let result = try_next(self);
        self.count += 1;
        Some(result)
    }
}
