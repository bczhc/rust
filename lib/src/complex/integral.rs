use num_complex::Complex;
use num_traits::{Float, NumAssign};
use rayon::prelude::{IntoParallelIterator, ParallelIterator};

pub trait Integrate {
    fn complex_integral_rayon<F, T>(segments: u32, x0: T, xn: T, function: F) -> Complex<T>
    where
        F: Fn(T) -> Complex<T> + Copy + Sync + Send,
        T: Float + NumAssign + NumAssign + Send + Sync;
}

pub struct Trapezoid;
pub struct LeftRectangle;
pub struct RightRectangle;
pub struct Simpson;
pub struct Simpson38;
pub struct Boole;

impl Integrate for Trapezoid {
    fn complex_integral_rayon<F, T>(segments: u32, x0: T, xn: T, function: F) -> Complex<T>
    where
        F: Fn(T) -> Complex<T> + Copy + Sync + Send,
        T: Float + NumAssign + NumAssign + Send + Sync,
    {
        let range = xn - x0;
        let d = range / T::from(segments).unwrap();
        let c2 = Complex::new(T::from(2).unwrap(), T::from(2).unwrap());
        (0..segments)
            .into_par_iter()
            .map(|x| T::from(x).unwrap())
            .map(move |x| (function(x * d) + function((x + T::one()) * d)) * d / c2)
            .sum()
    }
}

impl Integrate for LeftRectangle {
    fn complex_integral_rayon<F, T>(segments: u32, x0: T, xn: T, function: F) -> Complex<T>
    where
        F: Fn(T) -> Complex<T> + Copy + Sync + Send,
        T: Float + NumAssign + NumAssign + Send + Sync,
    {
        let range = xn - x0;
        let d = range / T::from(segments).unwrap();

        (0..segments)
            .into_par_iter()
            .map(|x| T::from(x).unwrap())
            .map(move |x| function(x * d) * d)
            .sum()
    }
}

impl Integrate for RightRectangle {
    fn complex_integral_rayon<F, T>(segments: u32, x0: T, xn: T, function: F) -> Complex<T>
    where
        F: Fn(T) -> Complex<T> + Copy + Sync + Send,
        T: Float + NumAssign + NumAssign + Send + Sync,
    {
        let range = xn - x0;
        let d = range / T::from(segments).unwrap();

        (1..=segments)
            .into_par_iter()
            .map(|x| T::from(x).unwrap())
            .map(move |x| function(x * d) * d)
            .sum()
    }
}

impl Integrate for Simpson {
    fn complex_integral_rayon<F, T>(segments: u32, x0: T, xn: T, function: F) -> Complex<T>
    where
        F: Fn(T) -> Complex<T> + Copy + Sync + Send,
        T: Float + NumAssign + NumAssign + Send + Sync,
    {
        let range = xn - x0;
        let d = range / T::from(segments).unwrap();
        let half_one = T::from(0.5).unwrap();
        let multiplier = Complex::new(d / T::from(6).unwrap(), T::zero());
        let four = Complex::new(T::from(4).unwrap(), T::zero());

        (0..segments)
            .into_par_iter()
            .map(|x| T::from(x).unwrap())
            .map(move |n| {
                multiplier
                    * (function(n * d)
                        + function((n + half_one) * d) * four
                        + function((n + T::one()) * d))
            })
            .sum()
    }
}

impl Integrate for Simpson38 {
    fn complex_integral_rayon<F, T>(segments: u32, x0: T, xn: T, function: F) -> Complex<T>
    where
        F: Fn(T) -> Complex<T> + Copy + Sync + Send,
        T: Float + NumAssign + NumAssign + Send + Sync,
    {
        let range = xn - x0;
        let d = range / T::from(segments).unwrap();
        let step2th = T::one() / T::from(3).unwrap();
        let step3th = T::from(2).unwrap() / T::from(3).unwrap();
        let multiplier = Complex::new(d / T::from(8).unwrap(), T::zero());
        let three = Complex::new(T::from(3).unwrap(), T::zero());

        (0..segments)
            .into_par_iter()
            .map(|x| T::from(x).unwrap())
            .map(move |n| {
                multiplier
                    * (function(n * d)
                        + function((n + step2th) * d) * three
                        + function((n + step3th) * d) * three)
                    + function((n + T::one()) * d)
            })
            .sum()
    }
}

impl Integrate for Boole {
    fn complex_integral_rayon<F, T>(segments: u32, x0: T, xn: T, function: F) -> Complex<T>
    where
        F: Fn(T) -> Complex<T> + Copy + Sync + Send,
        T: Float + NumAssign + NumAssign + Send + Sync,
    {
        let range = xn - x0;
        let d = range / T::from(segments).unwrap();
        let step2th = T::one() / T::from(4).unwrap();
        let step3th = step2th * T::from(2).unwrap();
        let step4th = step2th * T::from(3).unwrap();
        let multiplier = Complex::new(d / T::from(90).unwrap(), T::zero());
        let co_7 = Complex::new(T::from(7).unwrap(), T::zero());
        let co_12 = Complex::new(T::from(12).unwrap(), T::zero());
        let co_32 = Complex::new(T::from(32).unwrap(), T::zero());

        (0..segments)
            .into_par_iter()
            .map(|x| T::from(x).unwrap())
            .map(move |n| {
                multiplier
                    * (function(n * d) * co_7
                        + function((n + step2th) * d) * co_32
                        + function((n + step3th) * d) * co_12
                        + function((n + step4th) * d) * co_32
                        + function((n + T::one()) * d) * co_7)
                    + function((n + T::one()) * d)
            })
            .sum()
    }
}
