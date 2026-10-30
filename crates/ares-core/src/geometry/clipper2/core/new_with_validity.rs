// Copyright (c) Angus Johnson 2010-2025.
// BSL-1.0. Rust seed by MatterHackers.
// Derived from clipper2-rust 2f5e9d4249bc4d624773094ff6164335d9ff6ae7.
// See ../LICENSE-Boost-1.0 and ../SOURCE.md for pinned source authority.

use super::*;

impl<T> Rect<T>
where
    T: Num + Copy + PartialOrd,
{
    /// Create a rectangle, valid by default or invalid if specified
    /// Direct port from clipper.core.h line 307
    pub fn new_with_validity(is_valid: bool) -> Self
    where
        T: num_traits::Bounded,
    {
        if is_valid {
            Self {
                left: T::zero(),
                top: T::zero(),
                right: T::zero(),
                bottom: T::zero(),
            }
        } else {
            Self {
                left: T::max_value(),
                top: T::max_value(),
                right: T::min_value(),
                bottom: T::min_value(),
            }
        }
    }
}

impl<T> Rect<T>
where
    T: Num + Copy + PartialOrd,
{
    /// Create an invalid rectangle
    /// Direct port from clipper.core.h line 320
    pub fn invalid() -> Self
    where
        T: num_traits::Bounded,
    {
        Self {
            left: T::max_value(),
            top: T::max_value(),
            right: T::min_value(),
            bottom: T::min_value(),
        }
    }
}

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
    /// Check if point is contained within rectangle (exclusive bounds)
    /// Direct port from clipper.core.h line 352
    pub fn contains_point(&self, pt: &Point<T>) -> bool {
        pt.x > self.left && pt.x < self.right && pt.y > self.top && pt.y < self.bottom
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
    /// Check if this rectangle intersects with another
    /// Direct port from clipper.core.h line 372
    pub fn intersects(&self, rec: &Rect<T>) -> bool {
        let max_left = if self.left > rec.left {
            self.left
        } else {
            rec.left
        };
        let min_right = if self.right < rec.right {
            self.right
        } else {
            rec.right
        };
        let max_top = if self.top > rec.top {
            self.top
        } else {
            rec.top
        };
        let min_bottom = if self.bottom < rec.bottom {
            self.bottom
        } else {
            rec.bottom
        };

        max_left <= min_right && max_top <= min_bottom
    }
}

impl<T> Rect<T>
where
    T: Num + Copy + PartialOrd,
{
    /// Check if rectangle is valid (not using max sentinel values)
    /// Direct port from clipper.core.h line 329
    pub fn is_valid(&self) -> bool
    where
        T: num_traits::Bounded + PartialEq,
    {
        self.left != T::max_value()
    }
}

impl<T> Rect<T>
where
    T: Num + Copy + PartialOrd,
{
    /// Get width of rectangle
    pub fn width(&self) -> T {
        self.right - self.left
    }
}

impl<T> Rect<T>
where
    T: Num + Copy + PartialOrd,
{
    /// Get height of rectangle
    pub fn height(&self) -> T {
        self.bottom - self.top
    }
}

impl<T> Rect<T>
where
    T: Num + Copy + PartialOrd,
{
    /// Set width, adjusting right edge
    pub fn set_width(&mut self, width: T) {
        self.right = self.left + width;
    }
}

impl<T> Rect<T>
where
    T: Num + Copy + PartialOrd,
{
    /// Set height, adjusting bottom edge
    pub fn set_height(&mut self, height: T) {
        self.bottom = self.top + height;
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

impl<T> Rect<T>
where
    T: Float + Copy,
{
    /// Scale rectangle by floating-point factor
    pub fn scale(&mut self, scale: T) {
        self.left = self.left * scale;
        self.top = self.top * scale;
        self.right = self.right * scale;
        self.bottom = self.bottom * scale;
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

pub type RectD = Rect<f64>;

/// Vector of points forming a path
pub type Path<T> = Vec<Point<T>>;

pub type Path64 = Path<i64>;

pub type PathD = Path<f64>;

/// Vector of paths
pub type Paths<T> = Vec<Path<T>>;

pub type Paths64 = Paths<i64>;

pub type PathsD = Paths<f64>;

/// Invalid point constants
// z is 0, not MAX: C++ constructs these via Point(x, y), which
// default-initializes z (and equality ignores z regardless).
pub const INVALID_POINT64: Point64 = Point64 {
    x: i64::MAX,
    y: i64::MAX,

    z: 0,
};

pub const INVALID_POINTD: PointD = PointD {
    x: f64::MAX,
    y: f64::MAX,

    z: 0,
};
