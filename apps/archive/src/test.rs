use crate::errors::*;
use crate::reader::ArchiveReader;
use crate::{DigestWriter, GenericOsStrExt, FILE_CRC_64};
use clap::ArgMatches;
use crc_lib::Crc;
use std::ffi::OsStr;
use std::process::exit;

pub fn main(matches: &ArgMatches) -> Result<()> {
    let mut has_error = false;

    let archive = matches.get_one::<String>("archive").unwrap();

    let mut archive = ArchiveReader::new(archive)?;
    println!("{}\n", archive.header);

    let content_offset = archive.header.content_offset;

    let entries = archive.entries();
    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(Error::Checksum(entry)) => {
                eprintln!("Entry checksum error: {:?}", entry);
                has_error = true;
                continue;
            }
            Err(e) => {
                return Err(e);
            }
        };

        let path_name = OsStr::from_bytes(&entry.path).to_string();

        let content_checksum = entry.content_checksum;
        let abs_offset = entry.offset + content_offset;

        let crc = Crc::<u64>::new(&FILE_CRC_64);
        let mut digest = crc.digest();
        let mut crc_writer = DigestWriter::<u64>::new(&mut digest);

        archive.retrieve_content(&mut crc_writer, abs_offset, entry.stored_size)?;

        if content_checksum != digest.finalize() {
            eprintln!("Content checksum error: {}", path_name);
            has_error = true;
        }
    }

    println!();
    if !has_error {
        println!("Test OK!");
        Ok(())
    } else {
        eprintln!("Check failed");
        exit(1);
    }
}
