extern crate rust;

use rust::lib::i32::ToStringRadix;

fn main() {
    let x = '😂';
    println!("{}", x as u32);
    let surrogate = rust::lib::utf8::unicode_to_surrogate_pair(x as u32);
    println!("{}", rust::lib::utf8::surrogate_pair_to_unicode(surrogate.lead, surrogate.trail));
}