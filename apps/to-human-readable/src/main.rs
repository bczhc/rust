use bytesize::ByteSize;
use clap::{Arg, Command};
use std::io::{stdin, BufRead};

fn main() {
    let matches = Command::new("to-human-readable")
        .arg(Arg::new("bytes").required(false))
        .get_matches();

    if !matches.is_present("bytes") {
        let stdin = stdin().lock();
        for line in stdin.lines() {
            print_bytes(&line.unwrap());
        }
    } else {
        let bytes = matches.value_of("bytes").unwrap();
        print_bytes(bytes);
    };
}

fn print_bytes(str: &str) {
    let result = str.parse::<u64>();
    match result {
        Ok(bytes) => {
            println!(
                "{}  {}  {}",
                str,
                ByteSize(bytes).to_string_as(false),
                ByteSize(bytes).to_string_as(true)
            );
        }
        Err(_) => {
            println!("{}  ?  ?", str);
        }
    }
}
