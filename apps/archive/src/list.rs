use clap::ArgMatches;
use std::ffi::OsStr;

use crate::errors::Result;
use crate::reader::ArchiveReader;
use crate::{FileType, GenericOsStrExt};

pub fn main(matches: &ArgMatches) -> Result<()> {
    let path = matches.get_one::<String>("archive").unwrap();
    let archive = ArchiveReader::new(path)?;
    let header = &archive.header;

    println!("{}", header);

    let entries = archive.entries();
    for entry in entries {
        let entry = entry?;
        let path_bytes = &entry.path[..];
        let path = OsStr::from_bytes(path_bytes);
        println!(
            "./{}{}",
            path.to_string(),
            if !path.is_empty() && entry.file_type == FileType::Directory {
                "/"
            } else {
                ""
            }
        );

        #[cfg(debug_assertions)]
        dbg!(entry);
    }

    Ok(())
}
