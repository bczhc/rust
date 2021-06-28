use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

pub type PointF64 = Point<f64>;
pub type PointI32 = Point<i32>;
pub type PointU32 = Point<u32>;

#[derive(PartialEq, Copy, Debug, Clone, Hash)]
pub struct Point<T> {
    pub x: T,
    pub y: T,
}

impl<T> Point<T> {
    #[inline]
    pub fn new(x: T, y: T) -> Point<T> {
        return Self { x, y };
    }
}

impl Point<f64> {
    pub fn set(&mut self, x: f64, y: f64) {
        self.x = x;
        self.y = y;
    }

    pub fn mid(a: &PointF64, b: &PointF64) -> PointF64 {
        Self {
            x: (a.x + b.x) / 2.0,
            y: (a.y + b.y) / 2.0,
        }
    }
}

impl Add<&Self> for Point<f64> {
    type Output = Self;

    #[inline]
    fn add(self, rhs: &Point<f64>) -> Self::Output {
        return Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        };
    }
}

impl Add for Point<f64> {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        return self.add(&rhs);
    }
}

impl AddAssign for Point<f64> {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.add_assign(&rhs);
    }
}

impl AddAssign<&Self> for Point<f64> {
    #[inline]
    fn add_assign(&mut self, rhs: &Point<f64>) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl Sub for Point<f64> {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        return self.sub(&rhs);
    }
}

impl Sub<&Self> for Point<f64> {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: &Point<f64>) -> Self::Output {
        return Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        };
    }
}

impl SubAssign for Point<f64> {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        self.sub_assign(&rhs);
    }
}

impl SubAssign<&Self> for Point<f64> {
    #[inline]
    fn sub_assign(&mut self, rhs: &Point<f64>) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl Mul for Point<f64> {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        return self.mul(&rhs);
    }
}

impl Mul<&Self> for Point<f64> {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: &Point<f64>) -> Self::Output {
        return Self {
            x: self.x * rhs.x - self.y * rhs.y,
            y: self.x * rhs.y + self.y * rhs.x,
        };
    }
}

impl Mul<f64> for PointF64 {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl MulAssign<Self> for Point<f64> {
    #[inline]
    fn mul_assign(&mut self, rhs: Point<f64>) {
        self.mul_assign(&rhs);
    }
}

impl MulAssign<&Self> for Point<f64> {
    #[inline]
    fn mul_assign(&mut self, rhs: &Point<f64>) {
        let re1 = self.x * rhs.x - self.y * rhs.y;
        let im1 = self.x * rhs.y + self.y * rhs.x;
        self.x = re1;
        self.y = im1;
    }
}

impl MulAssign<f64> for PointF64 {
    fn mul_assign(&mut self, rhs: f64) {
        self.x *= rhs;
        self.y *= rhs;
    }
}

impl Div for Point<f64> {
    type Output = Self;

    #[inline]
    fn div(self, rhs: Self) -> Self::Output {
        return self.div(&rhs);
    }
}

impl Div<&Self> for Point<f64> {
    type Output = Self;

    #[inline]
    fn div(self, rhs: &Point<f64>) -> Self::Output {
        let a = rhs.x.powi(2) + rhs.y.powi(2);
        return Self {
            x: (self.x * rhs.x + self.y * rhs.y) / a,
            y: (self.y * rhs.x - self.x * rhs.y) / a,
        };
    }
}

impl Div<f64> for PointF64 {
    type Output = Self;

    fn div(self, rhs: f64) -> Self::Output {
        Self {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

impl DivAssign for Point<f64> {
    #[inline]
    fn div_assign(&mut self, rhs: Self) {
        self.div_assign(&rhs);
    }
}

impl DivAssign<&Self> for Point<f64> {
    #[inline]
    fn div_assign(&mut self, rhs: &Point<f64>) {
        let a = rhs.x.powi(2) + rhs.y.powi(2);
        let re1 = (self.x * rhs.x + self.y * rhs.y) / a;
        let im1 = (self.y * rhs.x - self.x * rhs.y) / a;
        self.x = re1;
        self.y = im1;
    }
}

impl DivAssign<f64> for PointF64 {
    fn div_assign(&mut self, rhs: f64) {
        self.x /= rhs;
        self.y /= rhs;
    }
}
