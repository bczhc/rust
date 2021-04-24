extern crate lib;

use lib::i32::ToStringRadix;

fn main() {
    println!("{:?}", 127123.to_string_radix(20));
}