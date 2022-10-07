use crate::compressors::Compress;
use crate::errors::{Error, Result};
use crate::{Compressor, Entry, FileType, Header, WriteTo, ENTRY_MAGIC, FILE_MAGIC, VERSION};
use byteorder::{LittleEndian, WriteBytesExt};
use cfg_if::cfg_if;
use std::ffi::{OsStr, OsString};
use std::fs::{File, OpenOptions};
use std::io;
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::mem::size_of_val;
use std::os::linux::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

pub struct Archive {
    writer: BufWriter<File>,
    compressor: Box<dyn Compress>,
    paths: Vec<PathBuf>,
}

impl Archive {
    pub fn new<P: AsRef<Path>>(path: P, compressor: Box<dyn Compress>) -> Result<Archive> {
        let file = File::options()
            .truncate(true)
            .create(true)
            .read(true)
            .write(true)
            .open(path)?;
        let writer = BufWriter::new(file);

        let mut archive = Self {
            writer,
            compressor,
            paths: Vec::new(),
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

        println!("{:?}", relative_path);
        let path_bytes = relative_path.as_os_str().to_bytes();
        if path_bytes.is_none() {
            panic!("Invalid path name meets");
        }
        let path_bytes = path_bytes.unwrap();

        let file_type = FileType::try_from(metadata.file_type());
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
            path: &path_bytes,
            file_type,
            compression_method: compressor_type,
            stored_size: 0, /* placeholder */
            original_size: metadata.len(),
            owner_id: owner_id as u16,
            group_id: group_id as u16,
            permission_mode: file_mode,
            modification_time,
            content_checksum: 0, /* placeholder */
            entry_checksum: 0,   /* placeholder */
        };

        entry.write_to(&mut self.writer)?;

        self.paths.push(path.as_ref().into());

        Ok(())
    }

    fn add_record<R: Read>(&mut self, reader: &mut R, recorded_path: &str) -> Result<()> {
        let writer = &mut self.writer;

        // Entry {
        //
        // };

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
            .seek(SeekFrom::Start((FILE_MAGIC.len() + 2) as u64))?;

        self.writer.write_u64::<LittleEndian>(offset)?;

        self.writer.seek(SeekFrom::Start(position))?;
        Ok(())
    }

    pub fn write_files(&mut self) -> Result<()> {
        for path in self.paths.iter().filter(|x| x.is_file()) {
            let file = File::open(path)?;
            let mut reader = BufReader::new(file);
            io::copy(&mut reader, &mut self.writer)?;
        }
        Ok(())
    }
}

impl Drop for Archive {
    /// finalize the archive
    fn drop(&mut self) {}
}

trait ToBytes {
    fn to_bytes(&self) -> Option<Vec<u8>>;
}

impl ToBytes for OsStr {
    fn to_bytes(&self) -> Option<Vec<u8>> {
        let bytes;
        cfg_if! {
            if #[cfg(windows)] {
                let option = self.to_str();
                bytes = match option {
                    None => {
                        return None;
                    }
                    Some(s) => {
                        Vec::from(s.as_bytes())
                    }
                }
            } else {
                use std::os::unix::ffi::OsStrExt;
                bytes = Vec::from(self.as_bytes())
            }
        };
        Some(bytes)
    }
}
