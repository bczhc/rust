use crate::complex_num::complex_integral::complex_integral;
use crate::complex_num::ComplexValueF64;
use crate::epicycle::Epicycle;
use crate::point::PointF64;
use std::f64::consts::PI;
use threadpool::ThreadPool;

pub struct QuadraticBezierCurve {
    pub p0: PointF64,
    pub p1: PointF64,
    pub p2: PointF64,
}

pub type BezierCurve = QuadraticBezierCurve;

pub fn fourier_series_calc<F: 'static, R: 'static>(
    epicycle_count: i32,
    period: f64,
    thread_count: i32,
    integral_separate_n: i32,
    function: F,
    result_callback: R,
) where
    F: Fn(f64) -> ComplexValueF64 + Send + Copy,
    R: Fn(Epicycle) + Send + Copy,
{
    let n_to = epicycle_count / 2;
    let n_from = -(epicycle_count - n_to) + 1;
    let omega = 2.0 * PI / period;
    let half_period = period / 2.0;

    let pool = ThreadPool::new(thread_count as usize);
    for n in n_from..=n_to {
        pool.execute(move || {
            let an = complex_integral(integral_separate_n, -half_period, half_period, |t| {
                ComplexValueF64::from_exponent_form(-(n as f64) * omega * t) * function(t)
            }) / period;
            let w = (n as f64) * omega;
            result_callback(Epicycle { n, a: an, w });
        });
    }
    pool.join();
}

pub fn quadratic_bezier_curve_length(curve: &QuadraticBezierCurve) -> f64 {
    let x0 = curve.p0.x;
    let x1 = curve.p1.x;
    let x2 = curve.p2.x;
    let y0 = curve.p0.y;
    let y1 = curve.p1.y;
    let y2 = curve.p2.y;

    let dx0 = x1 - x0;
    let dx1 = x2 - x1;
    let dy0 = y1 - y0;
    let dy1 = y2 - y1;

    let a = (dx1 - dx0).powf(2.0) + (dy1 - dy0).powf(2.0);
    let b = dx0 * (dx1 - dx0) + dy0 * (dy1 - dy0);
    let c = dx0.powf(2.0) + dy0.powf(2.0);
    let d = a * c - b.powf(2.0);

    let mut t = 1.0;
    let t1 = (t + b / a) * f64::sqrt(a * t.powf(2.0) + 2.0 * b * t + c)
        + ((d) / a.powf(3.0 / 2.0)) * f64::asinh((a * t + b) / f64::sqrt(d));

    t = 0.0;

    let t0 = (t + b / a) * f64::sqrt(a * t.powf(2.0) + 2.0 * b * t + c)
        + ((d) / a.powf(3.0 / 2.0)) * f64::asinh((a * t + b) / f64::sqrt(d));

    return t1 - t0;
}

/// Evaluate a quadratic bézier curve
///
/// t is in `[0, 1]`
pub fn quadratic_bezier_curve(curve: &QuadraticBezierCurve, t: f64) -> PointF64 {
    curve.p0 * (t.powf(2.0) - 2.0 * t + 1.0)
        + curve.p1 * (-2.0 * t + 2.0 * t)
        + curve.p2 * (t.powf(2.0))
}

pub struct BezierCurvePath {
    curve_len_sum: f64,
    bezier_curve_vec: Vec<BezierCurve>,
    bezier_curve_len_vec: Vec<f64>,
    period: f64,
}

impl BezierCurvePath {
    pub fn new(input: &Vec<PointF64>, period: f64) -> BezierCurvePath {
        let mut input = input.clone();
        if input.len() == 0 {
            for _ in 0..3 {
                input.push(PointF64::new(0.0, 0.0));
            }
        }
        if input.len() == 1 {
            input.push(input[0]);
            input.push(input[0]);
        }
        if input.len() == 2 {
            input.push(input[0]);
        }
        if input[input.len() - 1] != input[0] {
            input.push(input[0]);
        }
        input.push(input[1]);

        let mut bezier_curve_vec = Vec::new();
        for i in 1..=(input.len() - 2) {
            bezier_curve_vec.push(BezierCurve {
                p0: PointF64::mid(&input[i - 1], &input[i]),
                p1: input[i],
                p2: PointF64::mid(&input[i], &input[i + 1]),
            });
        }

        let mut bezier_curve_length_vec = Vec::new();
        for i in 0..bezier_curve_vec.len() {
            let len = quadratic_bezier_curve_length(&bezier_curve_vec[i]);
            bezier_curve_length_vec.push(len);
        }

        let len_sum: f64 = bezier_curve_length_vec.iter().sum();
        return Self {
            curve_len_sum: len_sum,
            bezier_curve_vec,
            bezier_curve_len_vec: bezier_curve_length_vec,
            period,
        };
    }

    /// t is in `[0, period]`
    pub fn evaluate_path(&self, t: f64) -> PointF64 {
        let a = (t / self.period) * self.curve_len_sum;
        let mut sum = 0_f64;
        let mut bezier_curve_index = 0;
        for i in 0..self.bezier_curve_len_vec.len() {
            sum += self.bezier_curve_len_vec[i];
            if a <= sum {
                bezier_curve_index = i;
                break;
            }
        }
        let hit_curve_len = self.bezier_curve_len_vec[bezier_curve_index];
        let t = (a - (sum - hit_curve_len)) / hit_curve_len;
        return quadratic_bezier_curve(&self.bezier_curve_vec[bezier_curve_index], t);
    }
}
