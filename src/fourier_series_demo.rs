use lib::fourier_series::BezierCurvePath;
use lib::point::PointF64;

fn main() {
    let points = vec![
        PointF64::new(0.0, 10.0),
        PointF64::new(0.0, 0.0),
        PointF64::new(10.0, 0.0),
    ];

    let curve_path = BezierCurvePath::new(&points, 100.0);
    for i in 0..=100 {
        let r = curve_path.evaluate_path(i as f64);
        println!("i: {}, r: {:?}", i, r);
    }
}
