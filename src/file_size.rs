extern crate size_format;

use std::env::args;
use std::path::Path;
use utils::{MsgPrinter, get_file_name, get_args_without_self_path, MsgType};
use size_format::SizeFormatterBinary;

mod utils;

fn main() -> Result<(), String> {
    let msg_printer = MsgPrinter::new(format!("Get file size.
Usage: {} [option] [--] <file-path>

Options:
-h, --human-readable  Print size as human-readable format.
--help  Show this help.", get_file_name()));

    let args = get_args_without_self_path();

    if args.len() == 0 {
        msg_printer.show_msg(MsgType::Help);
        return Ok(());
    }

    if args.len() >= 4 {
        msg_printer.show_msg(MsgType::InvalidArgumentCount(args.len()));
    }

    let mut arguments = Arguments {
        human_readable: false,
        file_path: None,
    };

    if args.len() == 1 {
        let argv = &args[0];
        if argv.starts_with("-") {
            // interpreted as an option
            match_option(&msg_printer, &mut arguments, argv)?;
        } else {
            // interpreted as file path positional argument
            return print_file_size(argv, arguments.human_readable)
        }
    }

    if args.len() == 2 {
        // (option) <file-path>
        // -- <file-path>
        return if args[0] == "--" {
            let file_path = &args[1];
            print_file_size(file_path, arguments.human_readable)
        } else {
            let option = &args[0];
            match_option(&msg_printer, &mut arguments, option)?;
            print_file_size(&args[1], arguments.human_readable)
        }
    }

    if args.len() == 3 {
        // (option) -- <file-path>
        if &args[1] != "--" {
            return Err(String::from(format!("Unknown parameter: {}", &args[1])));
        }
        let option = &args[0];
        let file_path = &args[2];

        match_option(&msg_printer, &mut arguments, option)?;
        return print_file_size(file_path, arguments.human_readable);
    }

    return Ok(());
}

fn match_option(msg_printer: &MsgPrinter, arguments: &mut Arguments, option: &String) -> Result<(), String> {
    match option.as_str() {
        "-h" | "--human-readable" => {
            arguments.human_readable = true;
        }
        "--help" => {
            return msg_printer.show_msg(MsgType::Help);
        }
        _ => {
            return msg_printer.show_msg(MsgType::UnknownOption(option));
        }
    }
    return Ok(());
}

fn print_file_size(file_path: &String, human_readable: bool) -> Result<(), String> {
    unsafe {
        let fp = libc::fopen(string_to_c_str(file_path), str_to_c_str("rb"));
        if fp.is_null() {
            return Err(String::from("Failed to open file"));
        }
        if libc::fseeko(fp, 0, libc::SEEK_END) != 0 {
            return Err(String::from("Failed to seek file"));
        }
        let size = libc::ftello(fp);
        if size < 0 {
            return Err(format!("ftello(...) error, errno: {}", errno::errno().0));
        }
        let size = size as u64;
        if human_readable {
            let size = SizeFormatterBinary::new(size).to_string();
            println!("{}B", size);
        } else {
            println!("{}", size);
        }
        return Ok(());
    }
}

fn str_to_c_str(s: &str) -> *const libc::c_char {
    unsafe {
        return s.as_bytes().as_ptr() as *const libc::c_char;
    }
}

fn string_to_c_str(s: &String) -> *const i8 {
    return str_to_c_str(s.as_str());
}

struct Arguments {
    human_readable: bool,
    file_path: Option<String>,
}