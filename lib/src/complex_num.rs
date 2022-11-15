use std::{ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign}, iter::Sum};

pub type ComplexValueF64 = ComplexValue<f64>;

#[derive(PartialEq, Copy, Debug, Clone, Hash)]
pub struct ComplexValue<T> {
    pub re: T,
    pub im: T,
}

impl<T> ComplexValue<T> {
    #[inline]
    pub fn new(re: T, im: T) -> ComplexValue<T> {
        Self { re, im }
    }
}

impl ComplexValueF64 {
    #[inline]
    pub fn module(&self) -> f64 {
        f64::sqrt(f64::powi(self.re, 2) + self.im.powi(2))
    }

    /// Definition: `r`e<sup>i&theta;</sup>
    pub fn from_exponent_form_with_r(r: f64, theta: f64) -> ComplexValueF64 {
        ComplexValueF64::from_exponent_form(theta) * r
    }

    /// Definition: e<sup>i&theta;</sup>
    pub fn from_exponent_form(theta: f64) -> ComplexValueF64 {
        ComplexValueF64::new(f64::cos(theta), f64::sin(theta))
    }

    pub fn set(&mut self, re: f64, im: f64) {
        self.re = re;
        self.im = im;
    }
}

impl Add<&Self> for ComplexValueF64 {
    type Output = Self;

    #[inline]
    fn add(self, rhs: &ComplexValueF64) -> Self::Output {
        Self {
            re: self.re + rhs.re,
            im: self.im + rhs.im,
        }
    }
}

impl Add for ComplexValueF64 {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        self.add(&rhs)
    }
}

impl AddAssign for ComplexValueF64 {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.add_assign(&rhs);
    }
}

impl AddAssign<&Self> for ComplexValueF64 {
    #[inline]
    fn add_assign(&mut self, rhs: &ComplexValueF64) {
        self.re += rhs.re;
        self.im += rhs.im;
    }
}

impl Sub for ComplexValueF64 {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        self.sub(&rhs)
    }
}

impl Sub<&Self> for ComplexValueF64 {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: &ComplexValueF64) -> Self::Output {
        Self {
            re: self.re - rhs.re,
            im: self.im - rhs.im,
        }
    }
}

impl SubAssign for ComplexValueF64 {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        self.sub_assign(&rhs);
    }
}

impl SubAssign<&Self> for ComplexValueF64 {
    #[inline]
    fn sub_assign(&mut self, rhs: &ComplexValueF64) {
        self.re -= rhs.re;
        self.im -= rhs.im;
    }
}

impl Mul for ComplexValueF64 {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        self.mul(&rhs)
    }
}

impl Mul<&Self> for ComplexValueF64 {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: &ComplexValueF64) -> Self::Output {
        Self {
            re: self.re * rhs.re - self.im * rhs.im,
            im: self.re * rhs.im + self.im * rhs.re,
        }
    }
}

impl Mul<f64> for ComplexValueF64 {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self::Output {
        Self {
            re: self.re * rhs,
            im: self.im * rhs,
        }
    }
}

impl MulAssign<Self> for ComplexValueF64 {
    #[inline]
    fn mul_assign(&mut self, rhs: ComplexValueF64) {
        self.mul_assign(&rhs);
    }
}

impl MulAssign<&Self> for ComplexValueF64 {
    #[inline]
    fn mul_assign(&mut self, rhs: &ComplexValueF64) {
        let re1 = self.re * rhs.re - self.im * rhs.im;
        let im1 = self.re * rhs.im + self.im * rhs.re;
        self.re = re1;
        self.im = im1;
    }
}

impl MulAssign<f64> for ComplexValueF64 {
    fn mul_assign(&mut self, rhs: f64) {
        self.re *= rhs;
        self.im *= rhs;
    }
}

impl Div for ComplexValueF64 {
    type Output = Self;

    #[inline]
    fn div(self, rhs: Self) -> Self::Output {
        self.div(&rhs)
    }
}

impl Div<f64> for ComplexValueF64 {
    type Output = Self;

    fn div(self, rhs: f64) -> Self::Output {
        Self {
            re: self.re / rhs,
            im: self.im / rhs,
        }
    }
}

impl Div<&Self> for ComplexValueF64 {
    type Output = Self;

    #[inline]
    fn div(self, rhs: &ComplexValueF64) -> Self::Output {
        let a = rhs.re.powi(2) + rhs.im.powi(2);
        Self {
            re: (self.re * rhs.re + self.im * rhs.im) / a,
            im: (self.im * rhs.re - self.re * rhs.im) / a,
        }
    }
}

impl DivAssign for ComplexValueF64 {
    #[inline]
    fn div_assign(&mut self, rhs: Self) {
        self.div_assign(&rhs);
    }
}

impl DivAssign<&Self> for ComplexValueF64 {
    #[inline]
    fn div_assign(&mut self, rhs: &ComplexValueF64) {
        let a = rhs.re.powi(2) + rhs.im.powi(2);
        let re1 = (self.re * rhs.re + self.im * rhs.im) / a;
        let im1 = (self.im * rhs.re - self.re * rhs.im) / a;
        self.re = re1;
        self.im = im1;
    }
}

impl DivAssign<f64> for ComplexValueF64 {
    fn div_assign(&mut self, rhs: f64) {
        self.re /= rhs;
        self.im /= rhs;
    }
}

impl<T> Default for ComplexValue<T> where T: Default {
    fn default() -> Self {
        Self { re: T::default(), im: T::default() }
    }
}

impl ComplexValueF64 {
    pub fn zero() -> Self {
        ComplexValueF64::default()
    }
}

impl Sum<ComplexValueF64> for ComplexValueF64 {
    fn sum<I: Iterator<Item = ComplexValueF64>>(iter: I) -> Self {
        iter.fold(ComplexValueF64::zero(), |a, b| a + b)
    }
}

pub mod complex_integral {
    use rayon::prelude::{IntoParallelIterator, ParallelIterator};

    use crate::complex_num::ComplexValueF64;

    pub fn complex_integral<F>(segments: u32, x0: f64, xn: f64, function: F) -> ComplexValueF64
    where
        F: Fn(f64) -> ComplexValueF64,
    {
        let d = (xn - x0) / segments as f64;
        let mut i = x0;
        let mut i2: f64;
        let mut sum = ComplexValueF64::new(0.0, 0.0);
        let c2 = ComplexValueF64::new(2.0, 0.0);
        let c_d = ComplexValueF64::new(d, 0.0);
        while i < xn {
            i2 = i + d;
            sum += (function(i) + function(i2)) * c_d / c2;
            i = i2;
        }
        sum
    }

    pub fn complex_integral_rayon<F>(
        segments: u32,
        x0: f64,
        xn: f64,
        function: F,
    ) -> ComplexValueF64
    where
        F: Fn(f64) -> ComplexValueF64 + Copy + Sync + Send,
    {
        let range = xn - x0;
        let d = range / segments as f64;

        (0..segments).into_par_iter()
        .map(move |x| {
            (function(x as f64 * d) + function((x + 1) as f64 * d)) * d / 2.0
        }).sum::<ComplexValueF64>()
    }
}
