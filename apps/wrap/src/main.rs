use std::io::{stdin, stdout, Write};
use bczhc_lib::char::CharsTrait;
use clap::{App, Arg};
use bczhc_lib::utf8::encode_utf8;

fn main() {
    let matches = App::new("wrap")
        .about("Wrap lines")
        .author("bczhc <bczhc0@126.com>")
        .arg(Arg::with_name("columns").default_value("50").required(false)).get_matches();

    let columns = matches.value_of("columns").unwrap();
    let columns: usize = columns.parse().unwrap();

    let mut stdin = stdin();
    let mut stdin = stdin.lock();
    let char_reader = stdin.chars();
    let stdout = stdout();
    let mut stdout = stdout.lock();
    let mut out_bytes = [0_u8; 4];

    let mut count = 0_usize;
    for mut c in char_reader {
        count += 1;
        if count % columns == 0 {
            println!();
        }
        let size = encode_utf8(c as u32, &mut out_bytes);
        stdout.write(&out_bytes[..size]).unwrap();
    }
}
