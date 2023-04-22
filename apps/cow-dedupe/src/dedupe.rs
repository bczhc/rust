use crate::cli::DedupeArgs;
use crate::group::collect_and_group_files;
use anyhow::anyhow;
use std::ffi::OsStr;
use std::fs::remove_file;
use std::path::Path;
use std::process::{Command, Stdio};

macro_rules! os_str {
    ($s:expr) => {
        OsStr::new($s)
    };
}

pub fn main(args: DedupeArgs) -> anyhow::Result<()> {
    let groups = collect_and_group_files(&args.common)?;

    // TODO: to many messy branches
    // TODO: progress bar
    for group in groups {
        let files = &group.files;
        let src = &files[0];
        for dest in files.iter().skip(1) {
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
                            "Program exited with non-zero status: {}; {:?}",
                            status,
                            cmd
                        ));
                    }
                }
            } else {
                // use `reflink` crate
                if args.dry_run {
                    println!("{:?} -> {:?}", src, dest);
                } else {
                    // first the dest file should be deleted
                    remove_file(dest)?;
                    reflink::reflink(src, dest)?;
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
