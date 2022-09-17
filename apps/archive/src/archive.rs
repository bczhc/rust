use crate::errors::{Error, Result};
use crate::{Entry, Header, WriteTo, FILE_MAGIC, VERSION};
use byteorder::{LittleEndian, WriteBytesExt};
use std::fs::File;
use std::io::{BufWriter, Seek, SeekFrom, Write};
use std::mem::size_of_val;
use std::path::Path;

pub struct Archive {
    writer: BufWriter<File>,
}

impl Archive {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Archive> {
        let file = File::options()
            .truncate(true)
            .create(true)
            .read(true)
            .write(true)
            .open(path)?;
        let writer = BufWriter::new(file);

        let mut archive = Self { writer };
        archive.init_header()?;
        Ok(archive)
    }

    pub fn add_path<P: AsRef<Path>>(&mut self, base_path: P, path: P) -> Result<()> {
        let relative_path = pathdiff::diff_paths(&path, &base_path).ok_or(Error::InvalidBaseDir)?;
        let metadata = path.as_ref().symlink_metadata()?;

        // Entry {};

        println!("{:?}", metadata);

        Ok(())
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
            .seek(SeekFrom::Start((size_of_val(FILE_MAGIC) + 2) as u64))?;

        self.writer.write_u64::<LittleEndian>(offset)?;

        self.writer.seek(SeekFrom::Start(position))?;
        Ok(())
    }
}

impl Drop for Archive {
    /// finalize the archive
    fn drop(&mut self) {}
}
