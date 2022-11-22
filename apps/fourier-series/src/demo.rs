use std::f64::consts::PI;

use bczhc_lib::complex::integral::{Boole, Integrate};
use num_complex::Complex64;

fn main() {
    let function =
        |t: f64| Complex64::new(f64::sqrt(t), f64::sin(2.0 * t) + f64::sqrt(t) / (t + 1.0));

    let r = Boole::complex_integral_rayon(100000, 0.0, 2.0 * PI, function);
    println!("{:?}", r);
}
