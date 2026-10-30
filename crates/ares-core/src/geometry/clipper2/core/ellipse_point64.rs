// Copyright (c) Angus Johnson 2010-2025.
// BSL-1.0. Rust seed by MatterHackers.
// Derived from clipper2-rust 2f5e9d4249bc4d624773094ff6164335d9ff6ae7.
// See ../LICENSE-Boost-1.0 and ../SOURCE.md for pinned source authority.

use super::*;

/// Generate an elliptical path around a center point
/// Direct port from clipper.h line 613 (Ellipse template function)
pub fn ellipse_point64(center: Point64, radius_x: f64, radius_y: f64, steps: usize) -> Path64 {
    if radius_x <= 0.0 {
        return Path64::new();
    }
    let radius_y = if radius_y <= 0.0 { radius_x } else { radius_y };
    let steps = if steps <= 2 {
        (constants::PI * ((radius_x + radius_y) / 2.0).sqrt()) as usize
    } else {
        steps
    };
    if steps == 0 {
        return Path64::new();
    }

    let si = (2.0 * constants::PI / steps as f64).sin();
    let co = (2.0 * constants::PI / steps as f64).cos();
    let mut dx = co;
    let mut dy = si;
    let mut result = Path64::with_capacity(steps);
    result.push(Point64::new(
        (center.x as f64 + radius_x).round() as i64,
        center.y,
    ));
    for _ in 1..steps {
        result.push(Point64::new(
            (center.x as f64 + radius_x * dx).round() as i64,
            (center.y as f64 + radius_y * dy).round() as i64,
        ));
        let x = dx * co - dy * si;
        dy = dy * co + dx * si;
        dx = x;
    }
    result
}

/// Generate an elliptical path around a center point (PointD version)
/// Direct port from clipper.h line 613 (Ellipse template function)
pub fn ellipse_point_d(center: PointD, radius_x: f64, radius_y: f64, steps: usize) -> PathD {
    if radius_x <= 0.0 {
        return PathD::new();
    }
    let radius_y = if radius_y <= 0.0 { radius_x } else { radius_y };
    let steps = if steps <= 2 {
        (constants::PI * ((radius_x + radius_y) / 2.0).sqrt()) as usize
    } else {
        steps
    };
    if steps == 0 {
        return PathD::new();
    }

    let si = (2.0 * constants::PI / steps as f64).sin();
    let co = (2.0 * constants::PI / steps as f64).cos();
    let mut dx = co;
    let mut dy = si;
    let mut result = PathD::with_capacity(steps);
    result.push(PointD::new(center.x + radius_x, center.y));
    for _ in 1..steps {
        result.push(PointD::new(
            center.x + radius_x * dx,
            center.y + radius_y * dy,
        ));
        let x = dx * co - dy * si;
        dy = dy * co + dx * si;
        dx = x;
    }
    result
}
