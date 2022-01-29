use bczhc_lib::utils::get_args_without_self_path;
use clap::{App, Arg};
use hound::{SampleFormat, WavReader, WavSpec, WavWriter};
use num::{FromPrimitive, Integer};
use std::alloc::Layout;
use std::convert::TryFrom;
use std::f64::consts::PI;
use std::io::{stdout, Write};
use std::iter::Sum;
use std::ops::{AddAssign, Deref, Div, Range, RangeBounds, RangeInclusive, Sub};
use std::process::Output;
use std::sync::{Arc, Mutex};
use std::thread::spawn;
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

#[derive(Debug, Clone)]
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

    let co_arc = Arc::new(Mutex::new(coefficients));
    let i_arc = Arc::new(Mutex::new(0));

    for n in 0..=series_n {
        let i = i_arc.clone();
        let co = co_arc.clone();
        pool.execute(move || {
            let an_result = calc_an(n);
            let bn_result = calc_bn(n);

            let mut co_guard = co.lock().unwrap();
            (&mut *co_guard)[n as usize] = SeriesCoefficient(an_result, bn_result);

            let mut i_guard = i.lock().unwrap();
            *i_guard += 1;

            println!(
                "Progress: {}%",
                (*i_guard as f64 / (series_n + 1) as f64) * 100.0
            );
        });
    }
    pool.join();
    Arc::try_unwrap(co_arc).unwrap().into_inner().unwrap()
}

fn fourier_series_evaluate(coefficients: &Vec<SeriesCoefficient>, period: f64, t: f64) -> f64 {
    let omega = 2.0 * PI / period;

    let mut result = coefficients[0].0 / 2.0;
    for n in 1..coefficients.len() {
        result += coefficients[n].0 * f64::cos(n as f64 * omega * t)
            + coefficients[n].1 * f64::sin(n as f64 * omega * t);
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

    println!("Evaluating Fourier series...");

    let ranges = separate_range(0..=(samples_len - 1), config.thread_num);

    let mut thread_handlers = Vec::with_capacity(ranges.len());

    let coefficient_p = &coefficients as *const Vec<SeriesCoefficient> as usize;

    let progress_i = Arc::new(Mutex::new(0_usize));

    let progress_display_divisor = samples_len / 1000;

    for i in 0..ranges.len() {
        let range = ranges[i].clone();
        let mut progress_i = progress_i.clone();
        let handler = spawn(move || {
            let coefficients = unsafe { &*(coefficient_p as *const Vec<SeriesCoefficient>) };

            let mut samples = Vec::with_capacity(range.clone().count());
            for j in range.clone() {
                let sample = (fourier_series_evaluate(coefficients, samples_len as f64, j as f64)
                    * i32::MAX as f64) as i32;
                samples.push(sample);

                let mut progress_i = progress_i.lock().unwrap();
                if *progress_i % progress_display_divisor == 0 {
                    println!(
                        "Progress: {}%",
                        *progress_i as f64 / samples_len as f64 * 100.0
                    );
                }
                *progress_i += 1;
            }
            assert_eq!(samples.len(), range.clone().count());
            (i, samples)
        });
        thread_handlers.push(handler);
    }

    let mut result_samples_vec = Vec::with_capacity(ranges.len());
    for handler in thread_handlers {
        result_samples_vec.push(handler.join().unwrap());
    }

    result_samples_vec.sort_by(|o1, o2| o1.0.cmp(&o2.0));

    println!("Writing wav...");

    for r in result_samples_vec {
        let r = r.1;
        for sample in r {
            writer.write_sample(sample).unwrap();
        }
    }

    println!("Done");
}

fn separate_range<Idx>(range: RangeInclusive<Idx>, mut segments: Idx) -> Vec<RangeInclusive<Idx>>
where
    Idx: num::ToPrimitive + Integer + Clone + Copy + FromPrimitive + Sum + AddAssign<Idx>,
{
    let len: Idx = range.end().clone() - range.start().clone() + Idx::from_i32(1).unwrap();

    if len < segments {
        segments = len;
    }

    let base: Idx = len / segments;

    let mut range_lengths = vec![base; segments.to_usize().unwrap()];
    *range_lengths.last_mut().unwrap() = Idx::from_u64(
        len.to_u64().unwrap() - base.to_u64().unwrap() * (range_lengths.len() - 1) as u64,
    )
    .unwrap();

    let mut result = Vec::with_capacity(range_lengths.len());

    let mut start = range.start().clone();
    for length in range_lengths {
        result.push(start..=(start + length - Idx::from_i32(1).unwrap()));
        start += length;
    }
    result
}
