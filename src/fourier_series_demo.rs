use lib::complex_num::complex_integral::complex_integral;
use lib::complex_num::ComplexValueF64;
use lib::epicycle::Epicycle;
use std::f64::consts::PI;
use std::sync::mpsc::channel;
use threadpool::ThreadPool;

fn main() {
    fourier_series_calc(
        100,
        2.0 * PI,
        2,
        1000000,
        |t| ComplexValueF64::new(t, t),
        |r| {
            println!("{:?}", r);
        },
    );
}

fn fourier_series_calc<F: 'static, R: 'static>(
    epicycle_count: i32,
    period: f64,
    thread_count: i32,
    integral_separate_n: i32,
    function: F,
    result_callback: R,
) where
    F: Fn(f64) -> ComplexValueF64 + Send + Copy,
    R: Fn(Epicycle) + Send + Copy,
{
    let n_to = epicycle_count / 2;
    let n_from = -(epicycle_count - n_to) + 1;
    let omega = 2.0 * PI / period;
    let half_period = period / 2.0;

    let pool = ThreadPool::new(thread_count as usize);
    for n in n_from..=n_to {
        pool.execute(move || {
            let an = complex_integral(integral_separate_n, -half_period, half_period, |t| {
                ComplexValueF64::from_exponent_form(-(n as f64) * omega * t) * function(t)
            }) / period;
            let w = (n as f64) * omega;
            result_callback(Epicycle { n, a: an, w });
        });
    }
    pool.join();
}
