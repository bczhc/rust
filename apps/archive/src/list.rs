use std::ffi::OsStr;

use clap::ArgMatches;
use prettytable::{format, row, Table};

use crate::errors::Result;
use crate::info::print_info;
use crate::reader::ArchiveReader;
use crate::{Error, FileType, GenericOsStrExt, Info};

pub fn main(matches: &ArgMatches) -> Result<()> {
    let path = matches.get_one::<String>("archive").unwrap();
    let archive = ArchiveReader::new(path)?;
    let header = &archive.header;

    print_info(header)?;
    println!();

    let mut original_size_sum = 0_u64;
    let mut stored_size_sum = 0_u64;
    let mut record_count = 0_u64;

    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);

    table.set_titles(row!["Type", "Original", "Stored", "Ratio", "Path"]);

    let entries = archive.entries();
    for entry in entries {
        let entry = entry?;
        let path_bytes = &entry.path[..];
        let path = OsStr::from_bytes(path_bytes);
        let path_string = format!(
            "./{}{}",
            path.to_string(),
            if !path.is_empty() && entry.file_type == FileType::Directory {
                "/"
            } else {
                ""
            }
        );

        if entry.file_type == FileType::Regular {
            let compression_ratio = entry.original_size as f64 / entry.stored_size as f64;
            table.add_row(row![
                format!("{:?}", entry.file_type),
                entry.original_size,
                entry.stored_size,
                if compression_ratio.is_finite() {
                    format!("{:.3}", compression_ratio)
                } else {
                    "-".into()
                },
                path_string
            ]);

            stored_size_sum += entry.stored_size;
            original_size_sum += entry.original_size;
        } else {
            table.add_row(row![
                format!("{:?}", entry.file_type),
                entry.original_size,
                "-",
                "-",
                path_string
            ]);
        }
        record_count += 1;
    }

    table.printstd();
    println!();
    println!("Records: {}", record_count);
    println!("Total original size: {}", original_size_sum);
    println!("Total stored size: {}", stored_size_sum);
    let space_saving = (1.0 - stored_size_sum as f64 / original_size_sum as f64) * 100.0;
    println!(
        "Space saving: {:.3}%",
        if space_saving.is_finite() {
            space_saving
        } else {
            0.0
        }
    );

    Ok(())
}
