// Copyright (c) Angus Johnson 2010-2025.
// BSL-1.0. Rust seed by MatterHackers.
// Derived from clipper2-rust 2f5e9d4249bc4d624773094ff6164335d9ff6ae7.
// See ../LICENSE-Boost-1.0 and ../SOURCE.md for pinned source authority.

use super::*;

impl<T> Rect<T>
where
    T: Num + Copy + PartialOrd,
{
    /// Get midpoint of rectangle
    /// Direct port from clipper.core.h line 336
    pub fn mid_point(&self) -> Point<T> {
        Point {
            x: (self.left + self.right) / (T::one() + T::one()),
            y: (self.top + self.bottom) / (T::one() + T::one()),

            z: 0,
        }
    }
}

impl<T> Rect<T>
where
    T: Num + Copy + PartialOrd,
{
    /// Convert rectangle to path (clockwise from top-left)
    /// Direct port from clipper.core.h line 341
    pub fn as_path(&self) -> Path<T> {
        vec![
            Point::new(self.left, self.top),
            Point::new(self.right, self.top),
            Point::new(self.right, self.bottom),
            Point::new(self.left, self.bottom),
        ]
    }
}

impl<T> Rect<T>
where
    T: Num + Copy + PartialOrd,
{
    /// Check if another rectangle is fully contained within this rectangle
    /// Direct port from clipper.core.h line 357
    pub fn contains_rect(&self, rec: &Rect<T>) -> bool {
        rec.left >= self.left
            && rec.right <= self.right
            && rec.top >= self.top
            && rec.bottom <= self.bottom
    }
}

impl<T> Rect<T>
where
    T: Num + Copy + PartialOrd,
{
    /// Check if rectangle is empty
    pub fn is_empty(&self) -> bool {
        self.left >= self.right || self.top >= self.bottom
    }
}

impl<T> PartialEq for Rect<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.left == other.left
            && self.right == other.right
            && self.top == other.top
            && self.bottom == other.bottom
    }
}

impl<T> std::ops::AddAssign for Rect<T>
where
    T: Num + Copy + PartialOrd,
{
    fn add_assign(&mut self, other: Self) {
        self.left = if self.left < other.left {
            self.left
        } else {
            other.left
        };
        self.top = if self.top < other.top {
            self.top
        } else {
            other.top
        };
        self.right = if self.right > other.right {
            self.right
        } else {
            other.right
        };
        self.bottom = if self.bottom > other.bottom {
            self.bottom
        } else {
            other.bottom
        };
    }
}

impl<T> std::ops::Add for Rect<T>
where
    T: Num + Copy + PartialOrd,
{
    type Output = Self;

    fn add(self, other: Self) -> Self {
        let mut result = self;
        result += other;
        result
    }
}

pub type Point64 = Point<i64>;

pub type PointD = Point<f64>;

pub type Rect64 = Rect<i64>;

/// Vector of points forming a path
pub type Path<T> = Vec<Point<T>>;

pub type Path64 = Path<i64>;

pub type PathD = Path<f64>;

/// Vector of paths
pub type Paths<T> = Vec<Path<T>>;

pub type Paths64 = Paths<i64>;
