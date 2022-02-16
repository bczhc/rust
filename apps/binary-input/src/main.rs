#[macro_use]
extern crate quick_error;

mod errors;

use bczhc_lib::utf8::utf8_bytes_length;
use errors::*;
use std::fs::File;
use std::io::{stdin, stdout, Cursor, Read, Stdin, Stdout, Write};
use std::os::unix::io::AsRawFd;
use termios::Termios;

const STDIN_FD: i32 = 0;

fn main() -> Result<()> {
    if !check_tty() {
        println!("Not a terminal");
        return Ok(());
    }

    let mut stdout = stdout();

    let mut lead_byte_bits = Vec::with_capacity(8);
    let mut tail_byte_bits = Vec::new();
    let mut tail_input = false;
    let mut tail_bytes_count = 0_u32;

    loop {
        let c = scan_key()?;
        let num = c - b'0';
        if num != 0 && num != 1 {
            continue;
        }

        if !tail_input {
            lead_byte_bits.push(num);

            if lead_byte_bits.len() == 8 {
                let byte = bits_to_byte(&lead_byte_bits[..]);
                tail_bytes_count = utf8_bytes_length(byte) - 1;

                if tail_bytes_count == 0 {
                    // it's an one-byte utf-8 encoded character
                    stdout.write_all(&[byte])?;
                    stdout.flush()?;
                    lead_byte_bits.clear();
                    continue;
                }

                tail_input = true;
            }
        } else {
            tail_byte_bits.push(num);
            if tail_byte_bits.len() == (tail_bytes_count * u8::BITS) as usize {
                let mut cursor = Cursor::new(&tail_byte_bits);
                let mut buf = [0_u8; 8];
                let mut tail_bytes = Vec::new();
                for _ in 0..tail_bytes_count {
                    cursor.read_exact(&mut buf)?;
                    let byte = bits_to_byte(&buf);
                    tail_bytes.push(byte);
                }
                stdout.write_all(&[bits_to_byte(&lead_byte_bits[..])])?;
                stdout.write_all(&tail_bytes[..])?;
                stdout.flush()?;

                lead_byte_bits.clear();
                tail_byte_bits.clear();

                tail_input = false;
            }
        }
    }
}

fn bits_to_byte(bits: &[u8]) -> u8 {
    let mut byte = 0_u8;
    for i in 0..8 {
        let bit = if bits[i] == 0 { 0_u8 } else { 1_u8 };
        byte |= bit << (7 - i);
    }
    byte
}

fn scan_key() -> Result<u8> {
    let stdin_fd = STDIN_FD;

    let saved_settings = Termios::from_fd(stdin_fd)?;
    let mut new_settings = saved_settings.clone();

    new_settings.c_lflag &= !(termios::ICANON | termios::ECHO);
    new_settings.c_cc[termios::VMIN] == 1;
    termios::tcsetattr(stdin_fd, termios::TCSANOW, &new_settings)?;

    let c = unsafe { libc::getchar() };
    termios::tcsetattr(stdin_fd, termios::TCSANOW, &saved_settings)?;
    Ok(c as u8)
}

fn check_tty() -> bool {
    unsafe { libc::isatty(STDIN_FD) == 1 }
}
