use hound::{SampleFormat, WavReader, WavSpec, WavWriter};
use std::f64::consts::PI;
use std::sync::{Arc, Mutex};
use threadpool::ThreadPool;

fn definite_integrate<F>(f: F, bounds: (f64, f64), segments: u32) -> f64
where
    F: Fn(f64) -> f64,
{
    let bounds_length = bounds.1 - bounds.0;
    let increment = (bounds_length) / segments as f64;

    let mut sum = 0.0;
    let mut i = 0.0;
    while i + increment <= bounds.1 {
        sum += (f(i) + f(i + increment)) / 2.0;
        i += increment;
    }
    sum
}

#[derive(Debug)]
struct SeriesCoefficients(Vec<f64>, Vec<f64>);

fn trig_fourier_series_calc<F>(
    f: F,
    period: f64,
    series_n: u32,
    integral_segments: u32,
    threads: usize,
) -> SeriesCoefficients
where
    F: Fn(f64) -> f64 + Sync + 'static + Send + Copy,
{
    let mut an = vec![0.0; series_n as usize + 1];
    let mut bn = vec![0.0; series_n as usize + 1];
    let omega = 2.0 * PI / period;

    let calc_an = move |n: u32| {
        (2.0 / period)
            * definite_integrate(
                |t| f(t) * f64::cos(n as f64 * omega * t),
                (0.0, period),
                integral_segments,
            )
    };
    let calc_bn = move |n: u32| {
        (2.0 / period)
            * definite_integrate(
                |t| f(t) * f64::sin(n as f64 * omega * t),
                (0.0, period),
                integral_segments,
            )
    };

    let pool = ThreadPool::new(threads);

    let an_arc = Arc::new(Mutex::new(an));
    let bn_arc = Arc::new(Mutex::new(bn));

    let i_arc = Arc::new(Mutex::new(0));

    for n in 0..=series_n {
        let an = an_arc.clone();
        let bn = bn_arc.clone();
        let i = i_arc.clone();
        pool.execute(move || {
            let an_result = calc_an(n);
            let bn_result = calc_bn(n);

            let mut an_guard = an.lock().unwrap();
            let mut bn_guard = bn.lock().unwrap();
            (&mut *an_guard)[n as usize] = an_result;
            (&mut *bn_guard)[n as usize] = bn_result;

            let mut i_guard = i.lock().unwrap();
            *i_guard += 1;

            println!(
                "Progress: {}%",
                (*i_guard as f64 / (series_n + 1) as f64) * 100.0
            );
        });
    }
    pool.join();

    let an = Arc::try_unwrap(an_arc).unwrap().into_inner().unwrap();
    let bn = Arc::try_unwrap(bn_arc).unwrap().into_inner().unwrap();
    SeriesCoefficients(an, bn)
}

fn fourier_series_evaluate(coefficients: &SeriesCoefficients, period: f64, t: f64) -> f64 {
    let a = &coefficients.0;
    let b = &coefficients.1;
    let omega = 2.0 * PI / period;

    let mut result = a[0] / 2.0;
    for n in 0..a.len() {
        result += a[n] * f64::cos(n as f64 * omega * t) + b[n] * f64::sin(n as f64 * omega * t);
    }
    result
}

#[inline]
fn linear_interpolate(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

fn main() {
    let sample_rate = 1000_usize;

    let mut reader = WavReader::open("/home/bczhc/w-1000.wav").unwrap();
    let samples: Vec<_> = reader
        .samples::<i32>()
        .collect::<Vec<_>>()
        .into_iter()
        .map(|x| x.unwrap() as f64 / i32::MAX as f64)
        .collect();

    let samples_1s = &samples[..sample_rate];
    println!("{:?}", samples_1s);

    // t is in 0..sample_rate
    let p = samples_1s.as_ptr() as usize;
    let interpolation = move |mut t: f64| {
        let mut i = f64::floor(t) as usize;
        if i + 1 >= sample_rate {
            i = sample_rate - 2;
        }
        if i <= 0 {
            i = 0;
        }

        let out = t - f64::floor(t);
        assert!(out >= 0.0 && out <= 1.0);

        let samples_1s = unsafe { std::slice::from_raw_parts(p as *const f64, sample_rate) };
        linear_interpolate(samples_1s[i], samples_1s[i + 1], out)
    };

    println!("Audio Fourier series:");

    let coefficients = trig_fourier_series_calc(
        move |t| interpolation(t),
        sample_rate as f64,
        10000,
        (sample_rate * 100) as u32,
        num_cpus::get(),
    );

    println!("{:?}", coefficients);

    let mut writer = WavWriter::create(
        "/home/bczhc/a.wav",
        WavSpec {
            channels: 1,
            sample_rate: sample_rate as u32,
            bits_per_sample: 32,
            sample_format: SampleFormat::Int,
        },
    )
    .unwrap();

    for i in 0..sample_rate {
        writer
            .write_sample(
                (fourier_series_evaluate(&coefficients, sample_rate as f64, i as f64)
                    * i32::MAX as f64) as i32,
            )
            .unwrap();
    }
}
