use num_complex::Complex;
use num_traits::{Float, NumAssign};
use rayon::prelude::{IntoParallelIterator, ParallelIterator};

pub fn complex_integral<F, T>(segments: u32, x0: T, xn: T, function: F) -> Complex<T>
where
    F: Fn(T) -> Complex<T>,
    T: Float + NumAssign,
{
    // TODO: lossless casting (avoid `unwrap`)
    let d = (xn - x0) / T::from(segments).unwrap();
    let mut i = x0;
    let mut i2: T;
    let mut sum = Complex::new(T::zero(), T::zero());
    let c2 = Complex::new(T::from(2.0).unwrap(), T::from(2.0).unwrap());
    let c_d = Complex::new(d, T::zero());
    while i < xn {
        i2 = i + d;
        sum += (function(i) + function(i2)) * c_d / c2;
        i = i2;
    }
    sum
}

pub fn complex_integral_rayon<F, T>(segments: u32, x0: T, xn: T, function: F) -> Complex<T>
where
    F: Fn(T) -> Complex<T> + Copy + Sync + Send,
    T: Float + NumAssign + Send + Sync,
{
    let range = xn - x0;
    let d = range / T::from(segments).unwrap();
    let c2 = Complex::new(T::from(2.0).unwrap(), T::from(2.0).unwrap());
    (0..segments)
        .into_par_iter()
        .map(|x| T::from(x).unwrap())
        .map(move |x| (function(x * d) + function((x + T::one()) * d)) * d / c2)
        .sum()
}
