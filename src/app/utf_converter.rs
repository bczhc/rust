extern crate lib;

use std::fs::File;
use std::io::{BufReader, ErrorKind, Read, stdin, stdout, Write};
use lib::byteorder::{get_endianness, Endianness};
use lib::utf8;
use lib::utf8::{get_utf8_bytes_length, solve_utf8_bytes};

fn main() -> Result<(), String> {
    let mut input_stream: &dyn Read = &stdin();
    let mut output_stream: &dyn Write = &stdout();

    let self_endianness = get_endianness();

    let args: Vec<String> = std::env::args().collect();
    let call_cmd = &args[0];
    let args = &args[1..];
    let arg_count = args.len();

    let path = std::path::Path::new(&call_cmd);
    let file_name = path.file_name().unwrap().to_str().unwrap();

    let help_msg = get_help_msg(file_name);

    if arg_count == 0 {
        print_help_msg(&help_msg);
        return Ok(());
    }
    if arg_count == 1 {
        let argv1 = &args[0];
        if argv1.eq_ignore_ascii_case("-h") || argv1.eq_ignore_ascii_case("--help") {
            print_help_msg(&help_msg);
        } else if argv1.eq_ignore_ascii_case("--about") {
            println!("Written by bczhc (https://github.com/bczhc).\n...");
        } else {
            return Err(format!("Unknown option: {}", argv1));
        }
        return Ok(());
    }

    let mut buffer_size: i64 = 8192;
    let mut input_file_path: Option<&String> = None;
    let mut output_file_path: Option<&String> = None;

    if arg_count > 2 {
        for i in (0..(arg_count - 3)).step_by(2) {
            let option = &args[i];
            let option_bytes = option.as_bytes();
            if option_bytes[0] != b'-' && !option[0..2].eq("--") {
                return Err(format!("Unknown option: {}", option));
            }
            if i > arg_count {
                return Err(String::from("Invalid arguments count."));
            }
            let arg = &args[i + 1];

            if option.eq("-b") || option.eq("--buffer-size") {
                let parsed = arg.parse::<i64>();
                if let Ok(v) = parsed {
                    buffer_size = v;
                } else if let Err(e) = parsed {
                    return Err(e.to_string());
                }
            } else if option.eq("-i") || option.eq("--input") {
                input_file_path = Some(arg);
            } else if option.eq("-o") || option.eq("--output") {
                output_file_path = Some(arg);
            } else {
                return Err(format!("Unknown option: {}", option));
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
    let converter: fn(u32, &mut [u8]) -> u32;

    match to.to_ascii_lowercase().as_str() {
        "utf8" => converter = unicode_to_utf8,
        "utf16be" | "utf-16be" => {
            converter = if self_endianness == Endianness::BigEndian {
                unicode_to_utf16_machine_endianness
            } else {
                unicode_to_utf16_reversed_machine_endianness
            }
        }
        "utf16le" | "utf-16le" => {
            converter = if self_endianness == Endianness::LittleEndian {
                unicode_to_utf16_machine_endianness
            } else {
                unicode_to_utf16_reversed_machine_endianness
            }
        }
        "utf32be" | "utf-32be" => {
            converter = if self_endianness == Endianness::BigEndian {
                unicode_to_utf32_machine_endianness
            } else {
                unicode_to_utf32_reversed_machine_endianness
            }
        }
        "utf32le" | "utf-32le" => {
            converter = if self_endianness == Endianness::LittleEndian {
                unicode_to_utf32_machine_endianness
            } else {
                unicode_to_utf32_reversed_machine_endianness
            }
        }
        _ => {
            return Err(format!("Unknown <to> encode: {}", to));
        }
    }

    match from.to_ascii_lowercase().as_str() {
        "utf8" => process_utf8_input(&converter),

        "utf16be" | "utf-16be" => {
            if self_endianness == Endianness::BigEndian {
                process_utf16_input_machine_endianness(&converter);
            } else {
                process_utf16_input_reversed_machine_endianness(&converter);
            }
        }
        "utf16le" | "utf-16le" => {
            if self_endianness == Endianness::LittleEndian {
                process_utf16_input_machine_endianness(&converter);
            } else {
                process_utf16_input_reversed_machine_endianness(&converter);
            }
        }
        "utf32be" | "utf-32be" => {
            if self_endianness == Endianness::BigEndian {
                process_utf32_input_machine_endianness(&converter);
            } else {
                process_utf32_input_reversed_machine_endianness(&converter);
            }
        }
        "utf32le" | "utf-32le" => {
            if self_endianness == Endianness::LittleEndian {
                process_utf32_input_machine_endianness(&converter);
            } else {
                process_utf32_input_reversed_machine_endianness(&converter);
            }
        }
        _ => {
            return Err(format!("Unknown <from> encode: {}", from));
        }
    }

    return Ok(());
}

fn get_help_msg(file_name: &str) -> String {
    return format!(
        "A from-stdin-to-stdout UTF converter.
Usage: {0} [option] <from> <to>
       {0} (--help | -h)
       {0} --about

Options:
  -b <size>, --buffer-size <size>  IO buffer size, in bytes.
  -i <path>, --input <path>  Input file instead of stdin.
  -o <path>, --output <path>  Output file instead of stdout.

Positional arguments:
from, to: [utf[-]](8|16be|16le|32be|32le)",
        file_name
    );
}

#[inline]
fn print_help_msg(help_msg: &str) {
    println!("{}", help_msg);
}

/// returns: bytes size
#[inline]
fn unicode_to_utf8(codepoint: u32, dest: &mut [u8]) -> u32 {
    return utf8::unicode_to_utf8(codepoint, dest);
}

/// returns: bytes size
#[inline]
fn unicode_to_utf16_machine_endianness(codepoint: u32, dest: &mut [u8]) -> u32 {
    return if codepoint <= 0xffff {
        let codepoint = (codepoint | 0b1111_1111_1111_1111__u32) as u16;
        unsafe {
            let mut p = &codepoint as *const u16 as *const u8;
            dest[0] = *p;
            p = ((p as usize) + 1) as *const u8;
            dest[1] = *p;
        }
        2
    } else {
        // codepoint >= 0x10000
        let surrogate_pair = utf8::unicode_to_surrogate_pair(codepoint);
        unsafe {
            let mut p = &surrogate_pair.lead as *const u16 as *const u8;
            dest[0] = *p;
            p = ((p as usize) + 1) as *const u8;
            dest[1] = *p;
            let mut p = &surrogate_pair.trail as *const u16 as *const u8;
            dest[2] = *p;
            p = ((p as usize) + 1) as *const u8;
            dest[3] = *p;
        }
        4
    };
}

/// returns: bytes size
#[inline]
fn unicode_to_utf16_reversed_machine_endianness(codepoint: u32, dest: &mut [u8]) -> u32 {
    let mut t: [u8; 4] = [0, 0, 0, 0];
    let r = unicode_to_utf16_machine_endianness(codepoint, &mut t);
    if r == 2 {
        dest[0] = t[2];
        dest[1] = t[1];
    } else {
        // r == 4
        dest[0] = t[3];
        dest[1] = t[2];
        dest[2] = t[1];
        dest[3] = t[0];
    }
    return r;
}

#[inline]
fn unicode_to_utf32_machine_endianness(codepoint: u32, dest: &mut [u8]) -> u32 {
    unsafe {
        let p = &codepoint as *const u32 as *const u8;
        dest[0] = *(((p as usize) + 0) as *const u8);
        dest[1] = *(((p as usize) + 4) as *const u8);
        dest[2] = *(((p as usize) + 8) as *const u8);
        dest[3] = *(((p as usize) + 12) as *const u8);
    }
    return 4;
}

#[inline]
fn unicode_to_utf32_reversed_machine_endianness(codepoint: u32, dest: &mut [u8]) -> u32 {
    let mut t: [u8; 4] = [0, 0, 0, 0];
    let _ = unicode_to_utf32_machine_endianness(codepoint, &mut t);
    dest[0] = t[0];
    dest[1] = t[1];
    dest[2] = t[2];
    dest[3] = t[3];
    return 4;
}

fn process_utf8_input(unicode_converter: &fn(u32, &mut [u8]) -> u32) {
    let mut br = BufReader::new(stdin());
    let mut read: [u8; 4] = [0, 0, 0, 0];
    let mut out_buf: [u8; 4] = [0, 0, 0, 0];
    loop {
        let r = br.read_exact(&mut read[0..1]);
        if let Err(e) = r {
            if let ErrorKind::UnexpectedEof = e.kind() {
                break;
            } else {
                panic!("{}", e);
            }
        }
        let utf8_bytes_length = get_utf8_bytes_length(read[0]);
        if utf8_bytes_length > 1 {
            // read the left bytes the current character needed
            br.read_exact(&mut read[1_usize..(utf8_bytes_length as usize)])
                .unwrap();
        }
        let solved = solve_utf8_bytes(&read);
        let size = unicode_converter(solved.codepoint, &mut out_buf);
        stdout().write(&out_buf[..size as usize]).unwrap();
    }
}

fn process_utf16_input_machine_endianness(unicode_converter: &fn(u32, &mut [u8]) -> u32) {
    let mut buf: [u8; 4] = [0, 0, 0, 0];
    let mut br = BufReader::new(stdin());
    loop {
        let r = br.read_exact(&mut buf[0..2]);
        if let Err(e) = r {
            if let ErrorKind::UnexpectedEof = e.kind() {
                break;
            } else {
                panic!("{}", e);
            }
        }
        unsafe {
            let p = &buf as *const u8 as *const u16;
            if *p >= 0xd800 && *p <= 0xdb7f {
                // use surrogate pair, need to read rwo more bytes
                br.read_exact(&mut buf[2..]).unwrap();
                let lead = *p;
                let trail = *(((p as usize) + 2) as *const u16);
                let unicode = utf8::surrogate_pair_to_unicode(lead, trail);
                let _ = unicode_converter(unicode, &mut buf);
                stdout().write(&buf[..]).unwrap();
            } else {
                let unicode = *p as u32;
                let _ = unicode_converter(unicode, &mut buf);
                stdout().write(&buf[..2]);
            }
        }
    }
}

fn process_utf16_input_reversed_machine_endianness(unicode_converter: &fn(u32, &mut [u8]) -> u32) {
    let mut buf: [u8; 4] = [0, 0, 0, 0];
    let mut br = BufReader::new(stdin());
    loop {
        let r = br.read_exact(&mut buf[0..2]);
        if let Err(e) = r {
            if let ErrorKind::UnexpectedEof = e.kind() {
                break;
            } else {
                panic!("{}", e);
            }
        }
        unsafe {
            let t = buf[0];
            buf[0] = buf[1];
            buf[1] = t;
            let p = &buf as *const u8 as *const u16;
            if *p >= 0xd800 && *p <= 0xdb7f {
                // use surrogate pair, need to read rwo more bytes
                br.read_exact(&mut buf[2..]).unwrap();
                let t = buf[2];
                buf[2] = buf[3];
                buf[3] = t;
                let lead = *p;
                let trail = *(((p as usize) + 2) as *const u16);
                let unicode = utf8::surrogate_pair_to_unicode(lead, trail);
                let _ = unicode_converter(unicode, &mut buf);
                stdout().write(&buf[..]).unwrap();
            } else {
                let unicode = *p as u32;
                let _ = unicode_converter(unicode, &mut buf);
                stdout().write(&buf[..2]);
            }
        }
    }
}

fn process_utf32_input_machine_endianness(unicode_converter: &fn(u32, &mut [u8]) -> u32) {
    let mut br = BufReader::new(stdin());
    let mut buf: [u8; 4] = [0, 0, 0, 0];
    loop {
        let r = br.read_exact(&mut buf);
        if let Err(e) = r {
            if let ErrorKind::UnexpectedEof = e.kind() {
                break;
            } else {
                panic!("{}", e);
            }
        }
        let unicode;
        unsafe { unicode = *(&buf as *const u8 as *const u32) }
        let _size = unicode_converter(unicode, &mut buf);
        stdout().write(&buf).unwrap();
    }
}

fn process_utf32_input_reversed_machine_endianness(unicode_converter: &fn(u32, &mut [u8]) -> u32) {
    let mut br = BufReader::new(stdin());
    let mut buf: [u8; 4] = [0, 0, 0, 0];
    let mut t: [u8; 4] = [0, 0, 0, 0];
    loop {
        let r = br.read_exact(&mut t);
        if let Err(e) = r {
            if let ErrorKind::UnexpectedEof = e.kind() {
                break;
            } else {
                panic!("{}", e);
            }
        }
        let unicode;
        unsafe {
            buf[0] = t[3];
            buf[1] = t[2];
            buf[2] = t[1];
            buf[3] = t[0];
            unicode = *(&buf as *const u8 as *const u32)
        }
        let _size = unicode_converter(unicode, &mut buf);
        stdout().write(&buf).unwrap();
    }
}