extern crate lib;

use lib::complex_num::ComplexValue;

fn main() {
    let mut value = ComplexValue::new(1.0, 2.0);
    value /= ComplexValue { re: 1.0, im: 3.0 };
    println!("{:?}", value);
}