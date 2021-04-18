extern crate rust;

use std::fs::File;
use std::io::{BufReader, Read, stdin};

use rust::lib::i32::ToStringRadix;

fn main() {
    let mut f = stdin();
    let mut buf: [u8; 3] = [0, 0, 0];
    let mut c = 0;
    loop {
        let i = f.read(&mut buf).unwrap();
        if i == 0 { break; }
        println!("{} {}", i, c);
        c += 1;
    }
}