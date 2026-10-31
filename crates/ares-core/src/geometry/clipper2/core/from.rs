// Copyright (c) Angus Johnson 2010-2025.
// BSL-1.0. Rust seed by MatterHackers.
// Derived from clipper2-rust 2f5e9d4249bc4d624773094ff6164335d9ff6ae7.
// See ../LICENSE-Boost-1.0 and ../SOURCE.md for pinned source authority.

use super::*;

/// Lossy Point64 -> PointD conversion (z preserved under `using_z`)
/// Direct port from clipper.core.h Point conversion constructor
impl From<&Point64> for PointD {
    fn from(value: &Point64) -> Self {
        Self {
            x: value.x as f64,
            y: value.y as f64,

            z: value.z,
        }
    }
}

/// Rounding PointD -> Point64 conversion (z preserved under `using_z`)
/// Direct port from clipper.core.h Point conversion constructor, whose
/// Init() rounds float sources when the target type is integral
impl From<&PointD> for Point64 {
    fn from(value: &PointD) -> Self {
        Self {
            x: value.x.round() as i64,
            y: value.y.round() as i64,

            z: value.z,
        }
    }
}

/// Helper trait for converting to f64 - matching C++ static_cast<double> behavior
pub trait ToF64 {
    fn to_f64(self) -> f64;
}

impl ToF64 for i64 {
    fn to_f64(self) -> f64 {
        self as f64
    }
}

impl ToF64 for i32 {
    fn to_f64(self) -> f64 {
        self as f64
    }
}

impl ToF64 for f64 {
    fn to_f64(self) -> f64 {
        self
    }
}

impl ToF64 for f32 {
    fn to_f64(self) -> f64 {
        self as f64
    }
}

/// `clipper.core.h:811–814`: multiply before converting the two products.
#[inline]
pub fn cross_product_two_vectors<T>(vec1: Point<T>, vec2: Point<T>) -> f64
where
    T: Num + Copy + ToF64,
{
    (vec1.y * vec2.x).to_f64() - (vec2.y * vec1.x).to_f64()
}

/// `clipper.core.h:816–820`: subtract in the coordinate type before conversion.
#[inline]
pub fn dot_product_three_points<T>(pt1: Point<T>, pt2: Point<T>, pt3: Point<T>) -> f64
where
    T: Num + Copy + ToF64,
{
    (pt2.x - pt1.x).to_f64() * (pt3.x - pt2.x).to_f64()
        + (pt2.y - pt1.y).to_f64() * (pt3.y - pt2.y).to_f64()
}

/// `clipper.core.h:822–826`: multiply before converting the two products.
#[inline]
pub fn dot_product_two_vectors<T>(vec1: Point<T>, vec2: Point<T>) -> f64
where
    T: Num + Copy + ToF64,
{
    (vec1.x * vec2.x).to_f64() + (vec1.y * vec2.y).to_f64()
}

/// `clipper.core.h:730–750`: exact products on native and WASM alike.
#[inline]
pub fn products_are_equal(a: i64, b: i64, c: i64, d: i64) -> bool {
    i128::from(a) * i128::from(b) == i128::from(c) * i128::from(d)
}

/// Strip duplicate consecutive points from a path
/// Direct port from clipper.core.h line 658
#[inline]
pub fn strip_duplicates_path<T>(path: &mut Path<T>, is_closed_path: bool)
where
    T: PartialEq + Clone,
{
    // Use stable dedup to remove consecutive duplicates
    path.dedup();

    // For closed paths, also remove duplicates between last and first points
    if is_closed_path {
        while path.len() > 1 && path.last() == path.first() {
            path.pop();
        }
    }
}

/// Calculate the bounding rectangle of a path
/// Direct port from clipper.core.h line 432
#[inline]
pub fn get_bounds_path<T>(path: &Path<T>) -> Rect<T>
where
    T: Copy + PartialOrd + num_traits::Bounded + num_traits::Num,
{
    let mut xmin = T::max_value();
    let mut ymin = T::max_value();
    let mut xmax = T::min_value();
    let mut ymax = T::min_value();

    for p in path {
        if p.x < xmin {
            xmin = p.x;
        }
        if p.x > xmax {
            xmax = p.x;
        }
        if p.y < ymin {
            ymin = p.y;
        }
        if p.y > ymax {
            ymax = p.y;
        }
    }

    Rect::new(xmin, ymin, xmax, ymax)
}
