extern crate bczhc_lib;

use bczhc_lib::io::ReadLines;
use std::env::args;
use std::io::{stdin, Read};
use std::path::Path;

fn main() -> Result<(), String> {
    let mut args: Vec<String> = args().collect();
    args.remove(0);

    if args.len() >= 2 {
        return show_msg(MsgType::InvalidArgumentCount(args.len()));
    }

    enum ReverseMode {
        Line,
        All,
    }

    struct Arguments {
        reverse_mode: ReverseMode,
    }
    let mut arguments = Arguments {
        reverse_mode: ReverseMode::All,
    };

    if args.len() == 1 {
        let option = &args[0];
        match option.as_str() {
            "-l" | "--line" => {
                arguments.reverse_mode = ReverseMode::Line;
            }
            "-a" | "--all" => {
                arguments.reverse_mode = ReverseMode::All;
            }
            "-h" | "--help" => {
                return show_msg(MsgType::Help);
            }
            _ => {
                return show_msg(MsgType::UnknownOption(option));
            }
        }
    }

    match arguments.reverse_mode {
        ReverseMode::Line => {
            let mut stdin = stdin();
            let lines = ReadLines::lines(&mut stdin);
            for line in lines {
                println!("{}", reverse_string(&line));
            }
        }
        ReverseMode::All => {
            let mut stdin = stdin();
            let mut buf = String::new();
            stdin.read_to_string(&mut buf).unwrap();
            let reversed = reverse_string(&buf);
            print!("{}", reversed);
        }
    }

    Ok(())
}

enum MsgType<'a> {
    Help,
    InvalidArgumentCount(usize),
    UnknownOption(&'a String),
}

fn show_msg(msg_type: MsgType) -> Result<(), String> {
    return match msg_type {
        MsgType::Help => {
            let file_path = args().next().unwrap();
            let file_name = Path::new(&file_path).file_name().unwrap().to_str().unwrap();
            println!(
                "Reverse string read from stdin.
Usage: {} [option]
Options:
  -l, --line  Reverse string by each line.
  -a, --all  Reverse all string read from stdin; this is the default mode.
  -h, --help  Show this help.",
                file_name
            );
            Ok(())
        }
        MsgType::InvalidArgumentCount(c) => Err(format!("Invalid argument count: {}", c)),
        MsgType::UnknownOption(o) => Err(format!("Unknown option: {}", o)),
    };
}

fn reverse_string(s: &str) -> String {
    if s.is_empty() {
        return String::from("");
    }
    let chars: Vec<char> = s.chars().collect();
    let mut i = chars.len() - 1;
    let mut r = String::new();
    loop {
        if i == 0 {
            r.push(chars[0]);
            break;
        }
        r.push(chars[i]);
        i -= 1;
    }
    r
}
