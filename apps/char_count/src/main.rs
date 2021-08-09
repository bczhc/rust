use bczhc_lib::utf8::utf8_bytes_length;

use std::io::{stdin, Read};

fn main() {
    let length = char_length(&mut stdin());
    println!("{:?}", length);
}

fn char_length<T>(reader: &mut T) -> i32
where
    T: Read,
{
    let mut buf: [u8; 4] = [0, 0, 0, 0];

    let mut c = 0_i32;
    loop {
        let result = reader.read_exact(&mut buf[0..1]);
        if let Err(_) = result {
            break;
        }

        let length = utf8_bytes_length(buf[0]);
        // skip (length - 1) bytes
        std::io::copy(&mut reader.take(length as u64 - 1), &mut std::io::sink()).unwrap();
        c += 1;
    }
    c
}
