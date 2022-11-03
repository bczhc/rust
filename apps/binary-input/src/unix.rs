use crate::errors::*;
use bczhc_lib::utf8::utf8_bytes_length;
use console::{Style, Term};

use std::io::{Cursor, Read, Write};

use termios::Termios;

const STDIN_FD: i32 = 0;

pub fn main() -> Result<()> {
    if !check_tty() {
        println!("Not a terminal");
        return Ok(());
    }

    let mut stdout = Term::stdout();

    let mut input_bits = Vec::with_capacity(8);
    let mut utf8_length = 0_u32;

    let bits_compose_style = Style::new().dim();
    let mut composed_chars_bytes = Vec::new();

    loop {
        let c = scan_key()?;
        let num = c - b'0';
        if num != 0 && num != 1 {
            continue;
        }
        input_bits.push(num);

        let bit_compose_string = format!("{}", bits_compose_style.apply_to(num.to_string()));
        stdout.write_all(bit_compose_string.as_bytes())?;

        if input_bits.len() == 8 {
            let lead_byte = bits_to_byte(&input_bits[..]);
            utf8_length = utf8_bytes_length(lead_byte);
        }
        if input_bits.len() == (utf8_length * u8::BITS) as usize {
            let mut cursor = Cursor::new(&input_bits);
            let mut buf = [0_u8; 8];
            let mut char_utf_bytes = vec![0_u8; utf8_length as usize];
            for _ in 0..utf8_length {
                cursor.read_exact(&mut buf)?;
                let byte = bits_to_byte(&buf);
                char_utf_bytes.push(byte);
                composed_chars_bytes.push(byte);
            }

            stdout.clear_line()?;
            stdout.write_all(&composed_chars_bytes[..])?;
            stdout.flush()?;
            input_bits.clear();
        }
    }
}

fn bits_to_byte(bits: &[u8]) -> u8 {
    let mut byte = 0_u8;
    for (i, bit) in bits.iter().enumerate().take(8) {
        let bit = u8::from(*bit != 0);
        byte |= bit << (7 - i);
    }
    byte
}

#[allow(unused)]
fn to_bits_string(bits: &[u8]) -> String {
    let mut string = String::with_capacity(bits.len());
    for b in bits {
        string.push(if *b == 0 { '0' } else { '1' });
    }
    string
}

fn scan_key() -> Result<u8> {
    let stdin_fd = STDIN_FD;

    let saved_settings = Termios::from_fd(stdin_fd)?;
    let mut new_settings = saved_settings;

    new_settings.c_lflag &= !(termios::ICANON | termios::ECHO);
    new_settings.c_cc[termios::VMIN] = 1;
    termios::tcsetattr(stdin_fd, termios::TCSANOW, &new_settings)?;

    let c = unsafe { libc::getchar() };
    termios::tcsetattr(stdin_fd, termios::TCSANOW, &saved_settings)?;
    Ok(c as u8)
}

fn check_tty() -> bool {
    unsafe { libc::isatty(STDIN_FD) == 1 }
}
