use euclid::{vec2, Point2D, Vector2D};
use num_traits::{AsPrimitive, Float, FromPrimitive};

pub use euclid;

use crate::complex::integral::Integrate;
use crate::epicycle::Epicycle;
use std::f64::consts::PI;
use std::marker::PhantomData;
use std::ops::{AddAssign, DivAssign, Mul, MulAssign, SubAssign};

type Point<T> = Point2D<T, ()>;

type ComplexValueF64 = num_complex::Complex64;

pub trait FloatNum:
    Float + AddAssign + SubAssign + MulAssign + DivAssign + AsPrimitive<usize> + AsPrimitive<f64>
{
}

impl<T> FloatNum for T where
    T: Float
        + AddAssign
        + SubAssign
        + MulAssign
        + DivAssign
        + AsPrimitive<usize>
        + AsPrimitive<f64>
{
}

pub fn calc_n_rayon<I, F>(period: f64, integral_segments: u32, n: i32, function: F) -> Epicycle
where
    F: Fn(f64) -> ComplexValueF64 + Send + Sync + Copy,
    I: Integrate,
{
    let omega = 2.0 * PI / period;
    let half_period = period / 2.0;

    let an = integrate::<I, _>(integral_segments, -half_period, half_period, move |t| {
        ComplexValueF64::from_polar(1.0, -(n as f64) * omega * t) * function(t)
    }) / period;

    Epicycle {
        n,
        a: an,
        p: ((n as f64) * omega),
    }
}

/// t is in \[0, 1\]
#[inline]
fn fraction_part<F>(t: F) -> F
where
    F: Float,
{
    match t.fract() {
        a @ _ if a.is_sign_negative() => F::one() + a,
        a @ _ => a,
    }
}

pub trait EvaluatePath<T>
where
    T: FloatNum,
{
    /// t is in \[0, 1\]
    fn evaluate(&self, t: T) -> Point<T>;
}

pub struct LinearPath<'a, T>
where
    T: FloatNum,
{
    lines_len_sum: T,
    points_vec: &'a Vec<Point<T>>,
    lines_len_vec: Vec<T>,
    lines_len_sum_vec: Vec<T>,
}

impl<'a, T> LinearPath<'a, T>
where
    T: FloatNum,
{
    pub fn new(input: &'a Vec<Point<T>>) -> Self {
        let len = input.len();
        if len <= 1 {
            panic!("The points length must be > 1")
        }
        let mut len_sum = T::zero();
        let mut len_vec = Vec::with_capacity(len - 1);
        let mut lines_len_sum_vec = Vec::with_capacity(len - 1);
        for i in 0..(len - 1) {
            let line_len = input[i].distance_to(input[i + 1]);
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

impl<'a, T> EvaluatePath<T> for LinearPath<'a, T>
where
    T: FloatNum,
{
    // TODO can use binary search
    fn evaluate(&self, t: T) -> Point<T> {
        let t = fraction_part(t);
        let len_in_total_len = t * self.lines_len_sum;

        let mut count = T::zero();
        let mut i = 0;
        while i < self.lines_len_vec.len() {
            let line_len = self.lines_len_vec[i];
            count += line_len;
            if count >= len_in_total_len {
                break;
            }
            i += 1;
        }
        if t == T::zero() {
            return self.points_vec[0];
        }
        let previous_line_length = if i == 0 {
            T::zero()
        } else {
            self.lines_len_sum_vec[i - 1]
        };
        let line_t = (len_in_total_len - previous_line_length) / self.lines_len_vec[i];
        linear_bezier(self.points_vec[i], self.points_vec[i + 1], line_t)
    }
}

/// t is in \[0, 1\]
#[inline]
fn linear_bezier<T>(p0: Point<T>, p1: Point<T>, t: T) -> Point<T>
where
    T: FloatNum,
{
    p0 + (p1 - p0) * t
}

pub struct Epicycles<I, F>
where
    F: Fn(f64) -> ComplexValueF64 + Send + Copy,
    I: Integrate,
{
    n_to: i32,
    n: i32,
    period: f64,
    integral_segment: u32,
    function: F,
    _phantom: PhantomData<I>,
}

pub fn compute_iter<I, F>(
    n_from: i32,
    n_to: i32,
    period: f64,
    integral_segments: u32,
    function: F,
) -> Epicycles<I, F>
where
    F: Fn(f64) -> ComplexValueF64 + Send + Copy,
    I: Integrate,
{
    Epicycles {
        n_to,
        n: n_from,
        period,
        integral_segment: integral_segments,
        function,
        _phantom: PhantomData,
    }
}

impl<I, F> Iterator for Epicycles<I, F>
where
    F: Fn(f64) -> ComplexValueF64 + Send + Sync + Copy,
    I: Integrate,
{
    type Item = Epicycle;

    fn next(&mut self) -> Option<Self::Item> {
        if self.n > self.n_to {
            return None;
        }

        let epicycle =
            calc_n_rayon::<I, _>(self.period, self.integral_segment, self.n, self.function);
        self.n += 1;
        Some(epicycle)
    }
}

#[cfg(test)]
mod test {
    use crate::fourier_series::fraction_part;

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
}

pub struct TimePath<'a, T>
where
    T: FloatNum,
{
    points: &'a Vec<Point<T>>,
    segment_time: T,
    points_len_minus_1: usize,
}

impl<'a, T> TimePath<'a, T>
where
    T: FloatNum,
    usize: AsPrimitive<T>,
{
    pub fn new(points: &'a Vec<Point<T>>) -> Self {
        let len = points.len();
        if len <= 1 {
            panic!("The points length must be > 1");
        }
        let segments_num = len - 1;
        let segment_time = T::one() / AsPrimitive::<T>::as_(segments_num);

        Self {
            points,
            segment_time,
            points_len_minus_1: points.len() - 1,
        }
    }
}

impl<'a, T> EvaluatePath<T> for TimePath<'a, T>
where
    T: FloatNum,
    usize: AsPrimitive<T>,
{
    /// `t` is in \[0, 1\]
    fn evaluate(&self, t: T) -> Point<T> {
        let t = fraction_part(t);
        let segment_index: usize = (t / self.segment_time).floor().as_();
        if segment_index == self.points_len_minus_1 {
            return self.points[self.points_len_minus_1];
        }
        let in_segment_t = t - AsPrimitive::<T>::as_(segment_index) * self.segment_time;
        linear_bezier(
            self.points[segment_index],
            self.points[segment_index + 1],
            in_segment_t / self.segment_time,
        )
    }
}

fn integrate<I, F>(segments: u32, x0: f64, xn: f64, function: F) -> ComplexValueF64
where
    F: Fn(f64) -> ComplexValueF64 + Copy + Sync + Send,
    I: Integrate,
{
    I::complex_integral_rayon(segments, x0, xn, function)
}
