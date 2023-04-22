use std::ffi::{OsStr, OsString};
use std::fs::{File, Permissions};
use std::io::stdout;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use std::{fs, os};

use cfg_if::cfg_if;
use chrono::{TimeZone, Utc};
use clap::ArgMatches;
use filetime::FileTime;

use bczhc_lib::io::OpenOrCreate;
use bczhc_lib::str::GenericOsStrExt;

use crate::compressors::{create_decompressor, Decompress, ExternalFilter};
use crate::errors::*;
use crate::reader::ArchiveReader;
use crate::{Compression, FileType, LocalResultExt};

pub fn main(matches: &ArgMatches) -> Result<()> {
    let archive_path = matches.get_one::<String>("archive").unwrap();
    let base_dir = matches.get_one::<String>("base-dir").unwrap();
    let external_filter_cmd = matches
        .get_many::<String>("data-filter-cmd")
        .map(|values| values.map(|x| x.to_owned()).collect::<Vec<_>>());
    let paths = matches
        .get_many::<String>("paths")
        .map(|x| x.map(OsString::from).collect::<Vec<_>>());
    let pipe_mode = matches.get_flag("pipe");

    let mut archive = ArchiveReader::new(archive_path)?;

    let header = archive.header.clone();
    let entries = archive.entries();

    if pipe_mode && paths.is_none() && header.entry_count >= 2 {
        return Err("When in pipe mode, at least one <path> argument should be present".into());
    }

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

        let decompressor: Box<dyn Decompress> = match &header.compression {
            Compression::External => match external_filter_cmd {
                None => {
                    return Err(Error::MissingDecompressor);
                }
                Some(ref cmd) => Box::new(ExternalFilter::new(cmd)),
            },
            _ => create_decompressor(header.compression),
        };

        if pipe_mode {
            // only support regular file
            // TODO: decompress if the data is compressed
            if entry.file_type == FileType::Regular {
                let mut stdout = stdout();
                let mut reader = archive.retrieve_content(entry.offset, entry.stored_size);
                decompressor.decompress_to(&mut reader, &mut stdout)?;
            }
            drop(decompressor);
            continue;
        }

        // write to files ↓↓
        println!("{}", path.as_os_str().escape_to_string());

        match entry.file_type {
            FileType::Regular => {
                let stored_size = entry.stored_size;

                if let Some(prefix) = target_path.parent() {
                    if !prefix.exists() {
                        fs::create_dir_all(prefix)?;
                    }
                }
                let mut file = File::open_or_create(target_path)?;

                let mut content_reader = archive.retrieve_content(entry.offset, stored_size);
                decompressor.decompress_to(&mut content_reader, &mut file)?;
                drop(decompressor);

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
                let time = Utc
                    .timestamp_opt(
                        entry.modification_time.seconds,
                        entry.modification_time.nanoseconds,
                    )
                    .check()?;
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
