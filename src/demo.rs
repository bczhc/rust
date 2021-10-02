use std::io::{stdin, Read};

fn main() {
    let mut s = String::new();
    stdin().read_to_string(&mut s);

    let chars = s.chars();
    let mut size = 0_usize;
    for _ in chars {
        size += 1;
    }

    println!("{}", size);
}
