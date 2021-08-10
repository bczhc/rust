use bczhc_lib::complex_num::ComplexValueF64;
use bczhc_lib::epicycle::Epicycle;
use bczhc_lib::fourier_series::fourier_series_calc;
use std::f64::consts::PI;
use std::thread::spawn;

fn main() {
    let mut vec = Vec::new();

    let p = &vec as *const Vec<Epicycle> as usize;
    fourier_series_calc(
        100,
        PI * 2.0,
        2,
        10000,
        |t| ComplexValueF64::new(1.0, 2.0 + t),
        move |r| unsafe {
            (&mut *(p as *mut Vec<Epicycle>)).push(r);
        },
    );

    println!("{:?}", vec);
}
