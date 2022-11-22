use std::{f64::consts::PI, time::Instant};

use bczhc_lib::complex::integral::{
    Boole, Integrate, LeftRectangle, RightRectangle, Simpson, Simpson38, Trapezoid,
};
use num_complex::Complex64;

fn main() {
    let segments = 100000;

    let now = Instant::now();
    let integral = Trapezoid::complex_integral_rayon(segments, 0.0, 2.0 * PI, |t| {
        Complex64::from_polar(1.0, t)
    });
    println!("{:?}", now.elapsed());
    println!("{:?}", integral);

    let now = Instant::now();
    let integral = LeftRectangle::complex_integral_rayon(segments, 0.0, 2.0 * PI, |t| {
        Complex64::from_polar(1.0, t)
    });
    println!("{:?}", now.elapsed());
    println!("{:?}", integral);

    let now = Instant::now();
    let integral = RightRectangle::complex_integral_rayon(segments, 0.0, 2.0 * PI, |t| {
        Complex64::from_polar(1.0, t)
    });
    println!("{:?}", now.elapsed());
    println!("{:?}", integral);

    let now = Instant::now();
    let integral =
        Simpson::complex_integral_rayon(segments, 0.0, 2.0 * PI, |t| Complex64::from_polar(1.0, t));
    println!("{:?}", now.elapsed());
    println!("{:?}", integral);

    let now = Instant::now();
    let integral = Simpson38::complex_integral_rayon(segments, 0.0, 2.0 * PI, |t| {
        Complex64::from_polar(1.0, t)
    });
    println!("{:?}", now.elapsed());
    println!("{:?}", integral);

    let now = Instant::now();
    let integral =
        Boole::complex_integral_rayon(segments, 0.0, 2.0 * PI, |t| Complex64::from_polar(1.0, t));
    println!("{:?}", now.elapsed());
    println!("{:?}", integral);
}
