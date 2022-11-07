use std::ffi::{OsStr, OsString};
use std::fs::{File, Permissions};

use std::path::{Path, PathBuf};
use std::time::SystemTime;
use std::{fs, os};

use cfg_if::cfg_if;
use chrono::{TimeZone, Utc};
use clap::ArgMatches;
use filetime::FileTime;

use bczhc_lib::io::OpenOrCreate;

use crate::compressors::{create_decompressor, Decompress, ExternalFilter};
use crate::errors::*;
use crate::reader::ArchiveReader;
use crate::{Compression, FileType, GenericOsStrExt};

pub fn main(matches: &ArgMatches) -> Result<()> {
    let archive_path = matches.get_one::<String>("archive").unwrap();
    let base_dir = matches.get_one::<String>("base-dir").unwrap();
    let external_filter_cmd = matches
        .get_many::<String>("data-filter-cmd")
        .map(|values| values.map(|x| x.to_owned()).collect::<Vec<_>>());
    let paths = matches
        .get_many::<String>("paths")
        .map(|x| x.map(OsString::from).collect::<Vec<_>>());

    let mut archive = ArchiveReader::new(archive_path)?;

    let header = archive.header.clone();
    let entries = archive.entries();

    for entry in entries {
        let entry = entry?;
        let path = Path::new(OsStr::from_bytes(&entry.path));
        let target_path = &{
            let mut p = PathBuf::from(base_dir);
            p.push(path);
            p
        };

        // extract specified paths if present
        if let Some(ref paths) = paths {
            if !paths.iter().any(|x| Path::new(&x) == path) {
                continue;
            }
        }

        println!(
            "./{}{}",
            path.as_os_str().to_string(),
            if !path.as_os_str().is_empty() && entry.file_type == FileType::Directory {
                "/"
            } else {
                ""
            }
        );

        match entry.file_type {
            FileType::Regular => {
                let stored_size = entry.stored_size;

                if let Some(prefix) = target_path.parent() {
                    if !prefix.exists() {
                        fs::create_dir_all(prefix)?;
                    }
                }
                let mut file = File::open_or_create(target_path)?;

                let decompressor: Box<dyn Decompress> = match &header.compression {
                    Compression::External => match external_filter_cmd {
                        None => {
                            return Err(Error::MissingDecompressor);
                        }
                        Some(ref cmd) => Box::new(ExternalFilter::new(cmd)),
                    },
                    _ => create_decompressor(header.compression),
                };
                let mut content_reader = archive.retrieve_content(entry.offset, stored_size);
                decompressor.decompress_to(&mut content_reader, &mut file)?;

                cfg_if! {
                    if #[cfg(unix)] {
                        use os::unix::fs::PermissionsExt;
                        use nix::unistd::*;
                        if nix::unistd::getuid().is_root() {
                            fs::set_permissions(target_path, Permissions::from_mode(entry.permission_mode as u32))?;
                            chown(
                                target_path,
                                Some(Uid::from(entry.owner_id as u32)),
                                Some(Gid::from(entry.group_id as u32)),
                            )?;
                        }
                    }
                }
                let time = Utc.timestamp(
                    entry.modification_time.seconds,
                    entry.modification_time.nanoseconds,
                );
                filetime::set_file_mtime(target_path, FileTime::from(SystemTime::from(time)))?;
            }
            FileType::Link => {
                let path = Path::new(OsStr::from_bytes(&entry.path));
                let linked_path = Path::new(OsStr::from_bytes(&entry.linked_path));
                fs::hard_link(linked_path, path)?;
            }
            FileType::Symlink => {
                let path = Path::new(OsStr::from_bytes(&entry.path));
                let linked_path = Path::new(OsStr::from_bytes(&entry.linked_path));
                cfg_if! {
                    if #[cfg(windows)] {
                        todo!();
                    } else {
                        os::unix::fs::symlink(linked_path, path)?;
                    }
                }
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
                        panic!("Not supported")
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
    }

    Ok(())
}
