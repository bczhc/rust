use crate::errors::*;
use crate::reader::ArchiveReader;
use crate::{FileType, GenericOsStrExt};
use bczhc_lib::io::OpenOrCreate;
use clap::ArgMatches;
use nix::sys::stat::Mode;
use std::ffi::OsStr;
use std::fs;
use std::fs::File;
use std::path::{Path, PathBuf};

pub fn main(matches: &ArgMatches) -> Result<()> {
    let archive_path = matches.get_one::<String>("archive").unwrap();
    let base_dir = matches.get_one::<String>("base-dir").unwrap();

    let mut archive = ArchiveReader::new(archive_path)?;
    let content_offset = archive.header.content_offset;
    let entries = archive.entries();

    for entry in entries {
        let entry = entry?;
        let path = OsStr::from_bytes(&entry.path);
        let target_path = &{
            let mut p = PathBuf::from(base_dir);
            p.push(path);
            p
        };
        match entry.file_type {
            FileType::Regular => {
                let abs_offset = content_offset + entry.offset;
                let stored_size = entry.stored_size;

                let mut file = File::open_or_create(target_path)?;
                archive.retrieve_content(&mut file, abs_offset, stored_size)?;
            }
            FileType::Link => {
                todo!()
            }
            FileType::Symlink => {
                todo!()
            }
            FileType::Fifo => {
                nix::unistd::mkfifo(
                    target_path,
                    Mode::from_bits(entry.permission_mode as nix::libc::mode_t).unwrap(),
                )?;
            }
            FileType::Directory => {
                if target_path.exists() {
                    if !(target_path.is_dir()) {
                        // overwrite completely using directory
                        // same default action as `tar` utility
                        fs::remove_file(target_path)?;
                        fs::create_dir(target_path)?;
                    }
                } else {
                    fs::create_dir(target_path)?;
                }
            }
        }
        println!("{:?}", path);
    }

    Ok(())
}
