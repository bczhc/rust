use ansi_term::Color;

use chrono::{FixedOffset, TimeZone, Timelike, Utc};
use clap::{App, Arg, ArgMatches};
use git2::Repository;
use std::env::current_dir;

use std::path::Path;

#[allow(unused)]
struct Options {
    show_commits: bool,
}

fn main() -> MyResult<()> {
    let matches = App::new("git-commit-time")
        .author("bczhc <bczhc0@126.com>")
        .about("List the hours of the git commits author time")
        .arg(
            Arg::with_name("repo-path")
                .required(false)
                .help("Path of a git repository"),
        )
        .arg(
            Arg::with_name("show-commits")
                .short("c")
                .long("show-commits"),
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

    let _options = read_options(&matches);

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
        let time = Utc.timestamp(timestamp, 0).with_timezone(&timezone);
        let hour = time.hour();
        frequency_arr[hour as usize] += 1;
    }

    print_frequency(&frequency_arr);

    Ok(())
}

fn read_options(matches: &ArgMatches) -> Options {
    Options {
        show_commits: matches.is_present("show-commits"),
    }
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

fn print_frequency(arr: &[usize]) {
    let mut max_commit_count = 0;
    let plain_print = || {
        for (hour, count) in arr.iter().enumerate() {
            println!("{}: {}", hour, count);
        }
    };

    let terminal_size = terminal_size::terminal_size();

    if terminal_size.is_none() {
        plain_print();
        return;
    }

    let mut max_left_string_len = 0;
    for (hour, count) in arr.iter().enumerate() {
        let left_string = format!("{}: {}", hour, count);
        max_left_string_len = max_left_string_len.max(left_string.len());
        max_commit_count = max_commit_count.max(*count);
    }

    let proper_width = terminal_size.unwrap().0 .0 as i32 - max_left_string_len as i32;
    if proper_width < 0 {
        plain_print();
        return;
    }
    let proper_width = proper_width as u16;
    for (hour, count) in arr.iter().enumerate() {
        let left_string = format!("{}: {}", hour, count);
        print!("{}", left_string);
        let space_count = (*count as f64 / max_commit_count as f64 * proper_width as f64) as u16;
        let mut space_string = String::with_capacity(space_count as usize);
        for _ in 0..space_count {
            space_string.push(' ');
        }
        let indent_count = max_left_string_len - left_string.len();
        for _ in 0..indent_count {
            print!(" ");
        }
        println!("{}", Color::Black.on(Color::White).paint(space_string));
    }
}

type MyResult<T> = Result<T, Error>;

#[derive(Debug)]
#[allow(unused)]
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
