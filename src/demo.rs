use std::fs::File;
use std::io::{BufReader, Read};

fn main() {
    let br = BufReader::new(File::open("a").unwrap());

    let _r = &br as &dyn Read;
}
