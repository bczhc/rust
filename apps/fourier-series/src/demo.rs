use std::{f64::consts::PI, time::Instant};

use bczhc_lib::complex::integral::{integral_trapezoid_rayon, integral_left_rectangle_rayon, integral_right_rectangle_rayon, integral_simpson_rayon, integral_simpson_38_rayon, integral_boole_rayon};
use num_complex::Complex64;

fn main() {
    let segments = 100000;

    let now = Instant::now();
    let integral =
        integral_trapezoid_rayon(segments, 0.0, 2.0 * PI, |t| Complex64::from_polar(1.0, t));
    println!("{:?}", now.elapsed());
    println!("{:?}", integral);

    let now = Instant::now();
    let integral =
        integral_left_rectangle_rayon(segments, 0.0, 2.0 * PI, |t| Complex64::from_polar(1.0, t));
    println!("{:?}", now.elapsed());
    println!("{:?}", integral);

    let now = Instant::now();
    let integral =
        integral_right_rectangle_rayon(segments, 0.0, 2.0 * PI, |t| Complex64::from_polar(1.0, t));
    println!("{:?}", now.elapsed());
    println!("{:?}", integral);

    let now = Instant::now();
    let integral =
        integral_simpson_rayon(segments, 0.0, 2.0 * PI, |t| Complex64::from_polar(1.0, t));
    println!("{:?}", now.elapsed());
    println!("{:?}", integral);

    let now = Instant::now();
    let integral =
        integral_simpson_38_rayon(segments, 0.0, 2.0 * PI, |t| Complex64::from_polar(1.0, t));
    println!("{:?}", now.elapsed());
    println!("{:?}", integral);

    let now = Instant::now();
    let integral =
        integral_boole_rayon(segments, 0.0, 2.0 * PI, |t| Complex64::from_polar(1.0, t));
    println!("{:?}", now.elapsed());
    println!("{:?}", integral);
}
