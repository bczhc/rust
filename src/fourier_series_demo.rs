use lib::fourier_series::{BezierCurvePath, BezierCurve, quadratic_bezier_curve_length, QuadraticBezierCurve};
use lib::io::ReadLine;
use lib::point::PointF64;
use std::fs::File;
use std::io::Read;

fn main() {
    for i in 0..100000 {
        println!("{}", quadratic_bezier_curve_length(&QuadraticBezierCurve {
            p0: PointF64::new(0.0, 10.0),
            p1: PointF64::new(1.0, 0.0),
            p2: PointF64::new(1.001, 0.0),
        }));
    }
    return;
    let mut input = Vec::new();

    let mut file = &mut File::open("./data").unwrap() as &mut dyn Read;
    loop {
        let line = file.read_line();
        if let None = line {
            break;
        }
        let line = line.unwrap();
        let mut split = line.split(' ');
        let x: f64 = split.next().unwrap().parse().unwrap();
        let y: f64 = split.next().unwrap().parse().unwrap();
        input.push(PointF64::new(x, y));
    }

    let path = BezierCurvePath::new(&input, 100.0);
    for i in 0..=100 {
        println!("{:?}", path.evaluate_path(i as f64));
    }
}
