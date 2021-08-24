use bczhc_lib::io::ReadLine;
use chrono::format::Fixed::{TimezoneName, TimezoneOffset};
use chrono::format::Numeric::Timestamp;
use chrono::{Date, DateTime, FixedOffset, Local, NaiveDate, Offset, TimeZone, Timelike, Utc};
use clap::{App, Arg};
use std::env::current_dir;
use std::fs::File;
use std::io::{Cursor, Read, Stdin};
use std::num::ParseIntError;
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

    let repository_dir = if matches.is_present("repo-path") {
        String::from(matches.value_of("repo-path").unwrap())
    } else {
        let current_dir = current_dir()?;
        String::from(current_dir.to_str().unwrap())
    };

    let mut process = Command::new("git")
        .args(&["log", "--format=%ad", "--date=raw"])
        .current_dir(repository_dir)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let mut stdout = process.stdout.unwrap();

    let mut frequency_arr = [0_usize; 24];
    let mut process_date = |timestamp: i64, timezone_offset_secs: i32| {
        let timezone = FixedOffset::east(timezone_offset_secs);
        let time = Utc.timestamp(timestamp as i64, 0).with_timezone(&timezone);
        let hour = time.hour();
        frequency_arr[hour as usize] += 1;
    };

    loop {
        let read = stdout.read_line_without_line_terminator();
        match read {
            None => {
                break;
            }
            Some(line) => {
                let (timestamp_str, timezone_num_str) = line.split_at(line.find(' ').unwrap());
                let timezone_num: i32 = (&timezone_num_str[1..]).parse().unwrap();
                let timezone_offset_secs = (timezone_num / 100 * 3600) + (timezone_num & 100 * 60);
                process_date(timestamp_str.parse().unwrap(), timezone_offset_secs);
            }
        }
    }

    let mut stderr = process.stderr.unwrap();

    let mut stderr_string = String::new();
    stderr.read_to_string(&mut stderr_string);
    if !stderr_string.is_empty() {
        return Err(Error::GitError(stderr_string));
    }

    for i in 0..frequency_arr.len() {
        println!("{}: {}", i, frequency_arr[i]);
    }

    Ok(())
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
