// Copyright (c) Angus Johnson 2010-2025.
// BSL-1.0. Rust seed by MatterHackers.
// Derived from clipper2-rust 2f5e9d4249bc4d624773094ff6164335d9ff6ae7.
// See ../LICENSE-Boost-1.0 and ../SOURCE.md for pinned source authority.

use super::*;

/// Test if three points are collinear (lie on the same straight line)
/// Direct port from clipper.core.h line 796 - simplified version for Point64
#[inline]
pub fn is_collinear(pt1: Point64, shared_pt: Point64, pt2: Point64) -> bool {
    let a = shared_pt.x - pt1.x;
    let b = pt2.y - shared_pt.y;
    let c = shared_pt.y - pt1.y;
    let d = pt2.x - shared_pt.x;

    // When checking for collinearity with very large coordinate values
    // ProductsAreEqual is more accurate than using CrossProduct
    products_are_equal(a, b, c, d)
}

/// Determine if a point is inside, on the boundary, or outside a polygon
/// Direct port from clipper.core.h line 1049 - simplified version for Point64
/// Uses the winding number algorithm with proper edge case handling
#[inline]
pub fn point_in_polygon(pt: Point64, polygon: &Path64) -> PointInPolygonResult {
    if polygon.len() < 3 {
        return PointInPolygonResult::IsOutside;
    }

    let mut val = 0i32;
    let mut first_idx = 0;

    // Find first vertex with different y coordinate than pt
    while first_idx < polygon.len() && polygon[first_idx].y == pt.y {
        first_idx += 1;
    }

    if first_idx == polygon.len() {
        // Not a proper polygon - all points have same y as test point
        return PointInPolygonResult::IsOutside;
    }

    let mut is_above = polygon[first_idx].y < pt.y;
    let starting_above = is_above;
    let mut curr_idx = (first_idx + 1) % polygon.len();
    let mut wrap_to_start = false;

    loop {
        // Handle wrapping around the polygon
        if curr_idx == polygon.len() {
            if wrap_to_start || first_idx == 0 {
                break;
            }
            wrap_to_start = true;
            curr_idx = 0;
        }

        // Skip vertices while moving in same direction relative to pt.y
        if is_above {
            while curr_idx != polygon.len()
                && (!wrap_to_start || curr_idx < first_idx)
                && polygon[curr_idx].y < pt.y
            {
                curr_idx += 1;
            }
            if curr_idx == polygon.len() || (wrap_to_start && curr_idx >= first_idx) {
                continue;
            }
        } else {
            while curr_idx != polygon.len()
                && (!wrap_to_start || curr_idx < first_idx)
                && polygon[curr_idx].y > pt.y
            {
                curr_idx += 1;
            }
            if curr_idx == polygon.len() || (wrap_to_start && curr_idx >= first_idx) {
                continue;
            }
        }

        let curr = &polygon[curr_idx];
        let prev_idx = if curr_idx == 0 {
            polygon.len() - 1
        } else {
            curr_idx - 1
        };
        let prev = &polygon[prev_idx];

        if curr.y == pt.y {
            if curr.x == pt.x || (curr.y == prev.y && ((pt.x < prev.x) != (pt.x < curr.x))) {
                return PointInPolygonResult::IsOn;
            }
            curr_idx += 1;
            if curr_idx == first_idx {
                break;
            }
            continue;
        }

        if pt.x < curr.x && pt.x < prev.x {
            // We're only interested in edges crossing on the left
        } else if pt.x > prev.x && pt.x > curr.x {
            val = 1 - val; // Toggle val
        } else {
            let d = cross_product_sign(*prev, *curr, pt);
            if d == 0 {
                return PointInPolygonResult::IsOn;
            }
            if (d < 0) == is_above {
                val = 1 - val;
            }
        }

        is_above = !is_above;
        curr_idx += 1;
    }

    // Final edge case check
    if is_above != starting_above {
        let curr_idx = if curr_idx >= polygon.len() {
            0
        } else {
            curr_idx
        };
        let prev_idx = if curr_idx == 0 {
            polygon.len() - 1
        } else {
            curr_idx - 1
        };
        let curr = &polygon[curr_idx];
        let prev = &polygon[prev_idx];
        let d = cross_product_sign(*prev, *curr, pt);
        if d == 0 {
            return PointInPolygonResult::IsOn;
        }
        if (d < 0) == is_above {
            val = 1 - val;
        }
    }

    if val == 0 {
        PointInPolygonResult::IsOutside
    } else {
        PointInPolygonResult::IsInside
    }
}

/// Trait for converting from f64 with appropriate rounding behavior
/// Matches C++ Point<T> constructor semantics: rounds for integral types, casts for float
pub trait FromF64: Copy {
    fn from_f64(val: f64) -> Self;
    fn is_integral() -> bool;
}

impl FromF64 for i64 {
    #[inline]
    fn from_f64(val: f64) -> Self {
        val.round() as i64
    }
    #[inline]
    fn is_integral() -> bool {
        true
    }
}

impl FromF64 for f64 {
    #[inline]
    fn from_f64(val: f64) -> Self {
        val
    }
    #[inline]
    fn is_integral() -> bool {
        false
    }
}

/// Scale a rectangle with type conversion
/// Direct port from clipper.core.h line 405
#[inline]
pub fn scale_rect<T1, T2>(rect: &Rect<T2>, scale: f64) -> Rect<T1>
where
    T1: Num + Copy + PartialOrd + FromF64,
    T2: Copy + ToF64,
{
    Rect {
        left: T1::from_f64(rect.left.to_f64() * scale),
        top: T1::from_f64(rect.top.to_f64() * scale),
        right: T1::from_f64(rect.right.to_f64() * scale),
        bottom: T1::from_f64(rect.bottom.to_f64() * scale),
    }
}

/// Scale a path with type conversion and separate x/y scales
/// Direct port from clipper.core.h line 523
#[inline]
pub fn scale_path<T1, T2>(
    path: &Path<T2>,
    scale_x: f64,
    scale_y: f64,
    error_code: &mut i32,
) -> Path<T1>
where
    T1: Num + Copy + PartialOrd + FromF64,
    T2: Copy + ToF64,
{
    let mut sx = scale_x;
    let mut sy = scale_y;

    if sx == 0.0 || sy == 0.0 {
        *error_code |= errors::SCALE_ERROR_I;
        // if no exception, treat as non-fatal error
        if sx == 0.0 {
            sx = 1.0;
        }
        if sy == 0.0 {
            sy = 1.0;
        }
    }

    let mut result = Vec::with_capacity(path.len());
    for pt in path {
        result.push(Point {
            x: T1::from_f64(pt.x.to_f64() * sx),
            y: T1::from_f64(pt.y.to_f64() * sy),

            z: pt.z,
        });
    }
    result
}

/// Scale a path with uniform scale
/// Direct port from clipper.core.h line 550
#[inline]
pub fn scale_path_uniform<T1, T2>(path: &Path<T2>, scale: f64, error_code: &mut i32) -> Path<T1>
where
    T1: Num + Copy + PartialOrd + FromF64,
    T2: Copy + ToF64,
{
    scale_path(path, scale, scale, error_code)
}

/// Scale multiple paths with type conversion and separate x/y scales
/// Includes range checking for integral output types
/// Direct port from clipper.core.h line 557
#[inline]
pub fn scale_paths<T1, T2>(
    paths: &Paths<T2>,
    scale_x: f64,
    scale_y: f64,
    error_code: &mut i32,
) -> Paths<T1>
where
    T1: Num + Copy + PartialOrd + FromF64,
    T2: Copy + ToF64,
{
    // Range check for integral output types
    if T1::is_integral() {
        // Compute bounds using ToF64 trait
        let mut xmin = f64::MAX;
        let mut ymin = f64::MAX;
        let mut xmax = f64::MIN;
        let mut ymax = f64::MIN;
        for path in paths {
            for p in path {
                let x = p.x.to_f64();
                let y = p.y.to_f64();
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
        if (xmin * scale_x) < constants::MIN_COORD_D
            || (xmax * scale_x) > constants::MAX_COORD_D
            || (ymin * scale_y) < constants::MIN_COORD_D
            || (ymax * scale_y) > constants::MAX_COORD_D
        {
            *error_code |= errors::RANGE_ERROR_I;
            return Paths::new(); // empty path
        }
    }

    let mut result = Vec::with_capacity(paths.len());
    for path in paths {
        result.push(scale_path(path, scale_x, scale_y, error_code));
    }
    result
}

/// Scale multiple paths with uniform scale
/// Direct port from clipper.core.h line 584
#[inline]
pub fn scale_paths_uniform<T1, T2>(paths: &Paths<T2>, scale: f64, error_code: &mut i32) -> Paths<T1>
where
    T1: Num + Copy + PartialOrd + FromF64,
    T2: Copy + ToF64,
{
    scale_paths(paths, scale, scale, error_code)
}

/// Transform a path between types (no scaling, just type conversion)
/// Direct port from clipper.core.h line 591
#[inline]
pub fn transform_path<T1, T2>(path: &Path<T2>) -> Path<T1>
where
    T1: Num + Copy + FromF64,
    T2: Copy + ToF64,
{
    let mut result = Vec::with_capacity(path.len());
    for pt in path {
        result.push(Point {
            x: T1::from_f64(pt.x.to_f64()),
            y: T1::from_f64(pt.y.to_f64()),

            z: pt.z,
        });
    }
    result
}
