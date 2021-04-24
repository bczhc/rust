use crate::complex_num::ComplexValue;

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Epicycle {
    pub n: f64,
    pub c: ComplexValue<f64>,
}