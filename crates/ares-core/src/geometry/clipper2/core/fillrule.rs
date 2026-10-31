// Copyright (c) Angus Johnson 2010-2025.
// BSL-1.0. Rust seed by MatterHackers.
// Derived from clipper2-rust 2f5e9d4249bc4d624773094ff6164335d9ff6ae7.
// See ../LICENSE-Boost-1.0 and ../SOURCE.md for pinned source authority.

use super::*;

/// Fill rule determines how polygons with self-intersections are filled
/// Direct port from clipper.core.h line 108
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(C)]
pub enum FillRule {
    /// Even-odd fill rule (also known as Alternate)
    #[default]
    EvenOdd,
    /// Non-zero fill rule (also known as Winding)
    NonZero,
    /// Positive fill rule
    Positive,
    /// Negative fill rule
    Negative,
}

/// Constants matching C++ implementation
/// Direct port from clipper.core.h line 55-71
pub mod constants {
    /// PI constant
    pub const PI: f64 = std::f64::consts::PI;

    /// Maximum coordinate value (INT64_MAX >> 2)
    pub const MAX_COORD: i64 = i64::MAX >> 2;
}

/// Z coordinate type when the `using_z` feature is enabled
/// Direct port from clipper.core.h line 111 (z_type)
pub type ZType = i64;

/// 2D point with generic numeric type
/// Direct port from clipper.core.h line 117
#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
pub struct Point<T> {
    pub x: T,
    pub y: T,

    pub z: ZType,
}

impl<T> PartialEq for Point<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}

impl<T> Point<T>
where
    T: Num + Copy,
{
    /// Create a new point
    pub fn new(x: T, y: T) -> Self {
        Self { x, y, z: 0 }
    }
}

impl<T> Point<T>
where
    T: Num + Copy,
{
    /// Create a new point with an explicit z value
    /// Direct port from clipper.core.h Point(x, y, z) constructor
    pub fn new_z(x: T, y: T, z: ZType) -> Self {
        Self { x, y, z }
    }
}

impl<T> Point<T>
where
    T: Num + Copy,
{
    /// Add two points
    // z is zeroed (not propagated): C++ operator+ builds the result with
    // Point(x, y), which default-initializes z. Only scaling preserves z.
    pub fn add_point(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,

            z: 0,
        }
    }
}

impl<T> Point<T>
where
    T: Num + Copy,
{
    /// Subtract two points
    pub fn sub_point(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,

            z: 0,
        }
    }
}

impl<T> Point<T>
where
    T: Num + Copy,
{
    /// Negate a point
    pub fn negate(self) -> Self {
        Self {
            x: T::zero() - self.x,
            y: T::zero() - self.y,

            z: 0,
        }
    }
}

impl<T> std::ops::Add for Point<T>
where
    T: Num + Copy,
{
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        self.add_point(rhs)
    }
}

impl<T> std::ops::Sub for Point<T>
where
    T: Num + Copy,
{
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        self.sub_point(rhs)
    }
}

impl<T> std::ops::Neg for Point<T>
where
    T: Num + Copy,
{
    type Output = Self;

    fn neg(self) -> Self::Output {
        self.negate()
    }
}

/// Rectangle with generic numeric type
/// Direct port from clipper.core.h line 295
#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
pub struct Rect<T> {
    pub left: T,
    pub top: T,
    pub right: T,
    pub bottom: T,
}

impl<T> Rect<T>
where
    T: Num + Copy + PartialOrd,
{
    /// Create a new rectangle
    pub fn new(left: T, top: T, right: T, bottom: T) -> Self {
        Self {
            left,
            top,
            right,
            bottom,
        }
    }
}
