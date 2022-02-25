use bczhc_lib::io::ReadLines;
use rand::{thread_rng, Rng};
use std::env::args;
use std::io::{stdin, Read};
use std::path::Path;

fn main() -> Result<(), String> {
    let mut args: Vec<String> = args().collect();
    args.remove(0);

    if args.len() >= 2 {
        return show_msg(MsgType::InvalidArgumentCount(args.len()));
    }

    let mut shuffle_mode = ShuffleMode::All;

    if args.len() == 1 {
        let option = &args[0];
        match option.to_ascii_lowercase().as_str() {
            "-h" | "--help" => {
                return show_msg(MsgType::Help);
            }
            "-l" | "--line" => {
                shuffle_mode = ShuffleMode::Line;
            }
            "-a" | "--all" => {
                shuffle_mode = ShuffleMode::All;
            }
            _ => {
                return show_msg(MsgType::UnknownOption(option));
            }
        }
    }

    match shuffle_mode {
        ShuffleMode::Line => {
            let mut stdin = stdin();
            let lines = ReadLines::lines(&mut stdin);
            for line in lines {
                shuffle_string_and_print(&line);
            }
            // newline to flush the stdout buffer
            println!();
        }
        ShuffleMode::All => {
            let mut s = String::new();
            stdin().read_to_string(&mut s).unwrap();
            shuffle_string_and_print(&s);
        }
    }
    Ok(())
}

fn shuffle_string_and_print(s: &str) {
    let mut rng = thread_rng();

    let chars = s.chars();
    let mut chars: Vec<char> = chars.collect();
    while !chars.is_empty() {
        let c = chars.remove(rng.gen_range(0..chars.len()));
        print!("{}", c);
    }
}

enum MsgType<'a> {
    Help,
    InvalidArgumentCount(usize),
    UnknownOption(&'a String),
}

enum ShuffleMode {
    Line,
    All,
}

fn show_msg(msg_type: MsgType) -> Result<(), String> {
    return match msg_type {
        MsgType::Help => {
            let file_path = args().next().unwrap();
            let file_name = Path::new(&file_path).file_name().unwrap().to_str().unwrap();
            let help_msg = format!(
                "Shuffle string read from stdin.
Usage: {} [option]
Options:
  -l, --line  Shuffle string by each line.
  -a, --all  Shuffle all string read from stdin; it's the default mode.
  -h, --help  Show this help.",
                file_name
            );
            println!("{}", help_msg);
            Ok(())
        }
        MsgType::InvalidArgumentCount(count) => {
            return Err(format!("Invalid argument count: {}", count));
        }
        MsgType::UnknownOption(option) => {
            return Err(format!("Unknown option: {}", option));
        }
    };
}
