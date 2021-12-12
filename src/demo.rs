use std::io::stdin;
use bczhc_lib::io::ReadLines;
use std::fs::File;

fn main() {
    let mut stdin = stdin();
    let lines = stdin.lines();
    for line in lines {
        println!("{}", line);
    }
}
