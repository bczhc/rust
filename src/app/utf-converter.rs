extern crate rust;

use std::env::args;
use std::error::Error;
use std::fs::File;
use std::io::{Read, stdin, stdout, Write};

use rust::lib::byteorder::*;
use rust::lib::utf8;
use rust::lib::utf8::solve_utf8_bytes;

fn main() -> Result<(), String> {
    let mut input_stream: &dyn Read = &stdin();
    let mut output_stream: &dyn Write = &stdout();

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
    let mut to_endianness: Endianness;

    if to.eq_ignore_ascii_case("utf8") || to.eq_ignore_ascii_case("utf-8") {} else if to.eq_ignore_ascii_case("utf16be") || to.eq_ignore_ascii_case("utf-16be") {}
    match to.as_str() {
        // "utf8" => converter = unicode_to_utf8,
        // "utf16be" | "utf-16be" => converter = unicode_to_utf16_big_endian,
        // "utf16le" | "utf-16le" => converter = unicode_to_utf16_little_endian,
        // "utf32be" | "utf-32be" => converter = unicode_to_utf32_big_endian,
        // "utf32le" | "utf-32le" => converter = unicode_to_utf32_little_endian,
        _ => {
            return Err(format!("Unknown <to> encode: {}", to));
        }
    }

    match from.as_str() {
        // "utf8" => process_utf8_input(&converter),
        // "utf16be" | "utf-16be" => process_utf16_input(&converter, Endianness::BigEndian),
        // "utf16le" | "utf-16le" => process_utf16_input(&converter, Endianness::LittleEndian),
        // "utf32be" | "utf-32be" => process_utf32_input(&converter, Endianness::BigEndian),
        // "utf32le" | "utf-32le" => process_utf32_input(&converter, Endianness::LittleEndian),
        _ => {
            return Err(format!("Unknown <from> encode: {}", from));
        }
    }

    return Ok(());
}

fn get_help_msg(file_name: &str) -> String {
    return format!("A from-stdin-to-stdout UTF converter.
Usage: {0} [option] <from> <to>
       {0} (--help | -h)
       {0} --about

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

/// returns: bytes size
#[inline]
fn unicode_to_utf8(codepoint: u32, dest: &mut [u8]) -> u32 {
    return utf8::unicode_to_utf8(codepoint, dest);
}

/// returns: bytes size
#[inline]
fn unicode_to_utf16_machine_byteorder(codepoint: u32, dest: &mut [u8]) -> u32 {
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
fn unicode_to_utf16_reversed_machine_byteorder(codepoint: u32, dest: &mut [u8]) -> u32 {
    let mut t: [u8; 4] = [0, 0, 0, 0];
    let r = unicode_to_utf16_machine_byteorder(codepoint, &mut t);
    dest[0] = t[3];
    dest[1] = t[2];
    dest[2] = t[1];
    dest[3] = t[0];
    return r;
}

#[inline]
fn unicode_to_utf32_machine_byteorder(codepoint: u32, dest: &mut [u8]) {}

#[inline]
fn unicode_to_utf32_reversed_machine_byteorder(codepoint: u32, dest: &mut [u8]) {}

fn process_utf8_input(unicode_converter: &fn(u32, &mut [u8])) {}

fn process_utf16_input(unicode_converter: &fn(u32, &mut [u8]), to_endianness: Endianness) {}

fn process_utf32_input(unicode_converter: &fn(u32, &mut [u8]), to_endianness: Endianness) {}