use crate::compressors::{create_compressor, create_decompressor};
use crate::errors::*;
use crate::reader::ArchiveReader;
use crate::{Compressor, FileType, GenericOsStrExt, StreamPipe};
use bczhc_lib::io::OpenOrCreate;
use clap::ArgMatches;
use nix::sys::stat::Mode;
use std::ffi::OsStr;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::{fs, io};

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

                // let decompressor = create_decompressor(entry.compression_method);
                // here must use a write-based decoder, as the target writer
                // in `retrieve_content(...)`
                // TODO: generalize APIs; refactor
                let mut decoder: Box<dyn Write> = match entry.compression_method {
                    Compressor::Gzip => Box::new(flate2::write::GzDecoder::new(&mut file)),
                    Compressor::Xz => Box::new(xz2::write::XzDecoder::new(&mut file)),
                    Compressor::Zstd => Box::new(zstd::stream::write::Decoder::new(&mut file)?),
                    Compressor::None => Box::new(StreamPipe::new(&mut file)),
                    Compressor::External => {
                        unreachable!()
                    }
                };
                archive.retrieve_content(&mut decoder, abs_offset, stored_size)?;
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
