use bczhc_lib::complex_num::ComplexValueF64;
use bczhc_lib::epicycle::Epicycle;
use bczhc_lib::fourier_series::{fourier_series_calc, EvaluatePath, LinearPath};
use bczhc_lib::point::PointF64;
use clap::{App, Arg};
use std::sync::Mutex;

fn main() {
    let thread_count_string = num_cpus::get().to_string();
    let matches = App::new("fourier-series-demo")
        .arg(Arg::with_name("epicycles-count").default_value("100"))
        .arg(Arg::with_name("period").default_value("100"))
        .arg(Arg::with_name("thread-count").default_value(thread_count_string.as_str()))
        .arg(Arg::with_name("integral-segments").default_value("10000"))
        .get_matches();

    let epicycles_count: u32 = matches
        .value_of("epicycles-count")
        .unwrap()
        .parse()
        .unwrap();
    let period: f64 = matches.value_of("period").unwrap().parse().unwrap();
    let thread_count: u32 = matches.value_of("thread-count").unwrap().parse().unwrap();
    let integral_segments: u32 = matches
        .value_of("integral-segments")
        .unwrap()
        .parse()
        .unwrap();

    let mut vec = Vec::new();

    let read = include_str!("../lib/data/fourier-series-data.txt");
    for line in read.lines() {
        let mut split = line.split(", ");
        let x: f64 = split.next().unwrap().parse().unwrap();
        let y: f64 = split.next().unwrap().parse().unwrap();
        vec.push(PointF64::new(x, y));
    }

    let path_evaluator = LinearPath::new(&vec);
    let path_evaluator_pointer = &path_evaluator as *const LinearPath as usize;

    let vec = Vec::new();
    let vec_mutex = Mutex::new(vec);

    let p = &vec_mutex as *const Mutex<Vec<Epicycle>> as usize;
    fourier_series_calc(
        epicycles_count,
        period,
        thread_count,
        integral_segments,
        move |t| unsafe {
            let path_evaluator = &*(path_evaluator_pointer as *const LinearPath);
            let point = path_evaluator.evaluate(t / period);
            ComplexValueF64::new(point.x, point.y)
        },
        move |r| unsafe {
            let mut guard = (&*(p as *mut Mutex<Vec<Epicycle>>)).lock().unwrap();
            println!("{:?}", r);
            guard.push(r);
        },
    );

    let guard = vec_mutex.lock().unwrap();
    println!("{:?}", &*guard);
}
