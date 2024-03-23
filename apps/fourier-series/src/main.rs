use std::fs::File;
use std::io::{stdout, Read, Write};
use std::time::Instant;

use num_complex::Complex64;
use rayon::ThreadPool;

use bczhc_lib::complex::integral::{self, Integrate};
use bczhc_lib::fourier_series::{compute_iter, EvaluatePath, LinearPath};
use fourier_series::cli::build_cli;
use fourier_series::{cli, TEST_INPUT_DATA};

type ComplexValueF64 = Complex64;

type PointF64 = bczhc_lib::fourier_series::euclid::Point2D<f64, ()>;

fn main() {
    let matches = build_cli().get_matches();

    let epicycle_count = *matches.get_one::<u32>("epicycle-count").unwrap();
    let period = *matches.get_one::<f64>("period").unwrap();
    let thread_count = *matches.get_one::<usize>("thread-count").unwrap();
    let integral_segments = *matches.get_one::<u32>("integral-segments").unwrap();
    let input_data_file = matches.get_one::<String>("data");
    let integrator = *matches.get_one::<cli::Integrator>("integrator").unwrap();

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

    let thread_pool = rayon::ThreadPoolBuilder::new()
        .num_threads(thread_count)
        .build()
        .unwrap();

    let params = Params {
        epicycle_count,
        integral_segments,
        path_evaluator,
        period,
        thread_pool,
    };

    let start = Instant::now();

    match integrator {
        cli::Integrator::Trapezoid => params.calc_and_print::<integral::Trapezoid>(),
        cli::Integrator::LeftRectangle => params.calc_and_print::<integral::LeftRectangle>(),
        cli::Integrator::RightRectangle => params.calc_and_print::<integral::RightRectangle>(),
        cli::Integrator::Simpson => params.calc_and_print::<integral::Simpson>(),
        cli::Integrator::Simpson38 => params.calc_and_print::<integral::Simpson38>(),
        cli::Integrator::Boole => params.calc_and_print::<integral::Boole>(),
    }

    println!("{:?}", start.elapsed());

    // Expect the user to run the process by double clicking it directly
    // so add a pause here
    #[cfg(windows)]
    wait_for_enter();
}

#[cfg(windows)]
fn wait_for_enter() {
    print!("Press enter to continue...");
    stdout().flush().unwrap();
    std::io::stdin().read_line(&mut String::new()).unwrap();
}

struct Params<E>
where
    E: EvaluatePath<f64>,
{
    thread_pool: ThreadPool,
    epicycle_count: u32,
    period: f64,
    integral_segments: u32,
    path_evaluator: E,
}

impl<E> Params<E>
where
    E: EvaluatePath<f64>,
{
    fn calc_and_print<I>(self)
    where
        I: Integrate + Send,
    {
        let epicycle_count = self.epicycle_count as i32;
        let n_to = epicycle_count / 2;
        let n_from = -(epicycle_count - n_to) + 1;

        let path_evaluator_pointer = &self.path_evaluator as *const E as usize;

        let period = self.period;

        let _epicycles = self.thread_pool.install(|| {
            compute_iter::<I, _, _>(
                n_from,
                n_to,
                period,
                self.integral_segments,
                move |t| unsafe {
                    let path_evaluator = &*(path_evaluator_pointer as *const E);
                    let point = path_evaluator.evaluate(t / period);
                    ComplexValueF64::new(point.x, point.y)
                },
            )
            .inspect(|e| {
                println!("{:?}", e);
            })
            .collect::<Vec<_>>()
        });
    }
}
