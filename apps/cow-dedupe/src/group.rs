use anyhow::anyhow;
use bytesize::ByteSize;
use colored::Colorize;
use std::cmp::Reverse;
use std::fs::File;
use std::io;
use std::io::{stdout, Read};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use bczhc_lib::mutex_lock;
use bczhc_lib::str::GenericOsStrExt;
use digest::generic_array::GenericArray;
use digest::typenum::Unsigned;
use indicatif::{ProgressBar, ProgressStyle};
use once_cell::sync::Lazy;
use rayon::prelude::ParallelSliceMut;
use sha2::{Sha256, Sha512};
use sha3::{Sha3_256, Sha3_512};

use crate::cli::{GroupArgs, HashFn, OutputFormat};
use crate::hash::{FixedDigest, B3_1024, B3_128, B3_160, B3_2048, B3_256, B3_512};
use crate::serde::{build_output, Output};
use crate::{group_by_content, group_by_size};

static ARGS: Lazy<Mutex<Option<GroupArgs>>> = Lazy::new(|| Mutex::new(None));

pub fn main(args: GroupArgs) -> anyhow::Result<()> {
    mutex_lock!(ARGS).replace(args.clone());

    if let Some(path) = args.input_file {
        // input file is present; only parse and print them
        parse_and_print(path)?;
        return Ok(());
    }

    let min_size = match &args.common.min_size.parse::<ByteSize>() {
        Ok(s) => s.0,
        Err(e) => return Err(anyhow::anyhow!("Invalid min size: {}", e)),
    };

    let paths = &args.common.path;
    let mut entries = collect_file(paths, min_size)?;
    eprintln!("File entries: {}", entries.len());
    eprintln!("Grouping by size...");
    let mut groups = group_by_size(&mut entries);
    eprintln!("Group count: {}", groups.len());

    match args.common.hash_fn {
        HashFn::B3_128 => generic_group_files_by_hash::<B3_128>(&mut groups),
        HashFn::B3_160 => generic_group_files_by_hash::<B3_160>(&mut groups),
        HashFn::B3_256 => generic_group_files_by_hash::<B3_256>(&mut groups),
        HashFn::B3_512 => generic_group_files_by_hash::<B3_512>(&mut groups),
        HashFn::B3_1024 => generic_group_files_by_hash::<B3_1024>(&mut groups),
        HashFn::B3_2048 => generic_group_files_by_hash::<B3_2048>(&mut groups),
        HashFn::Sha256 => generic_group_files_by_hash::<Sha256>(&mut groups),
        HashFn::Sha512 => generic_group_files_by_hash::<Sha512>(&mut groups),
        HashFn::Sha3_256 => generic_group_files_by_hash::<Sha3_256>(&mut groups),
        HashFn::Sha3_512 => generic_group_files_by_hash::<Sha3_512>(&mut groups),
    }?;

    Ok(())
}

fn generic_group_files_by_hash<H: FixedDigest>(
    files: &mut Vec<Vec<FileEntry>>,
) -> anyhow::Result<()>
where
    [(); H::OutputSize::USIZE]:,
    [u8; H::OutputSize::USIZE]: From<GenericArray<u8, H::OutputSize>>,
{
    eprintln!("Grouping by file fragments");
    // group_by_fragments(files)?;
    // eprintln!("File entries: {}", files.len());
    eprintln!("Grouping by file content...");
    let mut groups = group_by_content::<H>(files)?;
    eprintln!("Group count: {}", groups.len());
    let duplicated_file_group_count = groups.iter().filter(|x| x.1.len() >= 2).count();
    eprintln!("Duplicated file groups: {}", duplicated_file_group_count);

    groups.par_sort_by_key(|x| Reverse(x.1[0].size));

    // select duplicated items
    groups.retain(|x| x.1.len() >= 2);

    // print out
    let output_format = mutex_lock!(ARGS).as_ref().unwrap().output_format;
    match output_format {
        OutputFormat::Default => print_groups(&build_output(&groups)),
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(&build_output(&groups)).unwrap();
            println!("{}", json);
        }
        OutputFormat::Binary => {
            bincode::serialize_into(&mut stdout(), &build_output(&groups)).unwrap();
        }
    }

    Ok(())
}

#[derive(Clone, Debug)]
pub struct FileEntry {
    pub path: PathBuf,
    pub size: u64,
}

fn collect_file(paths: &Vec<String>, min_size: u64) -> io::Result<Vec<FileEntry>> {
    let progress_bar = ProgressBar::new_spinner();
    progress_bar.set_style(
        ProgressStyle::default_spinner()
            .template("{msg} {pos}")
            .unwrap(),
    );
    progress_bar.set_message("Collecting files".cyan().bold().to_string());

    let mut files_vec = Vec::new();
    for path in paths {
        let files = jwalk::WalkDir::new(path);
        for entry in files {
            let entry = entry?;
            if !entry.file_type().is_file() {
                continue;
            }
            let metadata = entry.metadata()?;
            let file_size = metadata.len();
            if file_size >= min_size {
                files_vec.push(FileEntry {
                    path: entry.path(),
                    size: file_size,
                });
            }
            progress_bar.inc(1);
        }
    }
    Ok(files_vec)
}

fn print_groups(output: &Output) {
    let compact_hash = !mutex_lock!(ARGS).as_ref().unwrap().full_hash;

    for x in output.groups.iter() {
        let file_count = x.files.len();

        let hash_str = if compact_hash {
            &x.hash[..40]
        } else {
            x.hash.as_str()
        };
        println!(
            "{}",
            format!(
                "{}, {} * {}",
                hash_str,
                bytesize::to_string(x.file_size, true),
                file_count
            )
            .yellow()
        );
        for x in &x.files {
            println!("{}", x.as_os_str().escape_to_string());
        }
        println!()
    }
}

fn parse_and_print<P: AsRef<Path>>(input: P) -> anyhow::Result<()> {
    let mut data = Vec::new();
    File::open(input)?.read_to_end(&mut data)?;
    if data.is_empty() {
        return Err(anyhow!("Empty input file"));
    }
    if data[0] == b'{' {
        // treat as json
        let json_str = std::str::from_utf8(&data)?;
        let output: Output = serde_json::from_str(json_str)?;
        print_groups(&output);
        return Ok(());
    }

    // binary format
    let output: Output = bincode::deserialize(&data)?;
    print_groups(&output);
    Ok(())
}
