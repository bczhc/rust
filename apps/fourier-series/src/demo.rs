
use std::f64::consts::PI;

use bczhc_lib::complex::integral::complex_integral_rayon;
use num_complex::Complex64;


fn main() {
    let integral = complex_integral_rayon(1000000000, 0.0, 2.0 * PI, |t| {
        Complex64::from_polar(1.0, t)
        
    });
    println!("{:?}", integral);
}
