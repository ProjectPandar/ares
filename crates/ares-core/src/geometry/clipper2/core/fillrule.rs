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

/// Exception type for Clipper2 errors
/// Direct port from clipper.core.h line 27
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Clipper2Exception {
    pub(crate) description: String,
}

impl Clipper2Exception {
    pub fn new(description: &str) -> Self {
        Self {
            description: description.to_string(),
        }
    }
}

impl Clipper2Exception {
    pub fn description(&self) -> &str {
        &self.description
    }
}

impl Display for Clipper2Exception {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Clipper2Exception: {}", self.description)
    }
}

impl std::error::Error for Clipper2Exception {}

/// Handle errors by throwing appropriate exceptions
/// Direct port from clipper.core.h line 73
pub fn do_error(error_code: i32) -> Result<(), Clipper2Exception> {
    use errors::*;

    match error_code {
        PRECISION_ERROR_I => Err(Clipper2Exception::new(PRECISION_ERROR)),
        SCALE_ERROR_I => Err(Clipper2Exception::new(SCALE_ERROR)),
        NON_PAIR_ERROR_I => Err(Clipper2Exception::new(NON_PAIR_ERROR)),
        UNDEFINED_ERROR_I => Err(Clipper2Exception::new(UNDEFINED_ERROR)),
        RANGE_ERROR_I => Err(Clipper2Exception::new(RANGE_ERROR)),
        _ => Err(Clipper2Exception::new("Unknown error")),
    }
}

/// Constants matching C++ implementation
/// Direct port from clipper.core.h line 55-71
pub mod constants {
    /// PI constant
    pub const PI: f64 = std::f64::consts::PI;

    /// Maximum decimal precision for clipper operations
    pub const CLIPPER2_MAX_DEC_PRECISION: i32 = 8;

    /// Maximum coordinate value (INT64_MAX >> 2)
    pub const MAX_COORD: i64 = i64::MAX >> 2;
    /// Minimum coordinate value
    pub const MIN_COORD: i64 = -MAX_COORD;
    /// Invalid coordinate sentinel
    pub const INVALID: i64 = i64::MAX;
    /// Maximum coordinate as double
    pub const MAX_COORD_D: f64 = MAX_COORD as f64;
    /// Minimum coordinate as double
    pub const MIN_COORD_D: f64 = MIN_COORD as f64;
    /// Maximum double value
    pub const MAX_DBL: f64 = f64::MAX;
}

/// Error constants matching C++ implementation
pub mod errors {
    /// Precision exceeds the permitted range
    pub const PRECISION_ERROR: &str = "Precision exceeds the permitted range";
    /// Values exceed permitted range
    pub const RANGE_ERROR: &str = "Values exceed permitted range";
    /// Invalid scale (either 0 or too large)
    pub const SCALE_ERROR: &str = "Invalid scale (either 0 or too large)";
    /// There must be 2 values for each coordinate
    pub const NON_PAIR_ERROR: &str = "There must be 2 values for each coordinate";
    /// There is an undefined error in Clipper2
    pub const UNDEFINED_ERROR: &str = "There is an undefined error in Clipper2";

    /// Error codes (2^n) - non-fatal
    pub const PRECISION_ERROR_I: i32 = 1;
    /// Error codes (2^n) - non-fatal
    pub const SCALE_ERROR_I: i32 = 2;
    /// Error codes (2^n) - non-fatal
    pub const NON_PAIR_ERROR_I: i32 = 4;
    /// Error codes (2^n) - fatal
    pub const UNDEFINED_ERROR_I: i32 = 32;
    /// Error codes (2^n)
    pub const RANGE_ERROR_I: i32 = 64;
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
    /// Create a zero point
    pub fn zero() -> Self
    where
        T: Zero,
    {
        Self {
            x: T::zero(),
            y: T::zero(),

            z: ZType::zero(),
        }
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

impl<T> Point<T>
where
    T: Num + Copy + PartialOrd,
{
    /// Scale a point by a floating-point factor
    pub fn scale<F>(self, scale: F) -> Point<F>
    where
        F: Float,
        T: Into<F>,
    {
        Point {
            x: self.x.into() * scale,
            y: self.y.into() * scale,

            z: self.z,
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
