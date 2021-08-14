use bczhc_lib::fs::ForeachDir;
use std::path::{Path, PathBuf};

fn test(period: f64, mut t: f64) -> f64 {
    let x = t / period;
    let floor = x.floor();
    if floor == x {
        // t is divisible by period (x is integer)
        t = 0.0;
    } else {
        if x > 0.0 {
            t -= floor * period;
        } else {
            // x < 0.0
            t += -floor * period;
        }
    }
    t
}

fn main() {
    for i in 0..1000 {
        println!("{}", test(10.0, i as f64 / 10.0));
    }
}
