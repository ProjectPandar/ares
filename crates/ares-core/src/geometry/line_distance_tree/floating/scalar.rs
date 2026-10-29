//! Native scalar precision for `AABBTreeLines`'s `Linef` and `ExtrusionLine`.

use std::ops::{Add, Div, Mul, Sub};

pub(crate) trait NativeFloat:
    Copy
    + PartialOrd
    + Add<Output = Self>
    + Sub<Output = Self>
    + Mul<Output = Self>
    + Div<Output = Self>
    + Into<f64>
{
    const ZERO: Self;
    const HALF: Self;

    fn from_f64(value: f64) -> Self;

    fn min(self, other: Self) -> Self {
        if other < self { other } else { self }
    }

    fn max(self, other: Self) -> Self {
        if self < other { other } else { self }
    }
}

impl NativeFloat for f32 {
    const ZERO: Self = 0.0;
    const HALF: Self = 0.5;

    fn from_f64(value: f64) -> Self {
        value as Self
    }
}

impl NativeFloat for f64 {
    const ZERO: Self = 0.0;
    const HALF: Self = 0.5;

    fn from_f64(value: f64) -> Self {
        value
    }
}
