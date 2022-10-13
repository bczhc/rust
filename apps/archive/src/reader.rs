use crate::errors::*;
use crate::{CalcCrcChecksum, Entry, FixedStoredSize, Header, ReadFrom, FILE_MAGIC};
use byteorder::{LittleEndian, ReadBytesExt};
use cfg_if::cfg_if;
use std::ffi::OsStr;
use std::fs::File;
use std::io;
use std::io::{Read, Seek, Take};
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

    pub fn entries(&mut self) -> Entries {
        Entries::new(self)
    }
}

pub struct Entries<'a> {
    file: &'a mut File,
    entries_reader: Take<File>,
    content_offset: u64,
}

impl<'a> Entries<'a> {
    fn new(outer: &'a mut ArchiveReader) -> Self {
        let header = &outer.header;
        // constrain to the entries section
        let mut entries_reader = outer
            .file
            .try_clone()
            .unwrap()
            .take(header.content_offset - Header::SIZE as u64);

        Self {
            file: &mut outer.file,
            entries_reader,
            content_offset: outer.header.content_offset,
        }
    }
}

impl<'a> Iterator for Entries<'a> {
    type Item = Result<Entry>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut entries_reader = &mut self.entries_reader;

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
