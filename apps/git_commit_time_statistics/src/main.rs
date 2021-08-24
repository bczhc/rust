use bczhc_lib::io::ReadLine;
use chrono::format::Fixed::{TimezoneName, TimezoneOffset};
use chrono::format::Numeric::Timestamp;
use chrono::{Date, DateTime, FixedOffset, Local, NaiveDate, TimeZone, Timelike, Utc, Offset};
use clap::{App, Arg};
use std::env::current_dir;
use std::fs::File;
use std::io::{Cursor, Read, Stdin};
use std::num::ParseIntError;
use std::process::{Command, Stdio};
use std::time::SystemTime;

fn main() -> MyResult<()> {
    let default_timezone_num = Local.timestamp(0, 0).offset().fix().local_minus_utc() / 3600;
    let default_timezone_num_string = default_timezone_num.to_string();

    let matches = App::new("git-commit-time")
        .author("bczhc <bczhc0@126.com>")
        .about("List the hours of the git commits author time")
        .arg(
            Arg::with_name("repo-path")
                .required(false)
                .help("Path of a git repository"),
        )
        .arg(
            Arg::with_name("timezone")
                .default_value(default_timezone_num_string.as_str())
                .short("t")
                .long("timezone")
                .help("Used time zone (UTC+<num>), format e.g.: -6, 0, or 8. The default is the local time zone"),
        )
        .get_matches();

    let repository_dir = if matches.is_present("repo-path") {
        String::from(matches.value_of("repo-path").unwrap())
    } else {
        let current_dir = current_dir()?;
        String::from(current_dir.to_str().unwrap())
    };

    let timezone_num: i8 = matches.value_of("timezone").unwrap().parse()?;
    if timezone_num < -12 || timezone_num > 12 {
        return Err(Error::InputRangeError);
    }
    let timezone = FixedOffset::east(timezone_num as i32 * 3600);

    let mut process = Command::new("git")
        .args(&["log", "--pretty=format:%at"])
        .current_dir(repository_dir)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let mut stdout = process.stdout.unwrap();
    let mut stderr = process.stderr.unwrap();

    let mut stderr_string = String::new();
    stderr.read_to_string(&mut stderr_string);
    if !stderr_string.is_empty() {
        return Err(Error::GitError(stderr_string));
    }

    let mut frequency_arr = [0_u8; 24];
    let mut process_timestamp = |timestamp: i64| {
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
                let timestamp = line.parse().unwrap();
                process_timestamp(timestamp);
            }
        }
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
    InputParseError(ParseError),
    InputRangeError,
    GitError(String),
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::IOError(e)
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(e: ParseIntError) -> Self {
        Error::InputParseError(ParseError::ParseIntError(e))
    }
}
