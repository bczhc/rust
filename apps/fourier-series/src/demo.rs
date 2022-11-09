
use bczhc_lib::complex_num::ComplexValueF64;

use bczhc_lib::fourier_series;
use rayon::prelude::ParallelIterator;
use rayon::prelude::{IntoParallelIterator};


fn main() {
    let f = |t| ComplexValueF64::new(t, f64::sin(t));

    let epicycle_count = 1000;
    let epicycle_count = epicycle_count;
    let n_to = epicycle_count / 2;
    let n_from = -(epicycle_count - n_to) + 1;

    let iter = (n_from..=n_to)
        .into_par_iter()
        .map(move |n| fourier_series::calc_n(100.0, 10000000, n, f));

    let vec = iter
        .map(|x| {
            println!("{:?}", x);
            x
        })
        .collect::<Vec<_>>();
    println!("{:?}", vec);
}
