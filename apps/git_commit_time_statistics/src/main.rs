use bczhc_lib::io::ReadLine;
use chrono::format::Fixed::{TimezoneName, TimezoneOffset};
use chrono::format::Numeric::Timestamp;
use chrono::{Date, DateTime, FixedOffset, Local, NaiveDate, Offset, TimeZone, Timelike, Utc};
use clap::{App, Arg};
use git2::{Repository, RepositoryOpenFlags, Worktree};
use std::env::current_dir;
use std::fs::File;
use std::io::{Cursor, Read, Stdin};
use std::num::ParseIntError;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::SystemTime;

fn main() -> MyResult<()> {
    let matches = App::new("git-commit-time")
        .author("bczhc <bczhc0@126.com>")
        .about("List the hours of the git commits author time")
        .arg(
            Arg::with_name("repo-path")
                .required(false)
                .help("Path of a git repository"),
        )
        .get_matches();

    let mut frequency_arr = [0_usize; 24];

    let repository_dir = if matches.is_present("repo-path") {
        String::from(matches.value_of("repo-path").unwrap())
    } else {
        let current_dir = current_dir()?;
        String::from(current_dir.to_str().unwrap())
    };

    let repository = open_repo_including_parents(&repository_dir)?;

    let mut walker = repository.revwalk()?;
    walker.push_head()?;
    for oid in walker {
        let oid = oid?;
        let commit = repository.find_commit(oid)?;
        let author = commit.author();
        let author_time = author.when();
        let timezone_offset_secs = author_time.offset_minutes() * 60;
        let timestamp = author_time.seconds();

        let timezone = FixedOffset::east(timezone_offset_secs);
        let time = Utc.timestamp(timestamp as i64, 0).with_timezone(&timezone);
        let hour = time.hour();
        frequency_arr[hour as usize] += 1;
    }

    for i in 0..frequency_arr.len() {
        println!("{}: {}", i, frequency_arr[i]);
    }

    Ok(())
}

fn open_repo_including_parents(path: &str) -> MyResult<Repository> {
    let mut repository_path = Path::new(&path);
    let mut dir = path;
    loop {
        let repo = Repository::open(dir);
        if let Ok(repo) = repo {
            return Ok(repo);
        }

        let parent = repository_path.parent();
        match parent {
            None => {
                break;
            }
            Some(parent) => {
                // TODO: invalid UTF-8 check (Option `None` value)
                dir = parent.to_str().unwrap();
                repository_path = parent;
            }
        }
    }
    // not found the git repository (including parent directories)
    Err(Error::GitError(String::from(
        "not a git repository (or any of the parent directories): .git",
    )))
}

type MyResult<T> = Result<T, Error>;

#[derive(Debug)]
enum ParseError {
    ParseIntError(std::num::ParseIntError),
}

#[derive(Debug)]
enum Error {
    IOError(std::io::Error),
    GitError(String),
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::IOError(e)
    }
}

impl From<git2::Error> for Error {
    fn from(e: git2::Error) -> Self {
        Error::GitError(String::from(e.message()))
    }
}
