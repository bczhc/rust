extern crate size_format;

use size_format::SizeFormatterBinary;
use std::env::args;
use std::io::{stdin, Read};
use std::path::Path;

fn main() -> Result<(), String> {
    let mut args: Vec<String> = args().collect();
    args.remove(0);

    if args.len() >= 2 {
        return show_msg(MsgType::InvalidArgumentCount(args.len()));
    }

    struct Arguments {
        human_readable: bool,
    }
    let mut arguments = Arguments {
        human_readable: false,
    };

    if args.len() == 1 {
        let option = &args[0];
        match option.as_str() {
            "-h" | "--human-readable" => {
                arguments.human_readable = true;
            }
            "--help" => {
                return show_msg(MsgType::Help);
            }
            _ => {
                return show_msg(MsgType::UnknownOption(option));
            }
        }
    }

    let mut stdin = stdin();
    let mut buf = [0_u8; 4096];
    let mut size_sum = 0_usize;
    loop {
        let r = stdin.read(&mut buf).unwrap();
        size_sum += r;
        if r == 0 {
            break;
        }
    }

    if arguments.human_readable {
        let size = SizeFormatterBinary::new(size_sum as u64).to_string();
        println!("{}B", size);
    } else {
        println!("{}", size_sum);
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
            let mut args = args();
            let file_path = args.next().unwrap();
            let file_name = Path::new(&file_path).file_name().unwrap().to_str().unwrap();

            println!(
                "Print the size of all data read from stdin.
Usage: {0} [option]
Options:
  -h, --human-readable  Print human-readable size format.
  --help  Show this help.",
                file_name
            );
            Ok(())
        }
        MsgType::InvalidArgumentCount(c) => Err(format!("Invalid argument count: {}", c)),
        MsgType::UnknownOption(o) => Err(format!("Unknown option: {}", o)),
    };
}
