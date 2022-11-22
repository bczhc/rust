use bczhc_lib::{
    complex::{
        self,
        integral::{self, Integrate},
    },
    fourier_series::Epicycles,
};
use cli::Integrator;
use num_complex::{Complex, Complex64};
use num_traits::{Float, NumAssign};
use once_cell::sync::Lazy;

pub mod cli;
type ComplexValueF64 = Complex64;

pub const TEST_INPUT_DATA: &str = include_str!("../../../lib/data/fourier-series-data.txt");
pub static CPU_NUM_STRING: Lazy<String> = Lazy::new(|| num_cpus::get().to_string());

// pub fn compute_iter<F>(
//     integrator: Integrator,
//     n_from: i32,
//     n_to: i32,
//     period: f64,
//     integral_segments: u32,
//     function: F,
// ) -> Epicycles<impl Integrate, F>
// where
//     F: Fn(f64) -> ComplexValueF64 + Send + Copy,
// {
//     use bczhc_lib::complex::integral as methods;
//     use bczhc_lib::fourier_series;

//     match integrator {
//         Integrator::Trapezoid => fourier_series::compute_iter::<methods::Trapezoid, _>(
//             n_from,
//             n_to,
//             period,
//             integral_segments,
//             function,
//         ),
//         Integrator::LeftRectangle => fourier_series::compute_iter::<methods::LeftRectangle, _>(
//             n_from,
//             n_to,
//             period,
//             integral_segments,
//             function,
//         ),
//         Integrator::RightRectangle => fourier_series::compute_iter::<methods::RightRectangle, _>(
//             n_from,
//             n_to,
//             period,
//             integral_segments,
//             function,
//         ),
//         Integrator::Simpson => fourier_series::compute_iter::<methods::Simpson, _>(
//             n_from,
//             n_to,
//             period,
//             integral_segments,
//             function,
//         ),
//         Integrator::Simpson38 => fourier_series::compute_iter::<methods::Simpson, _>(
//             n_from,
//             n_to,
//             period,
//             integral_segments,
//             function,
//         ),
//         Integrator::Boole => fourier_series::compute_iter::<methods::Boole, _>(
//             n_from,
//             n_to,
//             period,
//             integral_segments,
//             function,
//         ),
//     }
// }
