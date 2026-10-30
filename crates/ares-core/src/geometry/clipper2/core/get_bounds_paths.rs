// Copyright (c) Angus Johnson 2010-2025.
// BSL-1.0. Rust seed by MatterHackers.
// Derived from clipper2-rust 2f5e9d4249bc4d624773094ff6164335d9ff6ae7.
// See ../LICENSE-Boost-1.0 and ../SOURCE.md for pinned source authority.

use super::*;

/// Calculate the bounding rectangle of multiple paths
/// Direct port from clipper.core.h line 449
#[inline]
pub fn get_bounds_paths<T>(paths: &Paths<T>) -> Rect<T>
where
    T: Copy + PartialOrd + num_traits::Bounded + num_traits::Num,
{
    let mut xmin = T::max_value();
    let mut ymin = T::max_value();
    let mut xmax = T::min_value();
    let mut ymax = T::min_value();

    for path in paths {
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
    }

    Rect::new(xmin, ymin, xmax, ymax)
}

/// Calculate the bounding rectangle of a path with type conversion
/// Direct port from clipper.core.h line 467
#[inline]
pub fn get_bounds_path_convert<T, T2>(path: &Path<T2>) -> Rect<T>
where
    T: Copy + PartialOrd + num_traits::Bounded + num_traits::Num,
    T2: Copy + Into<T>,
{
    let mut xmin = T::max_value();
    let mut ymin = T::max_value();
    let mut xmax = T::min_value();
    let mut ymax = T::min_value();

    for p in path {
        let x: T = p.x.into();
        let y: T = p.y.into();
        if x < xmin {
            xmin = x;
        }
        if x > xmax {
            xmax = x;
        }
        if y < ymin {
            ymin = y;
        }
        if y > ymax {
            ymax = y;
        }
    }

    Rect::new(xmin, ymin, xmax, ymax)
}

/// Calculate the bounding rectangle of multiple paths with type conversion
/// Direct port from clipper.core.h line 484
#[inline]
pub fn get_bounds_paths_convert<T, T2>(paths: &Paths<T2>) -> Rect<T>
where
    T: Copy + PartialOrd + num_traits::Bounded + num_traits::Num,
    T2: Copy + Into<T>,
{
    let mut xmin = T::max_value();
    let mut ymin = T::max_value();
    let mut xmax = T::min_value();
    let mut ymax = T::min_value();

    for path in paths {
        for p in path {
            let x: T = p.x.into();
            let y: T = p.y.into();
            if x < xmin {
                xmin = x;
            }
            if x > xmax {
                xmax = x;
            }
            if y < ymin {
                ymin = y;
            }
            if y > ymax {
                ymax = y;
            }
        }
    }

    Rect::new(xmin, ymin, xmax, ymax)
}

/// Square a value (matches C++ template<typename T> inline double Sqr(T val))
/// Direct port from clipper.core.h line 611
#[inline]
pub fn sqr<T>(val: T) -> f64
where
    T: ToF64,
{
    let d = val.to_f64();
    d * d
}

/// `clipper.core.h:828–832`: coordinate differences precede floating conversion.
#[inline]
pub fn distance_sqr<T>(pt1: Point<T>, pt2: Point<T>) -> f64
where
    T: Num + Copy + ToF64,
{
    sqr(pt1.x - pt2.x) + sqr(pt1.y - pt2.y)
}

/// Calculate squared perpendicular distance from point to line
/// Direct port from clipper.core.h line 840
#[inline]
pub fn perpendicular_distance_from_line_sqr<T>(
    pt: Point<T>,
    line1: Point<T>,
    line2: Point<T>,
) -> f64
where
    T: Num + Copy + ToF64,
{
    // clipper.core.h:834–846.
    let a = (pt.x - line1.x).to_f64();
    let b = (pt.y - line1.y).to_f64();
    let c = (line2.x - line1.x).to_f64();
    let d = (line2.y - line1.y).to_f64();

    if c == 0.0 && d == 0.0 {
        return 0.0;
    }

    sqr(a * d - c * b) / (c * c + d * d)
}

/// `clipper.core.h:849–869`: retain the paired accumulation and cast positions.
#[inline]
pub fn area<T>(path: &Path<T>) -> f64
where
    T: Num + Copy + ToF64,
{
    let count = path.len();
    if count < 3 {
        return 0.0;
    }
    let mut area = 0.0;
    let mut previous = count - 1;
    let stop = if count & 1 == 0 { count } else { count - 1 };
    let mut index = 0;
    while index != stop {
        area += (path[previous].y + path[index].y).to_f64()
            * (path[previous].x - path[index].x).to_f64();
        previous = index + 1;
        area += (path[index].y + path[previous].y).to_f64()
            * (path[index].x - path[previous].x).to_f64();
        index += 2;
    }
    if count & 1 != 0 {
        area += (path[previous].y + path[index].y).to_f64()
            * (path[previous].x - path[index].x).to_f64();
    }
    area * 0.5
}

/// Calculate total area of multiple polygon paths
/// Direct port from clipper.core.h line 874
#[inline]
pub fn area_paths<T>(paths: &Paths<T>) -> f64
where
    T: Num + Copy + ToF64,
{
    let mut total_area = 0.0;
    for path in paths {
        total_area += area(path);
    }
    total_area
}

/// Test if a polygon has positive orientation (counterclockwise)
/// Direct port from clipper.core.h line 886
#[inline]
pub fn is_positive<T>(poly: &Path<T>) -> bool
where
    T: Num + Copy + ToF64,
{
    area(poly) >= 0.0
}

/// Get the location of a point relative to a rectangle
/// Returns false if the point is on the rectangle edge, true otherwise
/// Direct port from clipper.rectclip.cpp line 37
#[inline]
pub fn get_location(rec: &Rect64, pt: &Point64, loc: &mut Location) -> bool {
    if pt.x == rec.left && pt.y >= rec.top && pt.y <= rec.bottom {
        *loc = Location::Left;
        return false;
    } else if pt.x == rec.right && pt.y >= rec.top && pt.y <= rec.bottom {
        *loc = Location::Right;
        return false;
    } else if pt.y == rec.top && pt.x >= rec.left && pt.x <= rec.right {
        *loc = Location::Top;
        return false;
    } else if pt.y == rec.bottom && pt.x >= rec.left && pt.x <= rec.right {
        *loc = Location::Bottom;
        return false;
    } else if pt.x < rec.left {
        *loc = Location::Left;
    } else if pt.x > rec.right {
        *loc = Location::Right;
    } else if pt.y < rec.top {
        *loc = Location::Top;
    } else if pt.y > rec.bottom {
        *loc = Location::Bottom;
    } else {
        *loc = Location::Inside;
    }
    true
}

/// Check if a line segment between two points is horizontal
/// Returns true if both points have the same y-coordinate
/// Direct port from clipper.rectclip.cpp line 68
#[inline]
pub fn is_horizontal(pt1: &Point64, pt2: &Point64) -> bool {
    pt1.y == pt2.y
}

/// Result of point-in-polygon test
/// Direct port from clipper.core.h line 1046
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum PointInPolygonResult {
    IsOn,
    IsInside,
    IsOutside,
}

/// Calculate exact intersection point between two line segments for Point64
/// Direct port from clipper.core.h line 902 - simplified version for i64 coordinates
#[inline]
pub fn get_line_intersect_pt(
    ln1a: Point64,
    ln1b: Point64,
    ln2a: Point64,
    ln2b: Point64,
    ip: &mut Point64,
) -> bool {
    // clipper.core.h:956–979: subtract as i64, then convert to f64.
    let dx1 = (ln1b.x - ln1a.x) as f64;
    let dy1 = (ln1b.y - ln1a.y) as f64;
    let dx2 = (ln2b.x - ln2a.x) as f64;
    let dy2 = (ln2b.y - ln2a.y) as f64;

    let det = dy1 * dx2 - dy2 * dx1;
    if det == 0.0 {
        return false;
    }

    let t = ((ln1a.x - ln2a.x) as f64 * dy2 - (ln1a.y - ln2a.y) as f64 * dx2) / det;

    if t <= 0.0 {
        *ip = ln1a;
    } else if t >= 1.0 {
        *ip = ln1b;
    } else {
        // C++ uses static_cast<T> which truncates toward zero, not rounds
        ip.x = (ln1a.x as f64 + t * dx1) as i64;
        ip.y = (ln1a.y as f64 + t * dy1) as i64;

        {
            ip.z = 0;
        }
    }

    true
}

/// Calculate intersection point between two line segments for PointD (floating-point)
/// Direct port from clipper.core.h line 952 (floating-point specialization)
#[inline]
pub fn get_line_intersect_pt_d(
    ln1a: PointD,
    ln1b: PointD,
    ln2a: PointD,
    ln2b: PointD,
    ip: &mut PointD,
) -> bool {
    // https://en.wikipedia.org/wiki/Line%E2%80%93line_intersection
    let dx1 = ln1b.x - ln1a.x;
    let dy1 = ln1b.y - ln1a.y;
    let dx2 = ln2b.x - ln2a.x;
    let dy2 = ln2b.y - ln2a.y;

    let det = dy1 * dx2 - dy2 * dx1;
    if det == 0.0 {
        return false;
    }
    let t = ((ln1a.x - ln2a.x) * dy2 - (ln1a.y - ln2a.y) * dx2) / det;
    if t <= 0.0 {
        *ip = ln1a;
    } else if t >= 1.0 {
        *ip = ln1b;
    } else {
        ip.x = ln1a.x + t * dx1;
        ip.y = ln1a.y + t * dy1;
        // C++ zeroes z only for computed (interpolated) intersections; the
        // endpoint-copy branches above keep the endpoint's z

        {
            ip.z = 0;
        }
    }
    true
}
