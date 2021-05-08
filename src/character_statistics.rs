extern crate lib;

use std::collections::HashMap;
use std::io::{stdin, BufReader, Read};
use lib::utf8::{solve_utf8_bytes, get_utf8_bytes_length};
use std::cmp::Ordering;
use utils::{MsgPrinter, get_file_name, MsgType, get_args_without_self_path};
use std::mem::size_of;
use lib::char::han_char_range;

mod utils;

fn main() -> Result<(), String> {
    let mut m = Main::new();
    return m.run();
}

struct Argument {
    han_mode: bool,
}

struct Main {
    argument: Argument,
    msg_printer: MsgPrinter,
}

impl Main {
    #[inline]
    fn new() -> Self {
        let help_msg = format!("Count characters read from stdin.
Usage: {} [option]
Options:
  --han  Count Han characters.
  -h, --help  Show this help.", get_file_name());

        return Self {
            msg_printer: MsgPrinter::new(help_msg),
            argument: Argument {
                han_mode: false
            },
        };
    }

    fn run(&mut self) -> Result<(), String> {
        let args = get_args_without_self_path();

        if args.len() >= 2 {
            return self.msg_printer.show_msg(MsgType::InvalidArgumentCount(args.len()));
        }

        if args.len() == 1 {
            let option = &args[0];
            match option.as_str() {
                "-h" | "--help" => {
                    return self.msg_printer.show_msg(MsgType::Help);
                }
                "--han" => {
                    self.argument.han_mode = true;
                }
                _ => {
                    return self.msg_printer.show_msg(MsgType::UnknownOption(option));
                }
            }
        }

        self.do_statistics();
        return Ok(());
    }

    fn do_statistics(&self) {
        let mut reader = BufReader::new(stdin());
        let mut buf: [u8; 4] = [0, 0, 0, 0];

        let mut map = HashMap::new();

        loop {
            let r = reader.read_exact(&mut buf[0..1]);
            if let Err(_) = r {
                break;
            }

            let size = get_utf8_bytes_length(buf[0]) as usize;
            reader.read_exact(&mut buf[1..size]).unwrap();

            let solved = solve_utf8_bytes(&buf);
            let c = std::char::from_u32(solved.codepoint).unwrap();

            if map.contains_key(&c) {
                *map.get_mut(&c).unwrap() += 1;
            } else {
                map.insert(c, 1);
            }
        }

        let mut chars_vec: Vec<_> = map.iter().collect();
        chars_vec.sort_by(|&a, &b| {
            return if a.1 > b.1 {
                Ordering::Greater
            } else if a.1 == b.1 {
                Ordering::Equal
            } else {
                Ordering::Less
            };
        });

        if self.argument.han_mode {
            let mut count = 0;
            let mut t = 0;
            let mut i = (chars_vec.len() - 1) as isize;
            while i >= 0 {
                let (c, _) = chars_vec[i as usize];
                if han_char_range(*c as u32) {
                    print!("{}", c);
                    t += 1;
                    if t % 25 == 0 {
                        println!();
                        t = 0;
                    }
                    count += 1;
                }
                i -= 1;
            }
            if count != 0 {
                println!();
            }
            println!("Total: {}", count);
        } else {
            println!("{:?}", chars_vec);
            println!("Total: {}", chars_vec.len());
        }
    }
}
