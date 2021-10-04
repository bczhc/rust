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

/// t is in \[0, 1\]
#[inline]
fn fraction_part(t: f64) -> f64 {
    if (t as i32 as f64) == t {
        // `t` is a integer
        0.0
    } else {
        // get the fraction part of `t`
        if t > 0.0 {
            t - t.floor()
        } else {
            // x < 0.0
            (-t).ceil() + t
        }
    }
}

pub trait PathEvaluator {
    /// t is in \[0, 1\]
    fn evaluate(&self, t: f64) -> PointF64;
}

pub struct LinearPath<'a> {
    lines_len_sum: f64,
    points_vec: &'a Vec<PointF64>,
    lines_len_vec: Vec<f64>,
    lines_len_sum_vec: Vec<f64>,
}

#[inline]
fn line_length(p0: &PointF64, p1: &PointF64) -> f64 {
    f64::sqrt((p0.x - p1.x).powf(2.0) + (p0.y - p1.y).powf(2.0))
}

impl<'a> LinearPath<'a> {
    pub fn new(input: &'a Vec<PointF64>) -> LinearPath {
        let len = input.len();
        if len <= 1 {
            panic!("The points length must be > 1")
        }
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
        }
    }
}

impl<'a> PathEvaluator for LinearPath<'a> {
    // TODO can use binary search
    fn evaluate(&self, t: f64) -> PointF64 {
        let t = fraction_part(t);
        let len_in_total_len = t * self.lines_len_sum;

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

/// t is in \[0, 1\]
fn linear_bezier(p0: &PointF64, p1: &PointF64, t: f64) -> Point<f64> {
    *p0 + (*p1 - *p0) * t
}

#[cfg(test)]
mod test {
    use crate::complex_num::ComplexValueF64;
    use crate::epicycle::Epicycle;
    use crate::fourier_series::{fourier_series_calc, fraction_part, LinearPath, PathEvaluator};
    use crate::point::PointF64;
    use std::sync::Mutex;

    #[test]
    fn fraction_part_test() {
        let mut f = -10.0;
        while f < 10.0 {
            assert_eq!(fraction_part(f), 0.0);
            f += 1.0;
        }
        float_cmp::assert_approx_eq!(f64, fraction_part(3.2), 0.2);
        float_cmp::assert_approx_eq!(f64, fraction_part(12345.54321), 0.54321, epsilon = 0.00002);
        float_cmp::assert_approx_eq!(f64, fraction_part(-4.3), 0.7);
        float_cmp::assert_approx_eq!(
            f64,
            fraction_part(-54321.12345),
            1.0 - 0.12345,
            epsilon = 0.00002
        );
    }

    #[test]
    fn fourier_series_compute() {
        let mut vec = Vec::new();
        let read = include_str!("../data/fourier-series-data.txt");
        for line in read.lines() {
            let mut split = line.split(", ");
            let split: Vec<&str> = split.collect();
            assert_eq!(split.len(), 2);

            let x: f64 = split[0].parse().unwrap();
            let y: f64 = split[1].parse().unwrap();
            vec.push(PointF64::new(x, y));
        }

        let path_evaluator = LinearPath::new(&vec);
        let path_evaluator_pointer = &path_evaluator as *const LinearPath as usize;

        let mut vec: Vec<Epicycle> = Vec::new();
        let vec_mutex = Mutex::new(vec);
        let p = &vec_mutex as *const Mutex<Vec<Epicycle>> as usize;

        let period = 100.0;
        fourier_series_calc(
            100,
            100.0,
            4,
            10000,
            move |t| unsafe {
                let path_evaluator = &*(path_evaluator_pointer as *const LinearPath);
                let point = path_evaluator.evaluate(t / period);
                ComplexValueF64::new(point.x, point.y)
            },
            move |r| unsafe {
                let mut guard = (&mut *(p as *mut Mutex<Vec<Epicycle>>)).lock().unwrap();
                guard.push(r);
            },
        );

        let guard = vec_mutex.lock().unwrap();

        let mut result_vec = Vec::new();

        let result_text = include_str!("../data/fourier-series-result.txt");
        for line in result_text.lines() {
            let split = line.split(" ");
            let split: Vec<&str> = split.collect();
            assert_eq!(split.len(), 4);

            result_vec.push(Epicycle {
                n: split[0].parse().unwrap(),
                a: ComplexValueF64::new(split[1].parse().unwrap(), split[2].parse().unwrap()),
                p: split[3].parse().unwrap(),
            })
        }

        let find_result = |n: i32| {
            for r in &result_vec {
                if r.n == n {
                    return r;
                }
            }
            unreachable!()
        };

        let cmp_epicycle = |e1: Epicycle, e2: Epicycle| {
            return e1.n == e2.n
                || float_cmp::approx_eq!(f64, e1.a.re, e2.a.re)
                || float_cmp::approx_eq!(f64, e1.a.im, e2.a.im)
                || float_cmp::approx_eq!(f64, e1.p, e2.p);
        };

        for epicycle in &*guard {
            let found = find_result(epicycle.n);
            assert!(cmp_epicycle(*epicycle, *found));
        }
    }
}

pub struct TimePath<'a> {
    points: &'a Vec<PointF64>,
    segments_num: usize,
    segment_time: f64,
    points_len_minus_1: usize,
}

impl<'a> TimePath<'a> {
    pub fn new(points: &'a Vec<PointF64>) -> TimePath<'a> {
        let len = points.len();
        if len <= 1 {
            panic!("The points length must be > 1");
        }
        let segments_num = len - 1;
        let segment_time = 1.0 / segments_num as f64;

        Self {
            points,
            segments_num,
            segment_time,
            points_len_minus_1: points.len() - 1,
        }
    }
}

impl<'a> PathEvaluator for TimePath<'a> {
    /// `t` is in \[0, 1\]
    fn evaluate(&self, t: f64) -> PointF64 {
        let t = fraction_part(t);
        let segment_index = (t / self.segment_time).floor() as usize;
        if segment_index == self.points_len_minus_1 {
            return self.points[self.points_len_minus_1];
        }
        let in_segment_t = t - (segment_index as f64) * self.segment_time;
        linear_bezier(
            &self.points[segment_index],
            &self.points[segment_index + 1],
            in_segment_t / self.segment_time,
        )
    }
}
