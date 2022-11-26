use num_complex::Complex;
use num_traits::Float;

/// Struct for an epicycle in Fourier series
/// -----
///
/// ## Definition
/// E<sub>n</sub> = `a`e<sup>i`p`t</sup>
/// - E means "epicycle"
/// - p = n&omega;
///
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Epicycle<T>
where
    T: Float,
{
    pub n: i32,
    pub a: Complex<T>,
    pub p: T,
}
