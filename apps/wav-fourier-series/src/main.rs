use std::f64::consts::PI;
use std::iter::Sum;
use std::ops::{AddAssign, RangeInclusive};
use std::sync::{Arc, Mutex};
use std::thread::spawn;

use clap::{value_parser, Arg, Command, ValueHint};
use hound::{SampleFormat, WavReader, WavSpec, WavWriter};
use num::{FromPrimitive, Integer};
use rayon::prelude::*;

fn definite_integral_rayon<F>(function: F, bounds: (f64, f64), segments: u32) -> f64
where
    F: Fn(f64) -> f64 + Send + Sync,
{
    let range = bounds.1 - bounds.0;
    let d = range / segments as f64;
    let x0 = bounds.0;

    (0..segments)
        .into_par_iter()
        .map(move |x| (function(x0 + x as f64 * d) + function(x0 + (x as f64 + 1.0) * d)) * d / 2.0)
        .sum()
}

#[derive(Debug, Clone)]
/// a_n, b_n
struct SeriesCoefficient(f64, f64);

impl Default for SeriesCoefficient {
    #[inline]
    fn default() -> Self {
        Self(0.0, 0.0)
    }
}

fn trig_fourier_series_calc<F>(
    f: F,
    period: f64,
    series_n: u32,
    integral_segments: u32,
    threads: usize,
) -> Vec<SeriesCoefficient>
where
    F: Fn(f64) -> f64 + Sync + 'static + Send + Copy,
{
    let mut coefficients = vec![SeriesCoefficient::default(); series_n as usize + 1];
    let omega = 2.0 * PI / period;

    let calc_an = move |n: u32| {
        (2.0 / period)
            * definite_integral_rayon(
                |t| f(t) * f64::cos(n as f64 * omega * t),
                (0.0, period),
                integral_segments,
            )
    };
    let calc_bn = move |n: u32| {
        (2.0 / period)
            * definite_integral_rayon(
                |t| f(t) * f64::sin(n as f64 * omega * t),
                (0.0, period),
                integral_segments,
            )
    };
    let calc_a0 =
        move || (1.0 / period) * definite_integral_rayon(f, (0.0, period), integral_segments);
    let calc_b0 = || 0.0;

    coefficients[0] = SeriesCoefficient(calc_a0(), calc_b0());
    for n in 1..=series_n {
        coefficients[n as usize] = SeriesCoefficient(calc_an(n), calc_bn(n));
        if n % 100 == 0 {
            println!("Progress: {}%", n as f64 / series_n as f64 * 100.0);
        }
    }
    coefficients
}

fn fourier_series_evaluate(coefficients: &[SeriesCoefficient], period: f64, t: f64) -> f64 {
    let omega = 2.0 * PI / period;

    coefficients
        .iter()
        .enumerate()
        .skip(1)
        .map(|(n, co)| {
            co.0 * f64::cos(n as f64 * omega * t) + co.1 * f64::sin(n as f64 * omega * t)
        })
        .sum::<f64>()
        + coefficients[0].0 / 2.0
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
    let cpu_num_string = num_cpus::get().to_string().into_boxed_str();
    let cpu_num_string: &str = Box::leak(cpu_num_string);

    let matches = Command::new("audio-fourier-series")
        .arg(
            Arg::new("src")
                .required(true)
                .value_hint(ValueHint::FilePath),
        )
        .arg(
            Arg::new("dest")
                .required(true)
                .value_hint(ValueHint::FilePath),
        )
        .arg(
            Arg::new("series-count")
                .required(true)
                .value_parser(value_parser!(u32)),
        )
        .arg(
            Arg::new("integral-segments-in-1s")
                .required(true)
                .value_parser(value_parser!(u32)),
        )
        .arg(
            Arg::new("thread-num")
                .required(false)
                .default_value(cpu_num_string)
                .value_parser(value_parser!(usize)),
        )
        .get_matches();

    let config = Config {
        src: String::from(matches.get_one::<String>("src").unwrap()),
        dest: String::from(matches.get_one::<String>("dest").unwrap()),
        series_n: *matches.get_one("series-count").unwrap(),
        integral_segments_1s: *matches.get_one("integral-segments-in-1s").unwrap(),
        thread_num: *matches.get_one("thread-num").unwrap(),
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

    let seconds = samples_len as f64 / sample_rate as f64;
    let total_period_integral_segments = (seconds * config.integral_segments_1s as f64) as u32;

    let p = samples.as_ptr() as usize;
    // t is in 0..seconds
    let interpolation = move |t: f64| {
        let t = t / seconds * samples_len as f64;
        let mut i = f64::floor(t) as usize;
        if i + 1 >= samples_len {
            i = samples_len - 2;
        }
        if i <= 0 {
            i = 0;
        }

        let out = t - f64::floor(t);
        assert!((0.0..=1.0).contains(&out));

        let samples = unsafe { std::slice::from_raw_parts(p as *const f64, samples_len) };
        linear_interpolate(samples[i], samples[i + 1], out)
    };

    println!("Audio Fourier series:");

    let coefficients = trig_fourier_series_calc(
        interpolation,
        seconds,
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

    println!("Evaluating Fourier series...");

    let result_samples_vec = (0..=(samples_len - 1))
        .into_par_iter()
        .map(|sample_n| {
            fourier_series_evaluate(
                &coefficients,
                seconds,
                sample_n as f64 / samples_len as f64 * seconds,
            )
        })
        .map(|x| (x * i32::MAX as f64) as i32)
        .collect::<Vec<_>>();

    println!("Writing wav...");

    for r in result_samples_vec {
        writer.write_sample(r).unwrap();
    }

    println!("Done");
}
