use std::fs::File;
use std::io;
use std::io::{BufReader, Write};
use std::path::Path;
use std::str::FromStr;
use std::sync::mpsc::{channel, Receiver};
use std::thread::spawn;

use hound::{SampleFormat, WavReader, WavSpec, WavWriter};
use rayon::prelude::*;

use bczhc_lib::io::{OpenOrCreate, ReadLines};
use wav_fourier_series::cli::build_cli;
use wav_fourier_series::{
    fourier_series_evaluate, linear_interpolate, trig_fourier_series_calc, Config, Mode,
    SeriesCoefficient,
};

fn main() {
    let matches = build_cli().get_matches();

    let config = Config {
        src: String::from(matches.get_one::<String>("src").unwrap()),
        dest: String::from(matches.get_one::<String>("dest").unwrap()),
        series_n: *matches.get_one("series-count").unwrap(),
        integral_segments_1s: *matches.get_one("integral-segments-in-1s").unwrap(),
        mode: Mode::from_str(matches.get_one::<String>("mode").unwrap()).unwrap(),
    };

    let thread_num = *matches.get_one::<usize>("thread-num").unwrap();
    rayon::ThreadPoolBuilder::new()
        .num_threads(thread_num)
        .build_global()
        .unwrap();

    match config.mode {
        Mode::A2A => {
            let sample_info = read_wav_sample_info(&config.src);
            let coefficients =
                calc_coefficients(&config.src, config.series_n, config.integral_segments_1s);
            evaluate_series(
                &coefficients,
                &config.dest,
                sample_info.rate,
                sample_info.len,
            );
        }
        Mode::T2A => {
            let (coefficients, sample_info) = read_text(&config.src);
            evaluate_series(
                &coefficients,
                &config.dest,
                sample_info.rate,
                sample_info.len,
            );
        }
        Mode::A2T => {
            let coefficients =
                calc_coefficients(&config.src, config.series_n, config.integral_segments_1s);
            let sample_info = read_wav_sample_info(&config.src);
            let mut writer = File::open_or_create(&config.dest).unwrap();
            emit_text(&coefficients, &sample_info, &mut writer).unwrap();
        }
    }

    println!("Done");
}

fn read_wav_sample_info<P: AsRef<Path>>(src: P) -> SampleInfo {
    let mut reader = WavReader::open(src).unwrap();
    let sample_len = reader.samples::<i32>().len();
    let sample_rate = reader.spec().sample_rate;
    SampleInfo {
        rate: sample_rate,
        len: sample_len,
    }
}

struct SampleInfo {
    rate: u32,
    len: usize,
}

fn calc_coefficients<P: AsRef<Path>>(
    src: P,
    series_n: u32,
    integral_segments_1s: u32,
) -> Vec<SeriesCoefficient> {
    let mut reader = WavReader::open(src).unwrap();
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
        ((samples_len as f64 / sample_rate as f64) * integral_segments_1s as f64) as u32;

    // t is in 0..samples_len
    let p = samples.as_ptr() as usize;
    let interpolation = move |t: f64| {
        let mut i = f64::floor(t) as usize;
        if i + 1 >= samples_len {
            i = samples_len - 2;
        }

        let out = t - f64::floor(t);
        debug_assert!((0.0..=1.0).contains(&out));

        let samples = unsafe { std::slice::from_raw_parts(p as *const f64, samples_len) };
        linear_interpolate(samples[i], samples[i + 1], out)
    };

    println!("Audio Fourier series:");

    trig_fourier_series_calc(
        interpolation,
        samples_len as f64,
        series_n,
        total_period_integral_segments,
    )
}

fn emit_text<W: Write>(
    coefficients: &[SeriesCoefficient],
    sample_info: &SampleInfo,
    writer: &mut W,
) -> io::Result<()> {
    writeln!(writer, "sample_rate: {}", sample_info.rate)?;
    writeln!(writer, "samples_len: {}", sample_info.len)?;
    for co in coefficients {
        writeln!(writer, "{} {}", co.0, co.1)?;
    }
    Ok(())
}

fn read_text<P: AsRef<Path>>(path: P) -> (Vec<SeriesCoefficient>, SampleInfo) {
    let mut reader = BufReader::new(File::open(path).unwrap());

    let mut lines = reader.lines();
    let sample_rate = lines
        .next()
        .unwrap()
        .split("sample_rate: ")
        .last()
        .unwrap()
        .parse::<u32>()
        .unwrap();
    let samples_len = lines
        .next()
        .unwrap()
        .split("samples_len: ")
        .last()
        .unwrap()
        .parse::<usize>()
        .unwrap();
    println!("{:?}", (sample_rate, samples_len));
    let lines = lines.collect::<Vec<_>>();
    println!("Epicycle count: {}", lines.len());
    let coefficients = lines
        .into_par_iter()
        .map(|line| {
            let parsed = line
                .split_whitespace()
                .map(|x| x.parse::<f64>().unwrap())
                .collect::<Vec<_>>();
            SeriesCoefficient(parsed[0], parsed[1])
        })
        .collect::<Vec<_>>();
    (
        coefficients,
        SampleInfo {
            rate: sample_rate,
            len: samples_len,
        },
    )
}

fn evaluate_series<P: AsRef<Path>>(
    coefficients: &[SeriesCoefficient],
    audio_output: P,
    sample_rate: u32,
    samples_len: usize,
) {
    let mut writer = WavWriter::create(
        audio_output,
        WavSpec {
            channels: 1,
            sample_rate,
            bits_per_sample: 32,
            sample_format: SampleFormat::Int,
        },
    )
    .unwrap();

    println!("Evaluating Fourier series...");

    let (tx, rx) = channel();

    let t = spawn(move || {
        let rx: Receiver<(usize, i32)> = rx;
        let mut result_samples_vec = rx
            .iter()
            .enumerate()
            .map(|x| {
                if x.0 % 1000 == 0 {
                    println!("Progress: {}%", x.0 as f64 / samples_len as f64 * 100.0);
                }
                x.1
            })
            .collect::<Vec<_>>();
        result_samples_vec.sort_by_key(|x| x.0);

        println!("Writing wav...");

        for r in result_samples_vec {
            writer.write_sample(r.1).unwrap();
        }
    });

    (0..=(samples_len - 1))
        .into_par_iter()
        .map(|sample_n| {
            (
                sample_n,
                fourier_series_evaluate(coefficients, samples_len as f64, sample_n as f64),
            )
        })
        .map(|x| (x.0, (x.1 * i32::MAX as f64) as i32))
        .for_each_with(tx, |tx, x| {
            tx.send(x).unwrap();
        });

    t.join().unwrap();
}
