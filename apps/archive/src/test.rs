use crate::errors::*;
use crate::info::print_info;
use crate::reader::ArchiveReader;
use crate::{DigestWriter, FileType, GenericOsStrExt, FILE_CRC_64};
use clap::ArgMatches;
use crc_lib::Crc;
use indicatif::ProgressBar;
use std::ffi::OsStr;
use std::io;
use std::process::exit;

pub fn main(matches: &ArgMatches) -> Result<()> {
    let mut has_error = false;

    let archive = matches.get_one::<String>("archive").unwrap();

    let mut archive = ArchiveReader::new(archive)?;
    print_info(&archive.header)?;
    println!();

    let content_offset = archive.header.content_offset;

    eprintln!("Collecting entries...");
    let entries = archive.entries().collect::<Vec<_>>();
    eprintln!("Testing...");
    let progress_bar = ProgressBar::new(entries.len() as u64);

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(Error::Checksum(entry)) => {
                progress_bar.println(format!("Entry checksum error: {:?}", entry));
                has_error = true;
                progress_bar.inc(1);
                continue;
            }
            Err(e) => {
                progress_bar.inc(1);
                return Err(e);
            }
        };

        if entry.file_type != FileType::Regular {
            continue;
        }

        let path_name = OsStr::from_bytes(&entry.path).to_string();

        let content_checksum = entry.content_checksum;

        let crc = Crc::<u64>::new(&FILE_CRC_64);
        let mut digest = crc.digest();
        let mut crc_writer = DigestWriter::<u64>::new(&mut digest);

        let mut content_reader = archive.retrieve_content(entry.offset, entry.stored_size);
        io::copy(&mut content_reader, &mut crc_writer)?;

        if content_checksum != digest.finalize() {
            progress_bar.println(format!("Content checksum error: {}", path_name));
            has_error = true;
        }

        progress_bar.inc(1);
    }

    progress_bar.finish();
    if has_error {
        println!("Test done; error occurred.");
        exit(1)
    } else {
        println!("Test OK!");
        Ok(())
    }
}
