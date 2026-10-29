//! Portable outward intervals for the CGAL Interval_nt filtered predicate stage.
use super::Scalar;
use std::ops::{Add, Mul, Neg, Sub};

#[derive(Clone, Copy)]
pub(super) struct Interval {
    low: f64,
    high: f64,
}
impl Interval {
    fn bounds(low: f64, high: f64) -> Self {
        Self {
            low: low.next_down(),
            high: high.next_up(),
        }
    }
    pub(super) fn sign(self) -> Option<i8> {
        if self.low > 0.0 {
            Some(1)
        } else if self.high < 0.0 {
            Some(-1)
        } else {
            None
        }
    }
}
impl Scalar for Interval {
    fn from_double(value: f64) -> Self {
        Self {
            low: value,
            high: value,
        }
    }
}
impl Neg for Interval {
    type Output = Self;
    fn neg(self) -> Self {
        Self {
            low: -self.high,
            high: -self.low,
        }
    }
}
impl Add for Interval {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self::bounds(self.low + rhs.low, self.high + rhs.high)
    }
}
impl Sub for Interval {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self::bounds(self.low - rhs.high, self.high - rhs.low)
    }
}
impl Mul for Interval {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        let products = [
            self.low * rhs.low,
            self.low * rhs.high,
            self.high * rhs.low,
            self.high * rhs.high,
        ];
        if products.iter().any(|v| v.is_nan()) {
            return Self {
                low: f64::NEG_INFINITY,
                high: f64::INFINITY,
            };
        }
        Self::bounds(
            products.into_iter().fold(f64::INFINITY, f64::min),
            products.into_iter().fold(f64::NEG_INFINITY, f64::max),
        )
    }
}
