use crate::complex_num::ComplexValue;

/// Struct for an epicycle in Fourier series
///
/// Definition: a<sub>n</sub> = `a`e<sup>i`w`t</sup>
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Epicycle {
    pub n: i32,
    pub a: ComplexValue<f64>,
    pub w: f64,
}
