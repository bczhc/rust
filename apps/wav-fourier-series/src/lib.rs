use rayon::prelude::*;

use std::f64::consts::PI;
use std::str::FromStr;

pub mod cli;

pub fn definite_integral_rayon<F>(function: F, bounds: (f64, f64), segments: u32) -> f64
where
    F: Fn(f64) -> f64 + Send + Sync,
{
    let x0 = bounds.0;
    let range = bounds.1 - bounds.0;
    let d = range / segments as f64;
    let multiplier = d / 6.0;

    (0..segments)
        .into_par_iter()
        .map(|x| x as f64)
        .map(move |n| {
            multiplier
                * (function(x0 + n * d)
                    + function(x0 + (n + 0.5) * d) * 4.0
                    + function(x0 + (n + 1.0) * d))
        })
        .sum()
}

#[derive(Debug, Clone)]
/// a_n, b_n
pub struct SeriesCoefficient(pub f64, pub f64);

impl Default for SeriesCoefficient {
    #[inline]
    fn default() -> Self {
        Self(0.0, 0.0)
    }
}

pub fn trig_fourier_series_calc<F>(
    f: F,
    period: f64,
    series_n: u32,
    integral_segments: u32,
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

pub fn fourier_series_evaluate(coefficients: &[SeriesCoefficient], period: f64, t: f64) -> f64 {
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
pub fn linear_interpolate(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

pub struct Config {
    pub src: String,
    pub dest: String,
    pub series_n: u32,
    pub integral_segments_1s: u32,
    pub mode: Mode,
}

pub enum Mode {
    A2A,
    T2A,
    A2T,
}

impl FromStr for Mode {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "a2a" => Ok(Mode::A2A),
            "t2a" => Ok(Mode::T2A),
            "a2t" => Ok(Mode::A2T),
            _ => Err(()),
        }
    }
}
