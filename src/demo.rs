use bczhc_lib::utils::get_args_without_self_path;
use clap::{App, Arg};
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
    for n in 1..a.len() {
        result += a[n] * f64::cos(n as f64 * omega * t) + b[n] * f64::sin(n as f64 * omega * t);
    }
    result
}

#[inline]
fn linear_interpolate(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

struct Config {
    src: String,
    dest: String,
    series_n: u32,
    integral_segments_1s: u32,
    thread_num: usize,
}

fn main() {
    let cpu_num_string = num_cpus::get().to_string();
    let matches = App::new("audio-fourier-series")
        .arg(Arg::with_name("src").required(true))
        .arg(Arg::with_name("dest").required(true))
        .arg(Arg::with_name("series-count").required(true))
        .arg(Arg::with_name("integral-segments-in-1s").required(true))
        .arg(
            Arg::with_name("thread-num")
                .required(false)
                .default_value(cpu_num_string.as_str()),
        )
        .get_matches();

    let config = Config {
        src: String::from(matches.value_of("src").unwrap()),
        dest: String::from(matches.value_of("dest").unwrap()),
        series_n: matches.value_of("series-count").unwrap().parse().unwrap(),
        integral_segments_1s: matches
            .value_of("integral-segments-in-1s")
            .unwrap()
            .parse()
            .unwrap(),
        thread_num: matches.value_of("thread-num").unwrap().parse().unwrap(),
    };

    let mut reader = WavReader::open(&config.src).unwrap();
    let samples: Vec<_> = reader
        .samples::<i32>()
        .collect::<Vec<_>>()
        .into_iter()
        .map(|x| x.unwrap() as f64 / i32::MAX as f64)
        .collect();

    let sample_rate = reader.spec().sample_rate as usize;
    let samples_len = samples.len();
    println!("Total samples: {}", samples_len);

    let total_period_integral_segments =
        ((samples_len as f64 / sample_rate as f64) * config.integral_segments_1s as f64) as u32;

    // t is in 0..samples_len
    let p = samples.as_ptr() as usize;
    let interpolation = move |mut t: f64| {
        let mut i = f64::floor(t) as usize;
        if i + 1 >= samples_len {
            i = samples_len - 2;
        }
        if i <= 0 {
            i = 0;
        }

        let out = t - f64::floor(t);
        assert!(out >= 0.0 && out <= 1.0);

        let samples = unsafe { std::slice::from_raw_parts(p as *const f64, samples_len) };
        linear_interpolate(samples[i], samples[i + 1], out)
    };

    println!("Audio Fourier series:");

    let coefficients = trig_fourier_series_calc(
        move |t| interpolation(t),
        samples_len as f64,
        config.series_n,
        total_period_integral_segments,
        config.thread_num,
    );

    let mut writer = WavWriter::create(
        &config.dest,
        WavSpec {
            channels: 1,
            sample_rate: sample_rate as u32,
            bits_per_sample: 32,
            sample_format: SampleFormat::Int,
        },
    )
    .unwrap();

    for i in 0..samples_len {
        writer
            .write_sample(
                (fourier_series_evaluate(&coefficients, samples_len as f64, i as f64)
                    * i32::MAX as f64) as i32,
            )
            .unwrap();
    }
    println!("Done");
}
