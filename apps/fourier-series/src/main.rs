use std::ffi::OsStr;
use std::fs::File;
use std::io::Read;


use clap::{value_parser, Arg, Command};
use once_cell::sync::Lazy;

use bczhc_lib::complex_num::ComplexValueF64;

use bczhc_lib::fourier_series::{
    compute_iter, EvaluatePath, LinearPath,
};
use bczhc_lib::point::PointF64;

const TEST_INPUT_DATA: &str = include_str!("../../../lib/data/fourier-series-data.txt");
static CPU_NUM_STRING: Lazy<String> = Lazy::new(|| num_cpus::get().to_string());

fn main() {
    let matches = Command::new("fourier-series")
        .arg(
            Arg::new("epicycle-count")
                .value_parser(value_parser!(u32))
                .default_value("100"),
        )
        .arg(
            Arg::new("period")
                .value_parser(value_parser!(f64))
                .default_value("100"),
        )
        .arg(
            Arg::new("thread-count")
                .value_parser(value_parser!(usize))
                .default_value(OsStr::new(&*CPU_NUM_STRING)),
        )
        .arg(
            Arg::new("integral-segments")
                .value_parser(value_parser!(u32))
                .default_value("10000"),
        )
        .arg(
            Arg::new("data")
                .required(false)
                .short('d')
                .long("data")
                .help("Input data set file (if not provided, use a built-in test data set)"),
        )
        .get_matches();

    let epicycles_count = *matches.get_one::<u32>("epicycle-count").unwrap();
    let period = *matches.get_one::<f64>("period").unwrap();
    let thread_count = *matches.get_one::<usize>("thread-count").unwrap();
    let integral_segments = *matches.get_one::<u32>("integral-segments").unwrap();
    let input_data_file = matches.get_one::<String>("data");

    let mut vec = Vec::new();

    let input_data = match input_data_file {
        None => String::from(TEST_INPUT_DATA),
        Some(file) => {
            let mut read = String::new();
            File::open(file).unwrap().read_to_string(&mut read).unwrap();
            read
        }
    };
    for line in input_data.lines() {
        let mut split = line.split(", ");
        let x: f64 = split.next().unwrap().parse().unwrap();
        let y: f64 = split.next().unwrap().parse().unwrap();
        vec.push(PointF64::new(x, y));
    }

    let path_evaluator = LinearPath::new(&vec);
    let path_evaluator_pointer = &path_evaluator as *const LinearPath as usize;

    let epicycle_count = epicycles_count as i32;
    let n_to = epicycle_count / 2;
    let n_from = -(epicycle_count - n_to) + 1;

    let thread_pool = rayon::ThreadPoolBuilder::new()
        .num_threads(thread_count)
        .build()
        .unwrap();

    let epicycles = thread_pool.install(|| {
        compute_iter(n_from, n_to, period, integral_segments, move |t| unsafe {
            let path_evaluator = &*(path_evaluator_pointer as *const LinearPath);
            let point = path_evaluator.evaluate(t / period);
            ComplexValueF64::new(point.x, point.y)
        })
    });
    let epicycles = epicycles
        .map(|e| {
            println!("{:?}", e);
            e
        })
        .collect::<Vec<_>>();
    println!("{:?}", epicycles);
}
