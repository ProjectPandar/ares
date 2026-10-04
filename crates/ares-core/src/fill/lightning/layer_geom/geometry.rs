//! Grounding geometry from Fill/Lightning/Layer.cpp.

use crate::geometry::{Coord, EdgeGrid, Point};

pub(super) fn polygon_collides_with_line(from: Point, to: Point, locator: &EdgeGrid) -> bool {
    let mut hit = false;
    let _ = locator.visit_cells_intersecting_line(from, to, |_r, _c, edges| {
        for &edge in edges {
            let (s1, s2) = locator.segment(edge);
            if segments_intersect(s1, s2, from, to) {
                hit = true;
                return false;
            }
        }
        true
    });
    hit
}

/// `Geometry::segments_intersect` (Geometry.hpp:123) — sign-product crossing
/// test including the collinear-overlap case (a strict `>0` orientation test
/// misses collinear overlapping segments).
fn segments_intersect(ip1: Point, ip2: Point, jp1: Point, jp2: Point) -> bool {
    let cross2 = |ax: i128, ay: i128, bx: i128, by: i128| ax * by - ay * bx;
    let signs = |a1: Point, a2: Point, b1: Point, b2: Point| -> (i32, i32) {
        let ivx = (a2.x() - a1.x()) as i128;
        let ivy = (a2.y() - a1.y()) as i128;
        let v1x = (b1.x() - a1.x()) as i128;
        let v1y = (b1.y() - a1.y()) as i128;
        let v2x = (b2.x() - a1.x()) as i128;
        let v2y = (b2.y() - a1.y()) as i128;
        let t1 = cross2(ivx, ivy, v1x, v1y);
        let t2 = cross2(ivx, ivy, v2x, v2y);
        let s = |t: i128| {
            if t > 0 {
                1
            } else if t < 0 {
                -1
            } else {
                0
            }
        };
        (s(t1), s(t2))
    };
    let sign1 = signs(ip1, ip2, jp1, jp2);
    let sign2 = signs(jp1, jp2, ip1, ip2);
    let test1 = sign1.0 * sign1.1;
    let test2 = sign2.0 * sign2.1;
    if test1 <= 0 && test2 <= 0 {
        if test1 != 0 || test2 != 0 {
            return true; // not collinear — they intersect
        }
        if sign1.0 == 0 && sign1.1 == 0 {
            // Collinear — check overlap on the dominant axis.
            let vi_x = (ip2.x() - ip1.x()).abs();
            let vi_y = (ip2.y() - ip1.y()).abs();
            let axis = if vi_x > vi_y { 0 } else { 1 };
            let coord = |p: Point| if axis == 0 { p.x() } else { p.y() };
            let (mut i, mut j, mut k, mut l) = (coord(ip1), coord(ip2), coord(jp1), coord(jp2));
            if i > j {
                std::mem::swap(&mut i, &mut j);
            }
            if k > l {
                std::mem::swap(&mut k, &mut l);
            }
            return (k >= i && k <= j) || (i >= k && i <= l);
        }
    }
    false
}

/// `line_alg::distance_to_squared` with the projected `closest_point` out-arg
/// (Layer.cpp:130) — returns (d², closest point on segment).
pub(super) fn segment_closest(a: Point, b: Point, p: Point) -> (f64, Point) {
    let (ax, ay) = (a.x() as f64, a.y() as f64);
    let (bx, by) = (b.x() as f64, b.y() as f64);
    let (px, py) = (p.x() as f64, p.y() as f64);
    let (dx, dy) = (bx - ax, by - ay);
    let len2 = dx * dx + dy * dy;
    let t = if len2 == 0.0 {
        0.0
    } else {
        (((px - ax) * dx + (py - ay) * dy) / len2).clamp(0.0, 1.0)
    };
    let (cx, cy) = (ax + t * dx, ay + t * dy);
    let d2 = (px - cx).powi(2) + (py - cy).powi(2);
    (d2, Point::new(cx as Coord, cy as Coord))
}

pub(super) fn dist(a: Point, b: Point) -> Coord {
    (((a.x() - b.x()) as f64).powi(2) + ((a.y() - b.y()) as f64).powi(2)).sqrt() as Coord
}

/// `getWeightedDistance` for a boundary location (no valence — boundary points
/// get no boost): just the euclidean distance (Layer.cpp uses the same fn on a
/// boundary Point; for boundary it's the plain distance).
pub(super) fn get_weighted_distance_boundary(boundary: Point, unsupported: Point) -> Coord {
    dist(boundary, unsupported)
}
