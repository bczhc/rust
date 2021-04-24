use std::ops::{Add, Sub, Mul, Div, AddAssign, ShlAssign, SubAssign, MulAssign, DivAssign};

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
        return Self { re: self.re + rhs.re, im: self.im + rhs.im };
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
        return Self { re: self.re - rhs.re, im: self.im - rhs.im };
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
            im: self.re * self.im + self.im * rhs.re,
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

pub trait ComplexIntegral {
    fn get_definite_integral_by_trapezium<T>(x0: f64, xn: f64, integral_n: i32, func: T) -> ComplexValue<f64>
        where T: Fn(f64) -> ComplexValue<f64> {
        let d = (xn - x0) / integral_n as f64;
        let cv_d = ComplexValue::new(d, 0_f64);
        let cv_2 = ComplexValue::new(2_f64, 0_f64);
        let mut sum = ComplexValue::new(0_f64, 0_f64);
        let mut left = ComplexValue::new(0_f64, 0_f64);
        let mut right = ComplexValue::new(0_f64, 0_f64);
        let mut t = 0_f64;
        while t <= xn {
            left = func(t);
            right = func(t + d);
            sum += ((left + right).mul(&cv_d).div(&cv_2));
            t += d;
        }
        return sum;
    }
}

impl ComplexIntegral for ComplexValue<f64> {}