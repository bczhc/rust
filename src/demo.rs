extern crate lib;

use lib::complex_num::{ComplexValue, ComplexIntegral};
use std::f64::consts::PI;

fn main() {
    let r = ComplexValue::get_definite_integral_by_trapezium(0.0, PI, 100000, |t| {
        return ComplexValue::new(t.cos(), t.sin());
    });

    println!("{:?}", r);
}