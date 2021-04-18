extern crate rust;

use std::fs::File;
use std::io::{Read, stdin, BufReader};

use rust::lib::i32::ToStringRadix;

fn main() {
    let mut f = stdin();
    let mut buf = [0u8,0,0,0,0];
    let mut reader = BufReader::new(f);
    loop {
        let i = reader.read(&mut buf).unwrap();
        if i == 0 { break; }
        println!("{}", i);
    }
}