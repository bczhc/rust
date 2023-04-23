use std::ffi::OsStr;
use std::fs::remove_file;
use std::process::{Command, Stdio};

use anyhow::anyhow;
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};

use crate::cli::DedupeArgs;
use crate::group::collect_and_group_files;
use crate::{parse_input_file, print_redundant_size};

macro_rules! os_str {
    ($s:expr) => {
        OsStr::new($s)
    };
}

pub fn main(args: DedupeArgs) -> anyhow::Result<()> {
    let groups = match args.common.input_file {
        None => collect_and_group_files(&args.common)?,
        Some(f) => parse_input_file(&f)?,
    };

    print_redundant_size(&groups);

    let operation_count = groups.iter().map(|x| x.files.len() as u64 - 1).sum::<u64>();
    let pb = if args.dry_run {
        None
    } else {
        let pb = ProgressBar::new(operation_count);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{msg} {bar:50} {pos}/{len}")
                .unwrap(),
        );
        pb.set_message("Reflinking".cyan().bold().to_string());
        Some(pb)
    };

    // TODO: to many messy branches
    for group in groups {
        let files = &group.files;
        let src = &files[0];
        for dest in files.iter().skip(1) {
            if let Some(ref x) = pb {
                x.inc(1)
            };
            if args.use_cp_cmd.yes() {
                // use `cp` command
                let cmd = [
                    os_str!("cp"),
                    os_str!("--reflink"),
                    // archive mode
                    os_str!("-a"),
                    src.as_os_str(),
                    dest.as_os_str(),
                ];

                if args.dry_run {
                    println!("{:?}", cmd);
                } else {
                    let mut child = Command::new(cmd[0])
                        .args(&cmd[1..])
                        .stdin(Stdio::null())
                        .stdout(Stdio::inherit())
                        .stderr(Stdio::inherit())
                        .spawn()?;
                    let status = child.wait()?;
                    if !status.success() {
                        return Err(anyhow!(
                            "Program exited with non-zero status: {}; cmd: {:?}",
                            status,
                            cmd
                        ));
                    }
                }
            } else {
                // use `reflink` crate
                // TODO: by this approach I'm not familiar about its internal details
                //  and have some trouble preserving the file timestamp, which is
                //  important in my use case. So I by default choose using
                //  `ls --reflink -a` command.
                if args.dry_run {
                    println!("{:?} -> {:?}", src, dest);
                } else {
                    // first the dest file should be deleted
                    remove_file(dest).map_err(|e| anyhow!("Dest file lost: {}, {:?}", e, dest))?;
                    reflink::reflink(src, dest).map_err(|e| {
                        anyhow!("Failed to reflink: {}, {:?} -> {:?}", e, src, dest)
                    })?;
                    if !dest.exists() {
                        return Err(anyhow!(
                            "Check failed: destination file doesn't exist: {:?}",
                            dest
                        ));
                    }
                }
            }
        }
    }

    Ok(())
}
