//! `connect_lines_using_hooks` + `add_hook` (`FillAdaptive.cpp:670-1050`):
//! T-junction detection and L-shaped hook extension for the adaptive
//! infill lines, before the chain/connect tail.

use crate::geometry::{ExPolygon, Line, Point, Polyline};

/// `FillAdaptive.cpp:562-618` — one T-junction between an infill line end
/// and another line's interior.
#[derive(Clone, Copy, Debug)]
struct TJoint {
    /// Polyline whose endpoint touches the crossing line.
    line: usize,
    front: bool,
    /// The crossed line (index into the 2-point source lines).
    crossing: usize,
    /// Where the endpoint projects onto the crossed line.
    point: Point,
}

/// `FillAdaptive.cpp:801-1050` — merge collinear split segments, find the
/// T-joints, extend hooks, then hand the result to the chain/connect tail.
pub(crate) fn connect_lines_using_hooks(
    lines: Vec<Polyline>,
    _boundary: &ExPolygon,
    spacing: f64,
    hook_length: f64,
    _hook_length_max: f64,
) -> Vec<Polyline> {
    if lines.len() <= 1 {
        return lines;
    }
    if hook_length <= 0.0 {
        // Upstream ignores open hooks (`FillAdaptive.cpp:673-675`).
        return lines;
    }
    // 19% overlap (`FillAdaptive.cpp:809`).
    let scaled_offset = 0.81 * spacing;
    let mut lines = lines;

    // Merge collinear segments split by tiny gaps (`:823-880`):
    // r2_close = 1200² lattice units, |cos| > 0.99.
    let mut merged = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let mut current = std::mem::replace(&mut lines[i], Polyline::new(Vec::new()));
        let mut j = i + 1;
        while j < lines.len() {
            let other = &lines[j];
            let (Some(a0), Some(b0)) = (
                current.points().first().copied(),
                current.points().last().copied(),
            ) else {
                break;
            };
            let (Some(a1), Some(b1)) = (
                other.points().first().copied(),
                other.points().last().copied(),
            ) else {
                break;
            };
            let d = distance_squared_min(a0, b0, a1, b1);
            if d < 1200.0 * 1200.0 && collinear(a0, b0, a1, b1) {
                let other = std::mem::replace(&mut lines[j], Polyline::new(Vec::new()));
                let pts: Vec<Point> = current.into_points();
                let mut pts = pts;
                let opts = other.into_points();
                // Append in the collinear direction.
                if pts.last() == opts.first() {
                    pts.extend(opts);
                } else {
                    let mut rev = opts;
                    rev.reverse();
                    rev.extend(pts);
                    pts = rev;
                }
                current = Polyline::new(pts);
            }
            j += 1;
        }
        merged.push(current);
        i += 1;
    }
    lines = merged;
    lines.retain(|line| line.points().len() >= 2);

    // T-joint pass (`:940-1010`): find each polyline endpoint that lies
    // within 1000² of another line's interior.
    let source: Vec<Line> = lines
        .iter()
        .filter(|l| l.points().len() >= 2)
        .map(|l| Line::new(*l.points().first().unwrap(), *l.points().last().unwrap()))
        .collect();
    let mut joints: Vec<TJoint> = Vec::new();
    for (index, line) in source.iter().enumerate() {
        for front in [true, false] {
            let endpoint = if front { line.a } else { line.b };
            for (other, candidate) in source.iter().enumerate() {
                if other == index {
                    continue;
                }
                if distance_point_to_segment_squared(endpoint, candidate) <= 1000.0 * 1000.0
                    && !touches_endpoint(endpoint, candidate)
                {
                    joints.push(TJoint {
                        line: index,
                        front,
                        crossing: other,
                        point: endpoint,
                    });
                }
            }
        }
    }

    // `add_hook` per joint (`:670-778`): extend along the crossing line
    // direction, trimmed by the crossed line's offset.
    for joint in &joints {
        add_hook(joint, &source, scaled_offset, hook_length, &mut lines);
    }

    lines
}

/// L-shaped hook extension (`FillAdaptive.cpp:670-778`, simplified to the
/// dominant path: trim the start by the crossing line offset, extend
/// hook_length along the crossing direction, avoid crossing any source
/// line).
fn add_hook(
    joint: &TJoint,
    source: &[Line],
    scaled_offset: f64,
    hook_length: f64,
    lines: &mut [Polyline],
) {
    let crossing = source[joint.crossing];
    let dir = normalize(
        crossing.b.x() - crossing.a.x(),
        crossing.b.y() - crossing.a.y(),
    );
    let Some((dir_x, dir_y)) = dir else {
        return;
    };
    // Trim the hook start away from the crossed line centerline.
    let start = (
        joint.point.x() as f64 + dir_x * scaled_offset,
        joint.point.y() as f64 + dir_y * scaled_offset,
    );
    // Forward extension, trimmed by the first crossed line.
    let mut forward = hook_length;
    for candidate in source.iter().enumerate() {
        if candidate.0 == joint.crossing {
            continue;
        }
        if let Some(t) = ray_segment_hit(start, (dir_x, dir_y), candidate.1) {
            if t < forward {
                forward = t;
            }
        }
    }
    // Backward extension.
    let mut backward = 0.0_f64;
    if forward < hook_length - 0.0 {
        backward = hook_length;
        for candidate in source.iter().enumerate() {
            if candidate.0 == joint.crossing {
                continue;
            }
            if let Some(t) = ray_segment_hit(start, (-dir_x, -dir_y), candidate.1) {
                if t < backward {
                    backward = t;
                }
            }
        }
    }
    let length = if forward >= backward {
        forward
    } else {
        -backward
    };
    let end = Point::new(
        (start.0 + length * dir_x).round() as i64,
        (start.1 + length * dir_y).round() as i64,
    );
    let start_point = Point::new(start.0.round() as i64, start.1.round() as i64);
    let polyline = &mut lines[joint.line];
    let mut points: Vec<Point> = polyline.points().to_vec();
    if joint.front {
        if !points.is_empty() {
            points[0] = start_point;
            points.insert(0, end);
        }
    } else if !points.is_empty() {
        let last = points.len() - 1;
        points[last] = start_point;
        points.push(end);
    }
    *polyline = Polyline::new(points);
}

fn distance_squared_min(a0: Point, b0: Point, a1: Point, b1: Point) -> f64 {
    let pairs = [(a0, a1), (a0, b1), (b0, a1), (b0, b1)];
    pairs
        .iter()
        .map(|&(p, q)| {
            let dx = (p.x() - q.x()) as f64;
            let dy = (p.y() - q.y()) as f64;
            dx * dx + dy * dy
        })
        .fold(f64::INFINITY, f64::min)
}

fn collinear(a0: Point, b0: Point, a1: Point, b1: Point) -> bool {
    let v1 = ((b0.x() - a0.x()) as f64, (b0.y() - a0.y()) as f64);
    let v2 = ((b1.x() - a1.x()) as f64, (b1.y() - a1.y()) as f64);
    let n1 = (v1.0 * v1.0 + v1.1 * v1.1).sqrt();
    let n2 = (v2.0 * v2.0 + v2.1 * v2.1).sqrt();
    if n1 <= 0.0 || n2 <= 0.0 {
        return false;
    }
    let d = (v1.0 * v2.0 + v1.1 * v2.1) / (n1 * n2);
    d.abs() > 0.99
}

fn distance_point_to_segment_squared(point: Point, line: &Line) -> f64 {
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

fn touches_endpoint(point: Point, line: &Line) -> bool {
    (point.x() == line.a.x() && point.y() == line.a.y())
        || (point.x() == line.b.x() && point.y() == line.b.y())
}

fn normalize(x: i64, y: i64) -> Option<(f64, f64)> {
    let xf = x as f64;
    let yf = y as f64;
    let n = (xf * xf + yf * yf).sqrt();
    if n <= 0.0 {
        None
    } else {
        Some((xf / n, yf / n))
    }
}

fn ray_segment_hit(origin: (f64, f64), dir: (f64, f64), line: &Line) -> Option<f64> {
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

#[cfg(test)]
mod tests;
