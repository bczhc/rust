use crate::errors::*;
use crate::{CalcCrcChecksum, Entry, GetStoredSize, Header, ReadFrom, FILE_MAGIC};
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
    file: File,
    entries_reader: Take<File>,
    content_offset: u64,
}

impl Entries {
    fn new(outer: &ArchiveReader) -> Self {
        let header = &outer.header;
        // constrain to the entries section
        let entries_reader = outer
            .file
            .try_clone()
            .unwrap()
            .take(header.content_offset - header.stored_size() as u64);

        Self {
            file: outer.file.try_clone().unwrap(),
            entries_reader,
            content_offset: outer.header.content_offset,
        }
    }
}

impl Iterator for Entries {
    type Item = Result<Entry>;

    fn next(&mut self) -> Option<Self::Item> {
        let entries_reader = &mut self.entries_reader;

        let result = Entry::read_from(entries_reader);
        let entry = match result {
            Ok(entry) => entry,
            Err(e) => {
                return if let Error::Io(ref io) = e {
                    if io.kind() == io::ErrorKind::UnexpectedEof {
                        // no more entries to read; the stream position is expected to meet
                        // the end of the entries section
                        fn check(outer: &mut Entries) -> Result<()> {
                            if outer.file.stream_position()? != outer.content_offset {
                                return Err("Unexpected entries end position in file".into());
                            }
                            Ok(())
                        }
                        if let Err(e) = check(self) {
                            return Some(Err(e));
                        }
                        None
                    } else {
                        Some(Err(e))
                    }
                } else {
                    Some(Err(e))
                };
            }
        };
        let checksum = entries_reader.read_u32::<LittleEndian>();
        if let Err(e) = checksum {
            return Some(Err(e.into()));
        }
        if entry.crc_checksum() != checksum.unwrap() {
            return Some(Err(Error::Checksum(entry)));
        }
        Some(Ok(entry))
    }
}
