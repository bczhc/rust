use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

pub type ComplexValueF64 = ComplexValue<f64>;

#[derive(PartialEq, Copy, Debug, Clone, Hash)]
pub struct ComplexValue<T> {
    pub re: T,
    pub im: T,
}

impl<T> ComplexValue<T> {
    #[inline]
    pub fn new(re: T, im: T) -> ComplexValue<T> {
        return Self { re, im };
    }
}

impl ComplexValue<f64> {
    #[inline]
    pub fn module(&self) -> f64 {
        return f64::sqrt(f64::powi(self.re, 2) + self.im.powi(2));
    }

    pub fn set(&mut self, re: f64, im: f64) {
        self.re = re;
        self.im = im;
    }
}

impl Add<&Self> for ComplexValue<f64> {
    type Output = Self;

    #[inline]
    fn add(self, rhs: &ComplexValue<f64>) -> Self::Output {
        return Self {
            re: self.re + rhs.re,
            im: self.im + rhs.im,
        };
    }
}

impl Add for ComplexValue<f64> {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        return self.add(&rhs);
    }
}

impl AddAssign for ComplexValue<f64> {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.add_assign(&rhs);
    }
}

impl AddAssign<&Self> for ComplexValue<f64> {
    #[inline]
    fn add_assign(&mut self, rhs: &ComplexValue<f64>) {
        self.re += rhs.re;
        self.im += rhs.im;
    }
}

impl Sub for ComplexValue<f64> {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        return self.sub(&rhs);
    }
}

impl Sub<&Self> for ComplexValue<f64> {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: &ComplexValue<f64>) -> Self::Output {
        return Self {
            re: self.re - rhs.re,
            im: self.im - rhs.im,
        };
    }
}

impl SubAssign for ComplexValue<f64> {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        self.sub_assign(&rhs);
    }
}

impl SubAssign<&Self> for ComplexValue<f64> {
    #[inline]
    fn sub_assign(&mut self, rhs: &ComplexValue<f64>) {
        self.re -= rhs.re;
        self.im -= rhs.im;
    }
}

impl Mul for ComplexValue<f64> {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        return self.mul(&rhs);
    }
}

impl Mul<&Self> for ComplexValue<f64> {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: &ComplexValue<f64>) -> Self::Output {
        return Self {
            re: self.re * rhs.re - self.im * rhs.im,
            im: self.re * rhs.im + self.im * rhs.re,
        };
    }
}

impl MulAssign<Self> for ComplexValue<f64> {
    #[inline]
    fn mul_assign(&mut self, rhs: ComplexValue<f64>) {
        self.mul_assign(&rhs);
    }
}

impl MulAssign<&Self> for ComplexValue<f64> {
    #[inline]
    fn mul_assign(&mut self, rhs: &ComplexValue<f64>) {
        let re1 = self.re * rhs.re - self.im * rhs.im;
        let im1 = self.re * rhs.im + self.im * rhs.re;
        self.re = re1;
        self.im = im1;
    }
}

impl Div for ComplexValue<f64> {
    type Output = Self;

    #[inline]
    fn div(self, rhs: Self) -> Self::Output {
        return self.div(&rhs);
    }
}

impl Div<&Self> for ComplexValue<f64> {
    type Output = Self;

    #[inline]
    fn div(self, rhs: &ComplexValue<f64>) -> Self::Output {
        let a = rhs.re.powi(2) + rhs.im.powi(2);
        return Self {
            re: (self.re * rhs.re + self.im * rhs.im) / a,
            im: (self.im * rhs.re - self.re * rhs.im) / a,
        };
    }
}

impl DivAssign for ComplexValue<f64> {
    #[inline]
    fn div_assign(&mut self, rhs: Self) {
        self.div_assign(&rhs);
    }
}

impl DivAssign<&Self> for ComplexValue<f64> {
    #[inline]
    fn div_assign(&mut self, rhs: &ComplexValue<f64>) {
        let a = rhs.re.powi(2) + rhs.im.powi(2);
        let re1 = (self.re * rhs.re + self.im * rhs.im) / a;
        let im1 = (self.im * rhs.re - self.re * rhs.im) / a;
        self.re = re1;
        self.im = im1;
    }
}
