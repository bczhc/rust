
use std::f64::consts::PI;

use bczhc_lib::complex_num::ComplexValueF64;

use bczhc_lib::complex_num::complex_integral::complex_integral_rayon;
use bczhc_lib::fourier_series;
use rayon::prelude::ParallelIterator;
use rayon::prelude::{IntoParallelIterator};


fn main() {
    let integral = complex_integral_rayon(1000000000, 0.0, 2.0 * PI, |t| {
        ComplexValueF64::from_exponent_form(t)
    });
    println!("{:?}", integral);
}
