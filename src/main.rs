extern crate rust;

use rust::lib::i32::ToStringRadix;

fn main() {
    let a = [1, 2];

    let mut p = &a as *const i32;

    unsafe {
        println!("{}", *p);
        p = ((p as usize) + 4) as *const i32;
        println!("{}", *p);
    }
}