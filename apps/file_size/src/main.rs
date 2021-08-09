use bczhc_lib::utils::{get_args_without_self_path, get_file_name, MsgPrinter, MsgType};
use size_format::SizeFormatterBinary;

fn main() -> Result<(), String> {
    let msg_printer = MsgPrinter::new(format!(
        "Get file size.
Usage: {} [option] [--] <file-path>

Options:
-h, --human-readable  Print size as human-readable format.
--help  Show this help.",
        get_file_name()
    ));

    let args = get_args_without_self_path();

    if args.len() == 0 {
        return msg_printer.show_msg(MsgType::Help);
    }

    if args.len() >= 4 {
        return msg_printer.show_msg(MsgType::InvalidArgumentCount(args.len()));
    }

    let mut arguments = Arguments {
        human_readable: false,
    };

    if args.len() == 1 {
        let argv = &args[0];
        if argv.starts_with("-") {
            // interpreted as an option
            match_option(&msg_printer, &mut arguments, argv)?;
        } else {
            // interpreted as file path positional argument
            return print_file_size(argv, arguments.human_readable);
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
        };
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

fn match_option(
    msg_printer: &MsgPrinter,
    arguments: &mut Arguments,
    option: &String,
) -> Result<(), String> {
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
        if portable_fseek(fp, 0, libc::SEEK_END) != 0 {
            return Err(String::from("Failed to seek file"));
        }
        let size = portable_ftell(fp);
        if size < 0 {
            return Err(format!("ftell error, errno: {}", errno::errno().0));
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
    return s.as_bytes().as_ptr() as *const libc::c_char;
}

fn string_to_c_str(s: &String) -> *const libc::c_char {
    return str_to_c_str(s.as_str());
}

#[cfg(target_family = "windows")]
fn portable_ftell(fp: *mut libc::FILE) -> libc::off_t {
    return unsafe { libc::ftell(fp) };
}

#[cfg(target_family = "windows")]
fn portable_fseek(fp: *mut libc::FILE, offset: libc::c_long, whence: libc::c_int) -> libc::c_int {
    return unsafe { return libc::fseek(fp, offset, whence) };
}

#[cfg(target_family = "unix")]
fn portable_ftell(fp: *mut libc::FILE) -> libc::off_t {
    return unsafe { libc::ftello(fp) };
}

#[cfg(target_family = "unix")]
fn portable_fseek(fp: *mut libc::FILE, offset: libc::c_long, whence: libc::c_int) -> libc::c_int {
    return unsafe { libc::fseeko(fp, offset, whence) };
}

struct Arguments {
    human_readable: bool,
}
