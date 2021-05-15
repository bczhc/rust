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
    let x_0 = curve.p0.x;
    let y_0 = curve.p0.y;
    let x_1 = curve.p1.x;
    let y_1 = curve.p1.y;
    let x_2 = curve.p2.x;
    let y_2 = curve.p2.y;

    let mut t = 1.0;

    let t1 = integral_calc(x_0, y_0, x_1, y_1, x_2, y_2, t);
    t = 0.0;
    let t0 = integral_calc(x_0, y_0, x_1, y_1, x_2, y_2, t);
    return t1 - t0;
}

fn integral_calc(x_0: f64, y_0: f64, x_1: f64, y_1: f64, x_2: f64, y_2: f64, t: f64) -> f64 {
    ((f64::ln(
        t * x_0.powf(2.0) - x_0.powf(2.0) - 4.0 * t * x_1 * x_0
            + 3.0 * x_1 * x_0
            + 2.0 * t * x_2 * x_0
            - x_2 * x_0
            + 4.0 * t * x_1.powf(2.0)
            - 2.0 * x_1.powf(2.0)
            + t * x_2.powf(2.0)
            + t * y_0.powf(2.0)
            - y_0.powf(2.0)
            + 4.0 * t * y_1.powf(2.0)
            - 2.0 * y_1.powf(2.0)
            + t * y_2.powf(2.0)
            - 4.0 * t * x_1 * x_2
            + x_1 * x_2
            - 4.0 * t * y_0 * y_1
            + 3.0 * y_0 * y_1
            + 2.0 * t * y_0 * y_2
            - y_0 * y_2
            - 4.0 * t * y_1 * y_2
            + y_1 * y_2
            + f64::sqrt(
                x_0.powf(2.0)
                    + (2.0 * x_2 - 4.0 * x_1) * x_0
                    + 4.0 * x_1.powf(2.0)
                    + x_2.powf(2.0)
                    + y_0.powf(2.0)
                    + 4.0 * y_1.powf(2.0)
                    + y_2.powf(2.0)
                    - 4.0 * x_1 * x_2
                    - 4.0 * y_0 * y_1
                    + 2.0 * y_0 * y_2
                    - 4.0 * y_1 * y_2,
            ) * f64::sqrt(
                x_2.powf(2.0) * t.powf(2.0)
                    + y_0.powf(2.0) * t.powf(2.0)
                    + 4.0 * y_1.powf(2.0) * t.powf(2.0)
                    + y_2.powf(2.0) * t.powf(2.0)
                    - 4.0 * y_0 * y_1 * t.powf(2.0)
                    + 2.0 * y_0 * y_2 * t.powf(2.0)
                    - 4.0 * y_1 * y_2 * t.powf(2.0)
                    - 2.0 * y_0.powf(2.0) * t
                    - 4.0 * y_1.powf(2.0) * t
                    + 2.0 * (1.0 - 2.0 * t) * x_1 * x_2 * t
                    + 6.0 * y_0 * y_1 * t
                    - 2.0 * y_0 * y_2 * t
                    + 2.0 * y_1 * y_2 * t
                    + (t - 1.0).powf(2.0) * x_0.powf(2.0)
                    + (1.0 - 2.0 * t).powf(2.0) * x_1.powf(2.0)
                    + y_0.powf(2.0)
                    + y_1.powf(2.0)
                    - 2.0 * (t - 1.0) * x_0 * ((2.0 * t - 1.0) * x_1 - t * x_2)
                    - 2.0 * y_0 * y_1,
            ),
    ) * (x_2 * (y_1 - y_0) + x_1 * (y_0 - y_2) + x_0 * (y_2 - y_1)).powf(2.0))
        / (x_0.powf(2.0)
            + (2.0 * x_2 - 4.0 * x_1) * x_0
            + 4.0 * x_1.powf(2.0)
            + x_2.powf(2.0)
            + y_0.powf(2.0)
            + 4.0 * y_1.powf(2.0)
            + y_2.powf(2.0)
            - 4.0 * x_1 * x_2
            - 4.0 * y_0 * y_1
            + 2.0 * y_0 * y_2
            - 4.0 * y_1 * y_2)
            .powf(3.0 / 2.0))
        + f64::sqrt(
            x_2.powf(2.0) * t.powf(2.0)
                + y_0.powf(2.0) * t.powf(2.0)
                + 4.0 * y_1.powf(2.0) * t.powf(2.0)
                + y_2.powf(2.0) * t.powf(2.0)
                - 4.0 * y_0 * y_1 * t.powf(2.0)
                + 2.0 * y_0 * y_2 * t.powf(2.0)
                - 4.0 * y_1 * y_2 * t.powf(2.0)
                - 2.0 * y_0.powf(2.0) * t
                - 4.0 * y_1.powf(2.0) * t
                + 2.0 * (1.0 - 2.0 * t) * x_1 * x_2 * t
                + 6.0 * y_0 * y_1 * t
                - 2.0 * y_0 * y_2 * t
                + 2.0 * y_1 * y_2 * t
                + (t - 1.0).powf(2.0) * x_0.powf(2.0)
                + (1.0 - 2.0 * t).powf(2.0) * x_1.powf(2.0)
                + y_0.powf(2.0)
                + y_1.powf(2.0)
                - 2.0 * (t - 1.0) * x_0 * ((2.0 * t - 1.0) * x_1 - t * x_2)
                - 2.0 * y_0 * y_1,
        ) * (t
            + (-x_0.powf(2.0) + (3.0 * x_1 - x_2) * x_0 - 2.0 * x_1.powf(2.0) + x_1 * x_2
                - (y_0 - y_1) * (y_0 - 2.0 * y_1 + y_2))
                / (x_0.powf(2.0)
                    + (2.0 * x_2 - 4.0 * x_1) * x_0
                    + 4.0 * x_1.powf(2.0)
                    + x_2.powf(2.0)
                    + y_0.powf(2.0)
                    + 4.0 * y_1.powf(2.0)
                    + y_2.powf(2.0)
                    - 4.0 * x_1 * x_2
                    - 4.0 * y_0 * y_1
                    + 2.0 * y_0 * y_2
                    - 4.0 * y_1 * y_2))
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
