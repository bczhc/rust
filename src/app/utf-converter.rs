use std::env::args;
use std::error::Error;
use std::fs::File;
use std::io::{Read, stdin, stdout, Write};

fn main() -> Result<(), ()> {
    let mut input_stream: &dyn Read = &stdin();
    let mut output_stream: &dyn Write = &stdout();

    let args: Vec<String> = std::env::args().collect();
    let call_cmd = &args[0];
    let args = &args[1..];
    let arg_count = args.len();

    let path = std::path::Path::new(&call_cmd);
    let file_name = path.file_name().unwrap().to_str().unwrap();

    let help_msg = get_help_msg(file_name);

    if arg_count == 0 { print_help_msg(&help_msg); }
    if arg_count == 1 {
        let argv1 = &args[0];
        if argv1.eq_ignore_ascii_case("-h") || argv1.eq_ignore_ascii_case("--help") {
            print_help_msg(&help_msg);
            return Err(());
        } else if argv1.eq_ignore_ascii_case("--info") {
            println!("Written by bczhc (https://github.com/bczhc).\n...");
        } else {
            println!("Unknown option: {}", argv1);
        }
    }

    let mut buffer_size: i64 = 8192;
    let mut input_file_path: Option<&String> = None;
    let mut output_file_path: Option<&String> = None;

    if arg_count > 2 {
        for i in (0..(arg_count - 3)).step_by(2) {
            let option = &args[i];
            let option_bytes = option.as_bytes();
            if option_bytes[0] != b'-' && !option[0..2].eq("--") {
                eprintln!("Unknown option: {}", option);
                return Err(());
            }
            if i > arg_count {
                eprintln!("Invalid arguments count.");
                return Err(());
            }
            let arg = &args[i + 1];

            if option.eq("-b") || option.eq("--buffer-size") {
                let parsed = arg.parse::<i64>();
                if let Ok(v) = parsed {
                    buffer_size = v;
                } else if let Err(e) = parsed {
                    eprintln!("{}", e.to_string());
                    return Err(());
                }
            } else if option.eq("-i") || option.eq("--input") {
                input_file_path = Some(arg);
            } else if option.eq("-o") || option.eq("--output") {
                output_file_path = Some(arg);
            } else {
                eprintln!("Unknown option: {}", option);
                return Err(());
            }
        }
    }

    // TODO: SIGINT handling

    if let Some(path) = input_file_path {
        input_stream = &File::open(path).unwrap();
    }
    if let Some(path) = output_file_path {
        output_stream = &File::open(path).unwrap();
    }

    let from = &args[arg_count - 2];
    let to = &args[arg_count - 1];

    
    return Ok(());
}

fn get_help_msg(file_name: &str) -> String {
    return format!("A from-stdin-to-stdout UTF converter.
Usage: {0} [option] <from> <to>
       {0} (--help | -h)
       {0} --info

Options:
  -b <size>, --buffer-size <size>  IO buffer size, in bytes.
  -i <path>, --input <path>  Input file instead of stdin.
  -o <path>, --output <path>  Output file instead of stdout.

Positional arguments:
from, to: [utf[-]](8|16be|16le|32be|32le)", file_name);
}

#[inline]
fn print_help_msg(help_msg: &str) {
    println!("{}", help_msg);
}