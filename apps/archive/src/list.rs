use clap::ArgMatches;
use std::fs::File;
use std::io;
use std::io::{Read, Seek};
use std::mem::size_of_val;

use crate::errors::{Error, Result};
use crate::{Entry, FixedStoredSize, Header, ReadFrom, FILE_MAGIC};

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
        println!("{:?}", entry);
    }
    if file.stream_position()? != header.content_offset {
        return Err("Unexpected entries end position in file".into());
    }

    Ok(())
}
