extern crate lib;

use std::f64::consts::PI;
use lib::complex_num::ComplexValueF64;
use lib::complex_num::complex_integral::{IntegralCalculator, ComplexFunction};
use lib::utf8::get_utf8_size;
use std::fs::File;
use std::env::args;

fn main() -> Result<(), ()> {
    unsafe {
        if libc::isatty(libc::STDIN_FILENO) == 0 {
            println!("Stdin is not a terminal");
            return Ok(());
        }
    }
    loop {
        let c = scan_keyboard();
        println!("Received key char: {}", c);
    }
}

fn str_to_c_char_ptr(s: &str) -> *const libc::c_char {
    return unsafe { s.as_ptr() as *const libc::c_char };
}

fn scan_keyboard() -> char {
    let input;
    unsafe {
        let init_termios_struct = || {
            return libc::termios {
                c_iflag: 0,
                c_oflag: 0,
                c_cflag: 0,
                c_lflag: 0,
                c_line: 0,
                c_cc: [0; 32],
                c_ispeed: 0,
                c_ospeed: 0,
            };
        };

        let mut new_settings = init_termios_struct();
        let mut stored_settings = init_termios_struct();
        if libc::tcgetattr(libc::STDIN_FILENO, &stored_settings as *const libc::termios as *mut libc::termios) != 0 {
            panic!("tcgetattr(...) failed");
        };
        new_settings = stored_settings;
        new_settings.c_lflag &= !(libc::ICANON | libc::ECHO) as libc::tcflag_t;
        new_settings.c_cc[libc::VMIN] = 1;
        if libc::tcsetattr(libc::STDIN_FILENO, libc::TCSANOW, &new_settings as *const libc::termios) != 0 {
            panic!("tcsetattr(...) failed");
        };
        input = std::char::from_u32(libc::getchar() as u32).unwrap();
        if libc::tcsetattr(libc::STDIN_FILENO, libc::TCSANOW, &stored_settings as *const libc::termios) != 0 {
            panic!("tcsetattr(...) failed");
        }
    }
    return input;
}