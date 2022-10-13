use byteorder::{LittleEndian, ReadBytesExt};
use cfg_if::cfg_if;
use clap::ArgMatches;
use std::ffi::{OsStr, OsString};
use std::fs::File;
use std::io;
use std::io::{Read, Seek};
use std::mem::size_of_val;

use crate::errors::{Error, Result};
use crate::{CalcCrcChecksum, Entry, FixedStoredSize, Header, ReadFrom, FILE_MAGIC};

pub fn main(matches: &ArgMatches) -> Result<()> {
    let archive = matches.get_one::<String>("archive").unwrap();

    let mut file = File::open(archive)?;

    let header = Header::read_from(&mut file)?;
    if &header.magic_number != FILE_MAGIC {
        return Err(Error::InvalidFileType);
    }

    println!(
        "\
Version: {}
Content offset: {}",
        header.version, header.content_offset
    );

    let mut entries_reader = file
        .try_clone()
        .unwrap()
        .take(header.content_offset - Header::SIZE as u64);
    loop {
        let result = Entry::read_from(&mut entries_reader);
        let entry = match result {
            Ok(entry) => entry,
            Err(e) => {
                if let Error::Io(ref io) = e {
                    if io.kind() == io::ErrorKind::UnexpectedEof {
                        break;
                    } else {
                        return Err(e);
                    }
                } else {
                    return Err(e);
                }
            }
        };
        let checksum = entries_reader.read_u32::<LittleEndian>()?;
        if entry.crc_checksum() != checksum {
            return Err(Error::Checksum(entry));
        }

        // TODO: handle and escape special and non-printable characters
        let path_bytes = &entry.path[..];
        cfg_if! {
            if #[cfg(unix)] {
                use std::os::unix::ffi::OsStrExt;
                let str = OsStr::from_bytes(path_bytes);
                println!("{:?}", str);
            } else {
                let str = std::str::from_utf8(path_bytes).expect("Invalid UTF-8 path name meets");
                println!("{}", str);
            }
        }
    }
    if file.stream_position()? != header.content_offset {
        return Err("Unexpected entries end position in file".into());
    }

    Ok(())
}
