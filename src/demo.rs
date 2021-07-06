use std::cmp::max;
use std::collections::LinkedList;
use std::fs::File;
use lib::io::ReadLine;
use lib::point::PointF64;
use lib::fourier_series::LinearPath;

struct Solution;

fn main() {
    let mut points = Vec::new();

    let mut file = File::open("/home/bczhc/code/gtk/cmake-build-debug/data.txt").unwrap();
    loop {
        let line = file.read_line_without_line_terminator();
        if line == None {
            break;
        }
        let line = line.unwrap();

        let mut split = line.split(", ");
        let x: f64 = split.next().unwrap().parse().unwrap();
        let y: f64 = split.next().unwrap().parse().unwrap();
        points.push(PointF64::new(x, y));
    }

    println!("length: {}", points.len());

    let path = LinearPath::new(&points, 100.0);
    for i in 0..=100 {
        let point = path.evaluate_path(i as f64);
        println!("{:?}", point);
    }
}