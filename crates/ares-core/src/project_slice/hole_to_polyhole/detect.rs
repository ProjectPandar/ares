//! Detection predicates for `hole_to_polyhole` — the upstream
//! helpers called by `_transform_hole_to_polyholes`
//! (`PrintObject.cpp:205-238`): `Polygon::centroid`,
//! `Point::distance_to`, and the `convex_points(PI)` convexity
//! predicate (`Polygon.cpp:310-315` via
//! `filter_convex_concave_points_by_angle_threshold` :275-308 and
//! `filter_points_by_vectors` :235-258).

use super::{CoordT, EPSILON, HoleId};
use crate::geometry::{CoordinateScale, Point, Polygon};

/// `PrintObject.cpp:205-238` — circularity test returning the upstream
/// candidate tuple when the hole qualifies.
pub(super) fn detect(
    hole: &Polygon,
    options: &crate::RegionOptions,
    scale: CoordinateScale,
) -> Option<HoleId> {
    let points = hole.points();
    let center = centroid(points);
    // `double` statistics upstream (:211-218); only the tuple field
    // stores `float`.
    let mut diameter_min = f64::from(f32::MAX);
    let mut diameter_max = 0.0f64;
    let mut diameter_sum = 0.0f64;
    for point in points {
        let dist = distance_to(*point, center);
        diameter_min = diameter_min.min(dist);
        diameter_max = diameter_max.max(dist);
        diameter_sum += dist;
    }
    // `Line.a + Line.b) / 2` — integer midpoint (`PrintObject.cpp:225-230`).
    let mut diameter_line_min = f64::from(f32::MAX);
    let mut diameter_line_max = 0.0f64;
    for line in hole.lines() {
        let midline = Point::new((line.a.x() + line.b.x()) / 2, (line.a.y() + line.b.y()) / 2);
        let dist = distance_to(midline, center);
        diameter_line_min = diameter_line_min.min(dist);
        diameter_line_max = diameter_line_max.max(dist);
    }
    let mean_radius = diameter_sum / points.len() as f64;
    // :234-236 — `get_abs_value(unscaled(mean_radius))`: the Float branch
    // returns the literal mm value; the Percent branch multiplies the
    // *unscaled* mean radius (mm) by percent/100. `scale_()` then
    // truncates to coord_t.
    // `unscaled(mean_radius)` on the double (`Point.hpp`:
    // `coord * SCALING_FACTOR`, no integer truncation).
    let mean_radius_mm = mean_radius * scale.factor();
    let threshold_abs = match options.hole_to_polyhole_threshold {
        crate::FloatOrPercent::Float(value) => value,
        crate::FloatOrPercent::Percent(percent) => mean_radius_mm * percent.0 / 100.0,
    };
    // `scale_(x)` upstream = plain `x / SCALING_FACTOR` converted to
    // coord_t (`libslic3r.h:94-95`); `as` on f64→i64 saturates, so
    // out-of-range user thresholds saturate instead of panicking.
    let scaled_eps = (EPSILON / scale.factor()) as CoordT;
    let max_variation = scaled_eps.max((threshold_abs / scale.factor()) as CoordT);
    if diameter_max - diameter_min < (max_variation as f64) * 2.0
        && diameter_line_max - diameter_line_min < (max_variation as f64) * 2.0
    {
        Some(HoleId {
            center,
            diameter_max: diameter_max as f32,
            extruder_id: options.outer_wall_filament_id.0,
            max_variation,
            twist: options.hole_to_polyhole_twisted.0,
        })
    } else {
        None
    }
}

/// `Polygon::centroid` (`Polygon.cpp:141-157`): weighted vertex
/// centroid, `Point(Vec2d)` rounds each coordinate.
pub(super) fn centroid(points: &[Point]) -> Point {
    let mut area_sum = 0.0;
    let mut c = (0.0, 0.0);
    if points.len() >= 3 {
        let mut p1 = (
            points[points.len() - 1].x() as f64,
            points[points.len() - 1].y() as f64,
        );
        for point in points {
            let p2 = (point.x() as f64, point.y() as f64);
            let a = p1.0 * p2.1 - p1.1 * p2.0;
            area_sum += a;
            c.0 += (p1.0 + p2.0) * a;
            c.1 += (p1.1 + p2.1) * a;
            p1 = p2;
        }
    }
    Point::new(
        (c.0 / (3.0 * area_sum)).round() as CoordT,
        (c.1 / (3.0 * area_sum)).round() as CoordT,
    )
}

/// `Point::distance_to` — Euclidean distance in coord units.
pub(super) fn distance_to(point: Point, center: Point) -> f64 {
    let dx = (point.x() - center.x()) as f64;
    let dy = (point.y() - center.y()) as f64;
    (dx * dx + dy * dy).sqrt()
}

/// `Polygon::convex_points(PI)` (`Polygon.cpp:310-315`,
/// `filter_convex_concave_points_by_angle_threshold` :275-308,
/// `filter_points_by_vectors` :235-258): vertices where the *incoming*
/// edge (v1 = p - prev) and *outgoing* edge (v2 = next - p) satisfy
/// `cross2(v1, v2) > 0` and the normalized dot product is below
/// `cos(nextafter(PI, +inf))`. The dot clause is unreachable; kept
/// verbatim so the predicate stays a literal port.
pub(super) fn convex_points_pi(hole: &Polygon) -> Vec<Point> {
    let points = hole.points();
    if points.len() < 2 {
        return Vec::new();
    }
    // cos(nextafter(PI, +inf)) — the upstream threshold value.
    let cos_threshold = f64::from_bits(std::f64::consts::PI.to_bits() + 1).cos();
    let mut out = Vec::new();
    let mut p1 = *points.last().unwrap();
    let prev = points[points.len() - 2];
    let mut v1 = ((p1.x() - prev.x()) as f64, (p1.y() - prev.y()) as f64);
    for &p2 in points {
        let v2 = ((p2.x() - p1.x()) as f64, (p2.y() - p1.y()) as f64);
        // cross2(v1, v2) > 0 with Vec2d (f64) operands.
        if v1.0 * v2.1 - v1.1 * v2.0 > 0.0 {
            let n1 = normalize(v1);
            let n2 = normalize(v2);
            if n1.0 * n2.0 + n1.1 * n2.1 < cos_threshold {
                out.push(p1);
            }
        }
        v1 = v2;
        p1 = p2;
    }
    out
}

/// `Vec2d::normalized()` — zero vectors yield NaN components upstream;
/// the `<` comparison then fails (NaN), matching upstream.
fn normalize(v: (f64, f64)) -> (f64, f64) {
    let len = v.0.hypot(v.1);
    (v.0 / len, v.1 / len)
}
