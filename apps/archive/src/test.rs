use std::ffi::OsStr;
use std::io;
use std::process::exit;

use bczhc_lib::str::GenericOsStrExt;
use clap::ArgMatches;
use crc_lib::Crc;

use crate::errors::*;
use crate::info::print_info;
use crate::line_progress::LineProgress;
use crate::reader::ArchiveReader;
use crate::{DigestWriter, FileType, FILE_CRC_64};

pub fn main(matches: &ArgMatches) -> Result<()> {
    let mut has_error = false;

    let archive = matches.get_one::<String>("archive").unwrap();

    let mut archive = ArchiveReader::new(archive)?;
    print_info(&archive.header)?;
    println!();

    eprintln!("Collecting entries...");
    let entries = archive.entries().collect::<Vec<_>>();
    eprintln!("Testing...");
    let mut line_progress = LineProgress::new(entries.len() as u64);

    for entry in entries.into_iter().enumerate() {
        let progress = entry.0 as u64 + 1;
        let entry = entry.1;

        let entry = match entry {
            Ok(e) => e,
            Err(Error::Io(e)) => {
                return Err(Error::Io(e));
            }
            Err(Error::Checksum(entry)) => {
                line_progress.message(&format!("Entry checksum error: {:?}", entry))?;
                has_error = true;
                continue;
            }
            Err(e) => {
                line_progress.message(&format!("Other entry error: {:?}", e))?;
                has_error = true;
                continue;
            }
        };

        let path_name = OsStr::from_bytes(&entry.path).escape_to_string();
        line_progress.update(progress, path_name.clone())?;

        if entry.file_type != FileType::Regular {
            continue;
        }

        let content_checksum = entry.content_checksum;

        let crc = Crc::<u64>::new(&FILE_CRC_64);
        let mut digest = crc.digest();
        let mut crc_writer = DigestWriter::<u64>::new(&mut digest);

        let mut content_reader = archive.retrieve_content(entry.offset, entry.stored_size);
        io::copy(&mut content_reader, &mut crc_writer)?;

        if content_checksum != digest.finalize() {
            line_progress.message(&format!("Content checksum error: {}", path_name))?;
            has_error = true;
        }
    }

    println!();
    if has_error {
        println!("Test done; error occurred.");
        exit(1)
    } else {
        println!("Test OK!");
        Ok(())
    }
}
