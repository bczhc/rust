extern crate lib;

use std::collections::HashMap;
use std::io::{stdin, BufReader, Read};
use lib::utf8::{solve_utf8_bytes, get_utf8_bytes_length};
use std::cmp::Ordering;

fn main() {
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

        if !map.contains_key(&c) {
            map.insert(c, 0);
        } else {
            *map.get_mut(&c).unwrap() += 1;
        }
    }

    let mut vec: Vec<_> = map.iter().collect();
    vec.sort_by(|&a, &b| {
        return if a.1 > b.1 {
            Ordering::Greater
        } else if a.1 == b.1 {
            Ordering::Equal
        } else {
            Ordering::Less
        };
    });

    println!("{:?}", vec);
}