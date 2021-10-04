use bczhc_lib::fourier_series::{PathEvaluator, TimePath};
use bczhc_lib::point::PointF64;
use std::io::{stdin, Read};

fn main() {
    let points = vec![
        PointF64::new(0.0, 10.0),
        PointF64::new(0.0, 0.0),
        PointF64::new(5.0, 0.0),
        PointF64::new(10.0, 0.0),
    ];
    let path_evaluator = TimePath::new(&points);

    let mut f = -10.0;
    while f <= 10.0 {
        println!("{} {:?}", f, path_evaluator.evaluate(f));
        f += 0.01;
    }
}
