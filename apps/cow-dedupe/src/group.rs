use std::cmp::Reverse;
use std::io;
use std::io::stdout;
use std::path::PathBuf;
use std::sync::Mutex;

use bytesize::ByteSize;
use colored::Colorize;
use digest::generic_array::GenericArray;
use digest::typenum::Unsigned;
use indicatif::{ProgressBar, ProgressStyle};
use once_cell::sync::Lazy;
use rayon::prelude::ParallelSliceMut;
use sha2::{Sha256, Sha512};
use sha3::{Sha3_256, Sha3_512};

use bczhc_lib::mutex_lock;
use bczhc_lib::str::GenericOsStrExt;

use crate::cli::{CommonArgs, GroupArgs, HashFn, OutputFormat};
use crate::hash::{FixedDigest, B3_1024, B3_128, B3_160, B3_2048, B3_256, B3_512};
use crate::serde::build_output;
use crate::{
    group_by_hash, group_by_size, parse_input_file, print_redundant_size, unique_by_hardlinks,
    FileFragmentsHasher, FileFullHasher, Group,
};

static ARGS: Lazy<Mutex<Option<GroupArgs>>> = Lazy::new(|| Mutex::new(None));

pub fn main(args: GroupArgs) -> anyhow::Result<()> {
    mutex_lock!(ARGS).replace(args.clone());

    if let Some(path) = args.common.input_file {
        // input file is present; only parse and print them
        let groups = parse_input_file(path)?;
        print_groups(&groups);
        return Ok(());
    }

    let groups = collect_and_group_files(&args.common)?;
    print_redundant_size(&groups);

    // print out
    let output_format = mutex_lock!(ARGS).as_ref().unwrap().output_format;
    match output_format {
        OutputFormat::Default => print_groups(&groups),
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(&build_output(groups)).unwrap();
            println!("{}", json);
        }
        OutputFormat::Binary => {
            bincode::serialize_into(&mut stdout(), &build_output(groups)).unwrap();
        }
    }

    Ok(())
}

pub fn collect_and_group_files(args: &CommonArgs) -> anyhow::Result<Vec<Group>> {
    let min_size = match args.min_size.parse::<ByteSize>() {
        Ok(s) => s.0,
        Err(e) => return Err(anyhow::anyhow!("Invalid min size: {}", e)),
    };

    let paths = &args.path;
    let entries = collect_file(paths, min_size);
    println!("{}", format!("File entries: {}", entries.len()).cyan());
    eprintln!("{}", "Removing hardlinks...".cyan());
    let mut entries = unique_by_hardlinks(&entries);
    println!("{}", format!("File entries: {}", entries.len()).cyan());
    eprintln!("{}", "Grouping by size...".cyan());
    let mut groups = group_by_size(&mut entries);
    groups.retain(|x| x.len() >= 2);
    println!(
        "{}",
        format!(
            "File entries: {}",
            groups.iter().map(|x| x.len()).sum::<usize>()
        )
        .cyan()
    );

    let groups = match args.hash_fn {
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

    Ok(groups
        .into_iter()
        .map(|g| Group {
            file_size: g.1[0].size,
            hash: hex::encode(g.0),
            files: g.1.iter().map(|x| x.path.clone()).collect(),
        })
        .collect())
}

/// returns a vec of tuples, and each tuple is (hash, duplicated files)
fn generic_group_files_by_hash<H: FixedDigest>(
    files: &mut [Vec<FileEntry>],
) -> anyhow::Result<Vec<(Vec<u8>, Vec<FileEntry>)>>
where
    [(); H::OutputSize::USIZE]:,
    [u8; H::OutputSize::USIZE]: From<GenericArray<u8, H::OutputSize>>,
{
    eprintln!("{}", "Grouping by file fragments".cyan());
    let groups =
        group_by_hash::<H, FileFragmentsHasher, _, _>(|| files.iter().map(|x| x.as_slice()))?;
    println!("{}", format!("File entries: {}", groups.len()).cyan());
    eprintln!("{}", "Grouping by file content...".cyan());
    let mut groups =
        group_by_hash::<H, FileFullHasher, _, _>(|| groups.iter().map(|x| x.1.as_slice()))?;
    println!("{}", format!("Group count: {}", groups.len()).cyan());
    let duplicated_file_group_count = groups.iter().filter(|x| x.1.len() >= 2).count();
    println!(
        "{}",
        format!("Duplicated file groups: {}", duplicated_file_group_count).cyan()
    );

    groups.par_sort_by_key(|x| Reverse(x.1[0].size));

    // select duplicated items
    groups.retain(|x| x.1.len() >= 2);

    let r = groups
        .into_iter()
        .map(|g| {
            let hash = Vec::from(g.0);
            (hash, g.1)
        })
        .collect();
    Ok(r)
}

#[derive(Clone, Debug)]
pub struct FileEntry {
    pub path: PathBuf,
    pub size: u64,
    pub inode: Option<u64>,
}

fn collect_file(paths: &Vec<String>, min_size: u64) -> Vec<FileEntry> {
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
            let result: io::Result<()> = try {
                let entry = entry?;
                if !entry.file_type().is_file() {
                    continue;
                }
                let metadata = entry.metadata()?;
                let file_size = metadata.len();
                if file_size >= min_size {
                    let mut entry = FileEntry {
                        path: entry.path(),
                        size: file_size,
                        inode: None,
                    };
                    #[cfg(unix)]
                    {
                        use std::os::unix::prelude::MetadataExt;
                        entry.inode = Some(metadata.ino());
                    }
                    files_vec.push(entry);
                }
            };
            if let Err(e) = result {
                progress_bar.println(format!("Entry read error: {}", e));
            }
            progress_bar.inc(1);
        }
    }
    files_vec
}

fn print_groups(groups: &[Group]) {
    let compact_hash = !mutex_lock!(ARGS).as_ref().unwrap().full_hash;

    for x in groups.iter() {
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
            println!("{}", x.escape());
        }
        println!()
    }
}
