extern crate lib;

use std::f64::consts::PI;
use lib::complex_num::ComplexValueF64;
use lib::complex_num::complex_integral::{IntegralCalculator, ComplexFunction};

fn main() {
    struct Function {
    }
    impl ComplexFunction for Function {
        fn x(&self, t: f64) -> ComplexValueF64 {
            ComplexValueF64::new(f64::sin(t), f64::sin(t))
        }
    }

    let integral_calculator = IntegralCalculator::new(10000, Function {});
    let value = integral_calculator.calc(0.0, 2.0 * PI);
    println!("{:?}", value);
}