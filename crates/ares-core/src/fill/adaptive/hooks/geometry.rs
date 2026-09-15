//! Hook/intersect geometry helpers (`FillAdaptive.cpp:733-748` thick-line
//! shift, :562-618 direction predicates, :642-657 segment math).

use super::Joint;
use crate::geometry::{Line, Point};

pub(super) fn shift_from_thick_line(dir_x: f64, dir_y: f64, line: &Line, trim: f64) -> f64 {
    let dx = (line.b.x() - line.a.x()) as f64;
    let dy = (line.b.y() - line.a.y()) as f64;
    let n = (dx * dx + dy * dy).sqrt();
    if n <= 0.0 {
        return 0.0;
    }
    trim * (dir_x * dy / n - dir_y * dx / n).abs()
}

pub(super) struct Thresholds {
    pub(super) drop_both_sides: f64,
    pub(super) anchor_both_sides: f64,
    pub(super) drop_single_side: f64,
    pub(super) anchor_single_side: f64,
}

impl Thresholds {
    pub(super) fn new(scaled_offset: f64) -> Self {
        let drop_both = scaled_offset * (2.0 / (std::f64::consts::FRAC_PI_6).cos() + 0.5);
        let drop_single = scaled_offset * (1.0 / (std::f64::consts::FRAC_PI_6).cos() + 1.5);
        Self {
            drop_both_sides: drop_both,
            anchor_both_sides: drop_both + scaled_offset,
            drop_single_side: drop_single,
            anchor_single_side: drop_single + scaled_offset,
        }
    }
}

pub(super) fn has_other_tjoint(source: &[Line], index: usize, front: bool, radius: f64) -> bool {
    let line = &source[index];
    let endpoint = if front { line.b } else { line.a };
    for (other, candidate) in source.iter().enumerate() {
        if other == index {
            continue;
        }
        if distance_point_to_segment_squared(endpoint, candidate) <= radius * radius
            && projects_interior(endpoint, candidate)
        {
            return true;
        }
    }
    false
}

pub(super) fn projects_interior(point: Point, line: &Line) -> bool {
    let vx = (line.b.x() - line.a.x()) as f64;
    let vy = (line.b.y() - line.a.y()) as f64;
    let l2 = vx * vx + vy * vy;
    if l2 <= 0.0 {
        return false;
    }
    let t = (point.x() - line.a.x()) as f64 * vx + (point.y() - line.a.y()) as f64 * vy;
    t > 0.0 && t < l2
}

pub(super) fn left_of(closest_dir: (f64, f64), intersect_dir: (f64, f64)) -> bool {
    closest_dir.0 * intersect_dir.1 - closest_dir.1 * intersect_dir.0 > 0.0
}

pub(super) fn line_dir(line: Line, forward: bool) -> (f64, f64) {
    let mut dx = (line.b.x() - line.a.x()) as f64;
    let mut dy = (line.b.y() - line.a.y()) as f64;
    if !forward {
        dx = -dx;
        dy = -dy;
    }
    (dx, dy)
}

pub(super) fn candidate_dir(line: Line) -> (f64, f64) {
    line_dir(line, true)
}

pub(super) fn length(line: &Line) -> f64 {
    let dx = (line.b.x() - line.a.x()) as f64;
    let dy = (line.b.y() - line.a.y()) as f64;
    (dx * dx + dy * dy).sqrt()
}

pub(super) fn dot(x: f64, y: f64, dir: (f64, f64)) -> f64 {
    x * dir.0 + y * dir.1
}

pub(super) fn distance_squared(a: Point, b: Point) -> f64 {
    let dx = (a.x() - b.x()) as f64;
    let dy = (a.y() - b.y()) as f64;
    dx * dx + dy * dy
}

pub(super) fn distance_point_to_segment_squared(point: Point, line: &Line) -> f64 {
    let px = point.x() as f64;
    let py = point.y() as f64;
    let ax = line.a.x() as f64;
    let ay = line.a.y() as f64;
    let bx = line.b.x() as f64;
    let by = line.b.y() as f64;
    let vx = bx - ax;
    let vy = by - ay;
    let l2 = vx * vx + vy * vy;
    if l2 <= 0.0 {
        let dx = px - ax;
        let dy = py - ay;
        return dx * dx + dy * dy;
    }
    let t = (((px - ax) * vx + (py - ay) * vy) / l2).clamp(0.0, 1.0);
    let dx = px - (ax + t * vx);
    let dy = py - (ay + t * vy);
    dx * dx + dy * dy
}

pub(super) fn normalize(x: i64, y: i64) -> Option<(f64, f64)> {
    let xf = x as f64;
    let yf = y as f64;
    let n = (xf * xf + yf * yf).sqrt();
    if n <= 0.0 {
        None
    } else {
        Some((xf / n, yf / n))
    }
}

pub(super) fn segment_line_intersection(line: Line, other: Line) -> Option<Point> {
    segments_cross(line.a, line.b, other.a, other.b)
        .map(|(x, y)| Point::new(x.round() as i64, y.round() as i64))
}

pub(super) fn segments_cross(a1: Point, a2: Point, b1: Point, b2: Point) -> Option<(f64, f64)> {
    let d1x = (a2.x() - a1.x()) as f64;
    let d1y = (a2.y() - a1.y()) as f64;
    let d2x = (b2.x() - b1.x()) as f64;
    let d2y = (b2.y() - b1.y()) as f64;
    let denom = d1x * d2y - d1y * d2x;
    if denom.abs() < 1e-12 {
        return None;
    }
    let t = ((b1.x() - a1.x()) as f64 * d2y - (b1.y() - a1.y()) as f64 * d2x) / denom;
    let u = ((b1.x() - a1.x()) as f64 * d1y - (b1.y() - a1.y()) as f64 * d1x) / denom;
    if (0.0..=1.0).contains(&t) && (0.0..=1.0).contains(&u) {
        Some((a1.x() as f64 + t * d1x, a1.y() as f64 + t * d1y))
    } else {
        None
    }
}

pub(super) fn ray_segment_hit(origin: (f64, f64), dir: (f64, f64), line: &Line) -> Option<f64> {
    let ax = line.a.x() as f64 - origin.0;
    let ay = line.a.y() as f64 - origin.1;
    let bx = line.b.x() as f64 - origin.0;
    let by = line.b.y() as f64 - origin.1;
    let dx = bx - ax;
    let dy = by - ay;
    let denom = dir.0 * dy - dir.1 * dx;
    if denom.abs() < 1e-12 {
        return None;
    }
    let t = (ax * dy - ay * dx) / denom;
    let u = (ax * dir.1 - ay * dir.0) / -denom;
    if t > 0.0 && (0.0..=1.0).contains(&u) {
        Some(t)
    } else {
        None
    }
}
