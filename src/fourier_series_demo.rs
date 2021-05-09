extern crate lib;

use lib::fourier_series::{quadratic_bezier_curve_length, QuadraticBezierCurve};
use lib::point::PointF64;

fn main() {
    let length = quadratic_bezier_curve_length(&QuadraticBezierCurve {
        p0: PointF64 { x: 0.0, y: 10.0 },
        p1: PointF64 { x: 0.0, y: 0.0 },
        p2: PointF64 { x: 10.0, y: 0.0 },
    });

    println!("{}", length);
}
