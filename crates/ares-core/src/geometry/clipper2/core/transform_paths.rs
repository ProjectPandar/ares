// Copyright (c) Angus Johnson 2010-2025.
// BSL-1.0. Rust seed by MatterHackers.
// Derived from clipper2-rust 2f5e9d4249bc4d624773094ff6164335d9ff6ae7.
// See ../LICENSE-Boost-1.0 and ../SOURCE.md for pinned source authority.

use super::*;

/// Transform multiple paths between types (no scaling, just type conversion)
/// Direct port from clipper.core.h line 601
#[inline]
pub fn transform_paths<T1, T2>(paths: &Paths<T2>) -> Paths<T1>
where
    T1: Num + Copy + FromF64,
    T2: Copy + ToF64,
{
    let mut result = Vec::with_capacity(paths.len());
    for path in paths {
        result.push(transform_path(path));
    }
    result
}

/// Check if two points are near each other within a distance threshold
/// Direct port from clipper.core.h line 616
#[inline]
pub fn near_equal<T>(p1: &Point<T>, p2: &Point<T>, max_dist_sqrd: f64) -> bool
where
    T: Copy + ToF64,
{
    sqr(p1.x.to_f64() - p2.x.to_f64()) + sqr(p1.y.to_f64() - p2.y.to_f64()) < max_dist_sqrd
}

/// Strip near-equal consecutive points from a path
/// Direct port from clipper.core.h line 623
#[inline]
pub fn strip_near_equal<T>(path: &Path<T>, max_dist_sqrd: f64, is_closed_path: bool) -> Path<T>
where
    T: Copy + ToF64 + PartialEq + Num,
{
    if path.is_empty() {
        return Path::new();
    }

    let mut result = Vec::with_capacity(path.len());
    let first_pt = path[0];
    let mut last_pt = first_pt;
    result.push(first_pt);

    for pt in path.iter().skip(1) {
        if !near_equal(pt, &last_pt, max_dist_sqrd) {
            last_pt = *pt;
            result.push(last_pt);
        }
    }

    if !is_closed_path {
        return result;
    }

    while result.len() > 1 && near_equal(result.last().unwrap(), &first_pt, max_dist_sqrd) {
        result.pop();
    }

    result
}

/// Strip near-equal consecutive points from multiple paths
/// Direct port from clipper.core.h line 647
#[inline]
pub fn strip_near_equal_paths<T>(
    paths: &Paths<T>,
    max_dist_sqrd: f64,
    is_closed_path: bool,
) -> Paths<T>
where
    T: Copy + ToF64 + PartialEq + Num,
{
    let mut result = Vec::with_capacity(paths.len());
    for path in paths {
        result.push(strip_near_equal(path, max_dist_sqrd, is_closed_path));
    }
    result
}

/// Translate a point by dx, dy
/// Direct port from clipper.core.h line 975
#[inline]
pub fn translate_point<T>(pt: &Point<T>, dx: f64, dy: f64) -> Point<T>
where
    T: Copy + ToF64 + FromF64 + Num,
{
    Point {
        x: T::from_f64(pt.x.to_f64() + dx),
        y: T::from_f64(pt.y.to_f64() + dy),

        z: pt.z,
    }
}

/// Reflect a point about a pivot point
/// Direct port from clipper.core.h line 986
#[inline]
pub fn reflect_point<T>(pt: &Point<T>, pivot: &Point<T>) -> Point<T>
where
    T: Num + Copy,
{
    Point {
        x: pivot.x + (pivot.x - pt.x),
        y: pivot.y + (pivot.y - pt.y),

        z: pt.z,
    }
}

/// `clipper.engine.cpp:120–127`: inputs are finite engine slopes/displacements.
#[inline]
pub fn nearbyint_f64(x: f64) -> f64 {
    x.round_ties_even()
}

/// Get the sign of a value: -1, 0, or 1
/// Direct port from clipper.core.h line 996
#[inline]
pub fn get_sign<T>(val: &T) -> i32
where
    T: Num + PartialOrd,
{
    if val.is_zero() {
        0
    } else if *val > T::zero() {
        1
    } else {
        -1
    }
}

/// Get the sign of the cross product of vectors (pt1->pt2) and (pt2->pt3)
/// Uses high-precision 128-bit arithmetic for exact results with i64 coordinates
/// Direct port from clipper.core.h line 753
#[inline]
pub fn cross_product_sign(pt1: Point64, pt2: Point64, pt3: Point64) -> i32 {
    // Use i128 for exact arithmetic (available on all Rust platforms)
    let a = pt2.x as i128 - pt1.x as i128;
    let b = pt3.y as i128 - pt2.y as i128;
    let c = pt2.y as i128 - pt1.y as i128;
    let d = pt3.x as i128 - pt2.x as i128;

    let ab = a * b;
    let cd = c * d;

    if ab > cd {
        1
    } else if ab < cd {
        -1
    } else {
        0
    }
}

/// Check if two line segments intersect
/// Direct port from clipper.core.h line 1003 - parametric algorithm
#[inline]
pub fn segments_intersect(
    seg1a: Point64,
    seg1b: Point64,
    seg2a: Point64,
    seg2b: Point64,
    inclusive: bool,
) -> bool {
    let dy1 = (seg1b.y - seg1a.y) as f64;
    let dx1 = (seg1b.x - seg1a.x) as f64;
    let dy2 = (seg2b.y - seg2a.y) as f64;
    let dx2 = (seg2b.x - seg2a.x) as f64;
    let cp = dy1 * dx2 - dy2 * dx1;
    if cp == 0.0 {
        return false; // parallel segments
    }

    if inclusive {
        let t = (seg1a.x - seg2a.x) as f64 * dy2 - (seg1a.y - seg2a.y) as f64 * dx2;
        if t == 0.0 {
            return true;
        }
        if t > 0.0 {
            if cp < 0.0 || t > cp {
                return false;
            }
        } else if cp > 0.0 || t < cp {
            return false;
        }
        let t = (seg1a.x - seg2a.x) as f64 * dy1 - (seg1a.y - seg2a.y) as f64 * dx1;
        if t == 0.0 {
            return true;
        }
        if t > 0.0 {
            cp > 0.0 && t <= cp
        } else {
            cp < 0.0 && t >= cp
        }
    } else {
        let t = (seg1a.x - seg2a.x) as f64 * dy2 - (seg1a.y - seg2a.y) as f64 * dx2;
        if t == 0.0 {
            return false;
        }
        if t > 0.0 {
            if cp < 0.0 || t >= cp {
                return false;
            }
        } else if cp > 0.0 || t <= cp {
            return false;
        }
        let t = (seg1a.x - seg2a.x) as f64 * dy1 - (seg1a.y - seg2a.y) as f64 * dx1;
        if t == 0.0 {
            return false;
        }
        if t > 0.0 {
            cp > 0.0 && t < cp
        } else {
            cp < 0.0 && t > cp
        }
    }
}

/// `clipper.core.h:1052–1072`: convert the displacement before coordinate addition.
#[inline]
pub fn get_closest_point_on_segment<T>(off_pt: Point<T>, seg1: Point<T>, seg2: Point<T>) -> Point<T>
where
    T: Copy + ToF64 + FromF64 + Num + PartialEq,
{
    if seg1.x == seg2.x && seg1.y == seg2.y {
        return seg1;
    }
    let dx = (seg2.x - seg1.x).to_f64();
    let dy = (seg2.y - seg1.y).to_f64();
    let mut q = ((off_pt.x - seg1.x).to_f64() * dx + (off_pt.y - seg1.y).to_f64() * dy)
        / (sqr(dx) + sqr(dy));
    if q < 0.0 {
        q = 0.0;
    } else if q > 1.0 {
        q = 1.0;
    }
    let (dx, dy) = if T::is_integral() {
        (nearbyint_f64(q * dx), nearbyint_f64(q * dy))
    } else {
        (q * dx, q * dy)
    };
    Point::new(seg1.x + T::from_f64(dx), seg1.y + T::from_f64(dy))
}

/// `clipper.core.h:834–846`.
#[inline]
pub fn perpendic_dist_from_line_sqrd<T>(pt: Point<T>, line1: Point<T>, line2: Point<T>) -> f64
where
    T: Num + Copy + ToF64,
{
    let a = (pt.x - line1.x).to_f64();
    let b = (pt.y - line1.y).to_f64();
    let c = (line2.x - line1.x).to_f64();
    let d = (line2.y - line1.y).to_f64();
    if c == 0.0 && d == 0.0 {
        return 0.0;
    }
    sqr(a * d - c * b) / (c * c + d * d)
}

/// Generate an elliptical path inscribed in a rectangle
/// Direct port from clipper.core.h Ellipse(const Rect<T>&) overload
pub fn ellipse_rect64(rect: &Rect64, steps: usize) -> Path64 {
    ellipse_point64(
        rect.mid_point(),
        rect.width() as f64 * 0.5,
        rect.height() as f64 * 0.5,
        steps,
    )
}

/// Generate an elliptical path inscribed in a rectangle
/// Direct port from clipper.core.h Ellipse(const Rect<T>&) overload
pub fn ellipse_rect_d(rect: &RectD, steps: usize) -> PathD {
    ellipse_point_d(
        rect.mid_point(),
        rect.width() * 0.5,
        rect.height() * 0.5,
        steps,
    )
}
