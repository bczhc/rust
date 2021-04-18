extern crate rust;

use rust::lib::i32::ToStringRadix;

fn main() {
    let mut a = 2;
    { let b = &mut a; }
    { let c = &mut a; }

    println!("{:?}", 1123123213.to_string_radix(12).unwrap());
}