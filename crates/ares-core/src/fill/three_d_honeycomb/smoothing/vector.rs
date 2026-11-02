use std::ops::{Add, Div, Mul, Sub};

#[derive(Clone, Copy)]
pub(super) struct V(pub(super) f64, pub(super) f64);

impl V {
    pub(super) fn dot(self, other: Self) -> f64 {
        self.0 * other.0 + self.1 * other.1
    }
    pub(super) fn cross(self, other: Self) -> f64 {
        self.0 * other.1 - self.1 * other.0
    }
    pub(super) fn squared_norm(self) -> f64 {
        self.dot(self)
    }
    pub(super) fn norm(self) -> f64 {
        self.squared_norm().sqrt()
    }
    pub(super) fn normalized(self) -> Self {
        self / self.norm()
    }
}
impl Add for V {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self(self.0 + rhs.0, self.1 + rhs.1)
    }
}
impl Sub for V {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self(self.0 - rhs.0, self.1 - rhs.1)
    }
}
impl Mul<f64> for V {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self {
        Self(self.0 * rhs, self.1 * rhs)
    }
}
impl Div<f64> for V {
    type Output = Self;
    fn div(self, rhs: f64) -> Self {
        Self(self.0 / rhs, self.1 / rhs)
    }
}
