use hound::{SampleFormat, WavReader, WavSpec, WavWriter};
use rayon::prelude::*;

use wav_fourier_series::cli::build_cli;
use wav_fourier_series::{
    fourier_series_evaluate, linear_interpolate, trig_fourier_series_calc, Config,
};

fn main() {
    let matches = build_cli().get_matches();

    let config = Config {
        src: String::from(matches.get_one::<String>("src").unwrap()),
        dest: String::from(matches.get_one::<String>("dest").unwrap()),
        series_n: *matches.get_one("series-count").unwrap(),
        integral_segments_1s: *matches.get_one("integral-segments-in-1s").unwrap(),
    };

    let thread_num = *matches.get_one::<usize>("thread-num").unwrap();
    rayon::ThreadPoolBuilder::new()
        .num_threads(thread_num)
        .build_global()
        .unwrap();

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

    let coefficients = trig_fourier_series_calc(
        interpolation,
        samples_len as f64,
        config.series_n,
        total_period_integral_segments,
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
        .map(|sample_n| fourier_series_evaluate(&coefficients, samples_len as f64, sample_n as f64))
        .map(|x| (x * i32::MAX as f64) as i32)
        .collect::<Vec<_>>();

    println!("Writing wav...");

    for r in result_samples_vec {
        writer.write_sample(r).unwrap();
    }

    println!("Done");
}
