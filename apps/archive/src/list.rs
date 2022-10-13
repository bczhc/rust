use byteorder::{LittleEndian, ReadBytesExt};
use cfg_if::cfg_if;
use clap::ArgMatches;
use std::ffi::{OsStr, OsString};
use std::fs::File;
use std::io;
use std::io::{Read, Seek};
use std::mem::size_of_val;

use crate::errors::{Error, Result};
use crate::reader::ArchiveReader;
use crate::{CalcCrcChecksum, Entry, FixedStoredSize, Header, ReadFrom, FILE_MAGIC};

pub fn main(matches: &ArgMatches) -> Result<()> {
    let path = matches.get_one::<String>("archive").unwrap();
    let mut archive = ArchiveReader::new(path)?;
    let header = &archive.header;

    println!(
        "\
Version: {}
Content offset: {}",
        header.version, header.content_offset
    );

    let entries = archive.entries();
    for entry in entries {
        let entry = entry?;
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

    Ok(())
}
