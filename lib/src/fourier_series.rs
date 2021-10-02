use crate::complex_num::complex_integral::complex_integral;
use crate::complex_num::ComplexValueF64;
use crate::epicycle::Epicycle;
use crate::point::{Point, PointF64};
use std::f64::consts::PI;
use threadpool::ThreadPool;

pub fn fourier_series_calc<F: 'static, R: 'static>(
    epicycle_count: u32,
    period: f64,
    thread_count: u32,
    integral_segments: u32,
    function: F,
    result_callback: R,
) where
    F: Fn(f64) -> ComplexValueF64 + Send + Copy,
    R: Fn(Epicycle) + Send + Copy,
{
    let epicycle_count = epicycle_count as i32;
    let n_to = epicycle_count / 2;
    let n_from = -(epicycle_count - n_to) + 1;
    let omega = 2.0 * PI / period;
    let half_period = period / 2.0;

    let pool = ThreadPool::new(thread_count as usize);
    for n in n_from..=n_to {
        pool.execute(move || {
            let an = complex_integral(integral_segments, -half_period, half_period, |t| {
                ComplexValueF64::from_exponent_form(-(n as f64) * omega * t) * function(t)
            }) / period;
            result_callback(Epicycle {
                n,
                a: an,
                p: ((n as f64) * omega),
            });
        });
    }
    pool.join();
}

pub struct LinearPath<'a> {
    lines_len_sum: f64,
    points_vec: &'a Vec<PointF64>,
    lines_len_vec: Vec<f64>,
    lines_len_sum_vec: Vec<f64>,
    period: f64,
}

#[inline]
fn line_length(p0: &PointF64, p1: &PointF64) -> f64 {
    f64::sqrt((p0.x - p1.x).powf(2.0) + (p0.y - p1.y).powf(2.0))
}

impl<'a> LinearPath<'a> {
    pub fn new(input: &'a Vec<PointF64>, period: f64) -> LinearPath {
        let len = input.len();
        let mut len_sum = 0.0;
        let mut len_vec = Vec::with_capacity(len - 1);
        let mut lines_len_sum_vec = Vec::with_capacity(len - 1);
        for i in 0..(len - 1) {
            let line_len = line_length(&input[i], &input[i + 1]);
            len_sum += line_len;
            len_vec.push(line_len);
            lines_len_sum_vec.push(len_sum);
        }
        Self {
            lines_len_sum: len_sum,
            points_vec: input,
            lines_len_vec: len_vec,
            lines_len_sum_vec,
            period,
        }
    }

    #[inline]
    fn period_limit(mut t: f64, period: f64) -> f64 {
        let x = t / period;
        if (x as i32 as f64) == x {
            // t is divisible by period (x is integer)
            t = 0.0;
        } else {
            if x > 0.0 {
                t -= x.floor() * period;
            } else {
                // x < 0.0
                t += -x.floor() * period;
            }
        }
        t
    }

    /// the main period range of t is in `[0, period]`
    // TODO can use binary search
    pub fn evaluate_path(&self, mut t: f64) -> PointF64 {
        t = LinearPath::period_limit(t, self.period);
        let len_in_total_len = t / self.period * self.lines_len_sum;

        let mut count = 0.0;
        let mut i = 0;
        while i < self.lines_len_vec.len() {
            let line_len = self.lines_len_vec[i];
            count += line_len;
            if count >= len_in_total_len {
                break;
            }
            i += 1;
        }
        if t == 0.0 {
            return self.points_vec[0];
        }
        let previous_line_length = if i == 0 {
            0.0
        } else {
            self.lines_len_sum_vec[i - 1]
        };
        let line_t = (len_in_total_len - previous_line_length) / self.lines_len_vec[i];
        linear_bezier(&self.points_vec[i], &self.points_vec[i + 1], line_t)
    }
}

/// t is in `[0, 1]`
fn linear_bezier(p0: &PointF64, p1: &PointF64, t: f64) -> Point<f64> {
    *p0 + (*p1 - *p0) * t
}
