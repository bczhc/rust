extern crate lib;

use lib::i32::ToStringRadix;
use std::time::{SystemTime, UNIX_EPOCH};
use lib::time::get_current_time_millis;

fn main() {
    let mut c: u8 = 0;
    let start = get_current_time_millis();
    for _ in 0..100000000 {
        if c == 255 {c = 0;}
        c += 1;
    }
    let end = get_current_time_millis();
    println!("{}", c);
    println!("{}", end - start);
}