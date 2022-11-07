use crate::errors::*;
use crate::{CalcCrcChecksum, Entry, GetStoredSize, Header, ReadFrom, ENTRY_MAGIC, FILE_MAGIC};
use byteorder::{LittleEndian, ReadBytesExt};

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

    pub fn retrieve_content(&mut self, offset: u64, size: u64) -> io::Result<ContentReader> {
        ContentReader::new(&mut self.file, offset, size)
    }
}

pub struct ContentReader<'a> {
    file: &'a mut File,
    init_position: u64,
    took_reader: Take<File>,
}

impl<'a> Read for ContentReader<'a> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.took_reader.read(buf)
    }
}

impl<'a> ContentReader<'a> {
    fn new(file: &'a mut File, offset: u64, size: u64) -> io::Result<Self> {
        let init_position = file.stream_position()?;

        file.seek(SeekFrom::Start(offset))?;
        let take = file.try_clone().unwrap().take(size);

        Ok(Self {
            init_position,
            file,
            took_reader: take,
        })
    }

    /// must be called after the use of `ContentReader`
    /// to seek back to the starting stream position
    pub fn finish(&mut self) -> io::Result<()> {
        self.file.seek(SeekFrom::Start(self.init_position))?;
        Ok(())
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
            let entry = Entry::read_from(&mut s.file)?;

            let checksum = s.file.read_u32::<LittleEndian>()?;
            if entry.crc_checksum() != checksum {
                return Err(Error::Checksum(entry));
            }

            s.position = s.file.stream_position()?;
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
