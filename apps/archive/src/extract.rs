use std::ffi::OsStr;
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;

use cfg_if::cfg_if;
use clap::ArgMatches;

use bczhc_lib::io::OpenOrCreate;

use crate::compressors::{create_decompressor, Decompress, ExternalFilter};
use crate::errors::*;
use crate::reader::ArchiveReader;
use crate::{Compressor, FileType, GenericOsStrExt, StreamPipe};

pub fn main(matches: &ArgMatches) -> Result<()> {
    let archive_path = matches.get_one::<String>("archive").unwrap();
    let base_dir = matches.get_one::<String>("base-dir").unwrap();
    let external_filter_cmd = matches
        .get_many::<String>("data-filter-cmd")
        .map(|values| values.map(|x| x.to_owned()).collect::<Vec<_>>());

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

                let decompressor: Box<dyn Decompress> = match &entry.compression_method {
                    Compressor::External => match external_filter_cmd {
                        None => {
                            return Err(Error::MissingDecompressor);
                        }
                        Some(ref cmd) => Box::new(ExternalFilter::new(cmd)),
                    },
                    _ => create_decompressor(entry.compression_method),
                };
                let mut content_reader = archive.retrieve_content(abs_offset, stored_size)?;
                decompressor.decompress_to(&mut content_reader, &mut file)?;
                content_reader.finish()?;
            }
            FileType::Link => {
                todo!()
            }
            FileType::Symlink => {
                todo!()
            }
            FileType::Fifo => {
                cfg_if! {
                    if #[cfg(unix)] {
                        use nix::sys::stat::Mode;
                        nix::unistd::mkfifo(
                            target_path,
                            Mode::from_bits(entry.permission_mode as nix::libc::mode_t).unwrap(),
                        )?;
                    } else {
                        panic!("Not supported yet")
                    }
                }
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
        println!(
            "./{}{}",
            path.to_string(),
            if !path.is_empty() && entry.file_type == FileType::Directory {
                "/"
            } else {
                ""
            }
        );
    }

    Ok(())
}
