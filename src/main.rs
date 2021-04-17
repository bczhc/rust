extern crate rust;

use rust::lib::i32::ToStringRadix;
use std::cell::RefCell;
use std::sync::Mutex;

fn main() {
    #[derive(Eq, PartialEq)]
    enum A {
        A, B
    }

    println!("{}", A::A == A::A);
}