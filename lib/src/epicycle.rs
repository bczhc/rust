use crate::complex_num::{ComplexValue, ComplexValueF64};

/// Struct for an epicycle in Fourier series
/// -----
///
/// ## Definition
/// E<sub>n</sub> = `a`e<sup>i`p`t</sup>
/// - E means "epicycle"
/// - p = n&omega;
///
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Epicycle {
    pub n: i32,
    pub a: ComplexValueF64,
    pub p: f64,
}
