// Copyright (c) Angus Johnson 2010-2025.
// BSL-1.0. Rust seed by MatterHackers.
// Derived from clipper2-rust 2f5e9d4249bc4d624773094ff6164335d9ff6ae7.
// See ../LICENSE-Boost-1.0 and ../SOURCE.md for pinned source authority.

use super::*;

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

/// Result of point-in-polygon test
/// Direct port from clipper.core.h line 1046
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum PointInPolygonResult {
    On,
    Inside,
    Outside,
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
