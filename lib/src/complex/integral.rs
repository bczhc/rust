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
        let c2 = Complex::new(T::from(2).unwrap(), T::zero());
        (0..segments)
            .into_par_iter()
            .map(|x| T::from(x).unwrap())
            .map(move |x| (function(x0 + x * d) + function(x0 + (x + T::one()) * d)) * d / c2)
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
            .map(move |x| function(x0 + x * d) * d)
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
            .map(move |x| function(x0 + x * d) * d)
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
                    * (function(x0 + n * d)
                        + function(x0 + (n + half_one) * d) * four
                        + function(x0 + (n + T::one()) * d))
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
                    * (function(x0 + n * d)
                        + function(x0 + (n + step2th) * d) * three
                        + function(x0 + (n + step3th) * d) * three
                        + function(x0 + (n + T::one()) * d))
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
                    * (function(x0 + n * d) * co_7
                        + function(x0 + (n + step2th) * d) * co_32
                        + function(x0 + (n + step3th) * d) * co_12
                        + function(x0 + (n + step4th) * d) * co_32
                        + function(x0 + (n + T::one()) * d) * co_7)
            })
            .sum()
    }
}

#[cfg(test)]
mod test {
    use std::f64::consts::PI;

    use super::*;
    use float_cmp::assert_approx_eq;
    use num_complex::Complex64;
    use num_traits::Zero;

    #[test]
    pub fn integrate1() {
        test_integral(
            |t| Complex64::from_polar(1.0, t),
            0.0,
            2.0 * PI,
            10000,
            Complex64::zero(),
            0.000000000000001,
        )
    }

    #[test]
    pub fn integral2() {
        let function = |t: f64| {
            let re = 2.0 / (-f64::sqrt(t) + f64::powf(t, 3.0) - f64::sin(t));
            let im = f64::sin(f64::sqrt((1.0 + t) / (2.0 * f64::powf(t, 3.0))));
            Complex64::new(re, im)
        };

        test_integral(
            function,
            1.5,
            10.0,
            100000,
            Complex64::new(0.657_012_434_779_459_7, 1.503_664_889_479_836),
            0.0001,
        )
    }

    #[test]
    pub fn integral3() {
        let function =
            |t: f64| Complex64::new(f64::sqrt(t), f64::sin(2.0 * t) + f64::sqrt(t) / (t + 1.0));

        let expected = Complex64::new(
            4.0 / 3.0 * f64::sqrt(2.0) * f64::powf(PI, 3.0 / 2.0),
            2.0 * (f64::sqrt(2.0 * PI) - f64::atan(f64::sqrt(2.0 * PI))),
        );
        test_integral(function, 0.0, 2.0 * PI, 100000, expected, 0.0001)
    }

    fn test_integral<F>(
        function: F,
        from: f64,
        to: f64,
        segments: u32,
        expected: Complex64,
        epsilon: f64,
    ) where
        F: Fn(f64) -> Complex<f64> + Copy + Sync + Send,
    {
        fn check<I, F>(
            function: F,
            from: f64,
            to: f64,
            segments: u32,
            expected: Complex64,
            epsilon: f64,
        ) where
            F: Fn(f64) -> Complex<f64> + Copy + Sync + Send,
            I: Integrate,
        {
            let r = I::complex_integral_rayon(segments, from, to, function);
            assert_approx_eq!(f64, r.re, expected.re, epsilon = epsilon);
            assert_approx_eq!(f64, r.im, expected.im, epsilon = epsilon);
        }

        check::<Trapezoid, _>(function, from, to, segments, expected, epsilon);
        check::<LeftRectangle, _>(function, from, to, segments, expected, epsilon);
        check::<RightRectangle, _>(function, from, to, segments, expected, epsilon);
        check::<Simpson, _>(function, from, to, segments, expected, epsilon);
        check::<Simpson38, _>(function, from, to, segments, expected, epsilon);
        check::<Boole, _>(function, from, to, segments, expected, epsilon);
    }
}
