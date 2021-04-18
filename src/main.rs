extern crate rust;

use std::fs::File;
use std::io::{stdin, BufReader, ErrorKind, Read};

use rust::lib::i32::ToStringRadix;

fn main() {
    let mut f = stdin();
    let mut buf: [u8; 3] = [0, 0, 0];
    let mut c = 0;
    loop {
        let i = f.read_exact(&mut buf);
        if let Err(v) = i {
            if let ErrorKind::UnexpectedEof = v.kind() {
                break;
            } else {
                panic!("{}", v);
            }
        }
        println!("{}", c);
        c += 1;
    }
}
