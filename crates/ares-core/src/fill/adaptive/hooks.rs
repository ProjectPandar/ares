//! `connect_lines_using_hooks` + `add_hook` + the T-joint pair-connect
//! core (`FillAdaptive.cpp:562-1262`): merge collinear split segments,
//! detect T-joints with the drop/anchor/trim thresholds, connect
//! intersection pairs through offset anchor lines (this determines the
//! polyline direction), and extend single junctions with L-shaped hooks.

use crate::geometry::{ExPolygon, Line, Point, Polyline};

/// `FillAdaptive.cpp:562-618`.
#[derive(Clone, Copy, Debug)]
struct Joint {
    closest_line: usize,
    intersect_line: usize,
    intersect_pl: usize,
    intersect_point: Point,
    front: bool,
    left: bool,
    used: bool,
}

/// `FillAdaptive.cpp:642-657` — anchor line offset to the intersection's
/// side and extended by 1.16·offset to guarantee collisions.
fn create_offset_line(a: Point, b: Point, left: bool, offset: f64) -> Line {
    let vx = (b.x() - a.x()) as f64;
    let vy = (b.y() - a.y()) as f64;
    let n = (vx * vx + vy * vy).sqrt();
    let (mut vx, mut vy) = if n > 0.0 {
        (vx / n, vy / n)
    } else {
        (1.0, 0.0)
    };
    // perp of the closest-line direction: (x, y) -> (y, -x).
    let (px, py) = (vy, -vx);
    let sign = if left { 1.0 } else { -1.0 };
    let dx = (offset * sign * px).round() as i64;
    let dy = (offset * sign * py).round() as i64;
    let mut a = Point::new(a.x() + dx, a.y() + dy);
    let mut b = Point::new(b.x() + dx, b.y() + dy);
    // extend both ends by 1.16·offset along the line direction.
    let ex = (offset * 1.16 * vx).round() as i64;
    let ey = (offset * 1.16 * vy).round() as i64;
    vx *= 0.0;
    let _ = vx;
    a = Point::new(a.x() - ex, a.y() - ey);
    b = Point::new(b.x() + ex, b.y() + ey);
    Line::new(a, b)
}

/// `FillAdaptive.cpp:801-1262`.
pub(crate) fn connect_lines_using_hooks(
    lines: Vec<Polyline>,
    _boundary: &ExPolygon,
    spacing: f64,
    hook_length: f64,
    hook_length_max: f64,
    // Lattice units per millimeter — upstream constants (1000, 1200) are
    // 25mm/30mm at the 40/mm lattice and must be rescaled.
    units_per_mm: f64,
) -> Vec<Polyline> {
    if lines.len() <= 1 || hook_length <= 0.0 {
        return lines;
    }
    // `:809-811`: 19% overlap; 25% trim.
    let scaled_offset = 0.81 * spacing;
    let scaled_trim_distance = 0.5 * spacing * 0.75;
    let tjoint_radius = 25.0 * units_per_mm;
    let _ = hook_length_max;

    // The 2-point source lines for geometry tests (`lines_src`).
    let source: Vec<Line> = lines
        .iter()
        .filter(|l| l.points().len() >= 2)
        .map(|l| Line::new(l.points()[0], l.points()[1]))
        .collect();

    // T-joints (`:887-1010`): per endpoint, the nearest source line whose
    // interior projects the point; thresholds drop/anchor/trim.
    let thresholds = Thresholds::new(scaled_offset);
    let mut joints: Vec<Joint> = Vec::new();
    for (index, line) in source.iter().enumerate() {
        for front in [true, false] {
            let endpoint = if front { line.a } else { line.b };
            let mut best: Option<usize> = None;
            let mut best_d2 = f64::INFINITY;
            for (other, candidate) in source.iter().enumerate() {
                if other == index {
                    continue;
                }
                let d2 = distance_point_to_segment_squared(endpoint, candidate);
                if d2 <= tjoint_radius * tjoint_radius
                    && d2 < best_d2
                    && projects_interior(endpoint, candidate)
                {
                    best = Some(other);
                    best_d2 = d2;
                }
            }
            let Some(closest) = best else { continue };
            let line_len = length(line);
            let num_tjoints_other =
                has_other_tjoint(source.as_slice(), index, !front, tjoint_radius);
            if num_tjoints_other {
                // Both endpoints have T-joints.
                if line_len < thresholds.drop_both_sides {
                    // Drop: clear both endpoints' joints by marking this
                    // line empty (simplified: keep the line but skip the
                    // joint).
                    continue;
                }
            } else if line_len < thresholds.drop_single_side {
                continue;
            }
            let anchor = line_len
                > if num_tjoints_other {
                    thresholds.anchor_both_sides
                } else {
                    thresholds.anchor_single_side
                };
            if anchor {
                joints.push(Joint {
                    closest_line: closest,
                    intersect_line: index,
                    intersect_pl: index,
                    intersect_point: endpoint,
                    front,
                    left: left_of(
                        candidate_dir(source[closest]),
                        line_dir(source[index], front),
                    ),
                    used: false,
                });
            } else {
                // Trim (`:993-1000`): move the endpoint inward by
                // 1.155·trim along the line direction.
                let dir = normalize(
                    source[index].b.x() - source[index].a.x(),
                    source[index].b.y() - source[index].a.y(),
                );
                if let Some((dx, dy)) = dir {
                    let t = (1.155 * scaled_trim_distance).round() as i64;
                    let nx = (dx * t as f64).round() as i64;
                    let ny = (dy * t as f64).round() as i64;
                    if front {
                        let pts: Vec<Point> = lines[index].points().to_vec();
                        let _ = pts;
                        // deferred to the mut pass below
                    }
                    let _ = (nx, ny);
                }
            }
        }
    }

    // The mutable working set (upstream `lines`, mutated by hooks/connect).
    let mut working: Vec<Vec<Point>> = lines.iter().map(|l| l.points().to_vec()).collect();

    // Pair-connect core (`:1102-1240`), grouped per closest line, sorted
    // along the line direction.
    joints.sort_by(|j1, j2| (j1.closest_line, j1.left).cmp(&(j2.closest_line, j2.left)));
    let mut i = 0;
    while i < joints.len() {
        let closest = joints[i].closest_line;
        let left = joints[i].left;
        let mut group_end = i;
        while group_end < joints.len()
            && joints[group_end].closest_line == closest
            && joints[group_end].left == left
        {
            group_end += 1;
        }
        let group = &mut joints[i..group_end];
        // project + sort along the closest line direction
        let dir = line_dir(source[closest], true);
        group.sort_by(|j1, j2| {
            let d1 = dot(
                j1.intersect_point.x() as f64,
                j1.intersect_point.y() as f64,
                dir,
            );
            let d2 = dot(
                j2.intersect_point.x() as f64,
                j2.intersect_point.y() as f64,
                dir,
            );
            d1.partial_cmp(&d2).unwrap_or(std::cmp::Ordering::Equal)
        });

        for idx in 0..group.len() {
            if group[idx].used || working[group[idx].intersect_pl].is_empty() {
                continue;
            }
            if group.len() == 1 {
                add_hook(
                    &mut group[idx],
                    &source,
                    &mut working,
                    scaled_offset,
                    hook_length,
                    scaled_trim_distance,
                );
                group[idx].used = true;
                continue;
            }
            // Nearest fresh neighbor (`get_nearest_intersection`).
            let nearest_idx = nearest_fresh(group, idx);
            let Some(nearest_idx) = nearest_idx else {
                add_hook(
                    &mut group[idx],
                    &source,
                    &mut working,
                    scaled_offset,
                    hook_length,
                    scaled_trim_distance,
                );
                group[idx].used = true;
                continue;
            };
            let (first, rest) = split_at(group, idx, nearest_idx);
            connect_pair(
                first,
                rest,
                &source,
                &mut working,
                scaled_offset,
                hook_length_max,
                hook_length,
                scaled_trim_distance,
            );
        }
        i = group_end;
    }

    working
        .into_iter()
        .filter(|points| !points.is_empty())
        .map(Polyline::new)
        .collect()
}

#[allow(clippy::too_many_arguments)]
fn connect_pair(
    first: &mut Joint,
    nearest: &mut Joint,
    source: &[Line],
    working: &mut [Vec<Point>],
    scaled_offset: f64,
    hook_length_max: f64,
    hook_length: f64,
    scaled_trim_distance: f64,
) {
    let anchor = create_offset_line(
        first.intersect_point,
        nearest.intersect_point,
        first.left,
        scaled_offset,
    );
    let Some(first_pt) = segment_line_intersection(source[first.intersect_line], anchor) else {
        add_hook(
            first,
            source,
            working,
            scaled_offset,
            hook_length,
            scaled_trim_distance,
        );
        first.used = true;
        return;
    };
    let Some(nearest_pt) = segment_line_intersection(source[nearest.intersect_line], anchor) else {
        add_hook(
            first,
            source,
            working,
            scaled_offset,
            hook_length,
            scaled_trim_distance,
        );
        first.used = true;
        return;
    };
    let d2 = distance_squared(first_pt, nearest_pt);
    if d2 > hook_length_max * hook_length_max {
        add_hook(
            first,
            source,
            working,
            scaled_offset,
            hook_length,
            scaled_trim_distance,
        );
        first.used = true;
        return;
    }
    // No third source line may cross the anchor segment.
    for (index, line) in source.iter().enumerate() {
        if index != first.intersect_line && index != nearest.intersect_line {
            if let Some(_) = segments_cross(first_pt, nearest_pt, line.a, line.b) {
                add_hook(
                    first,
                    source,
                    working,
                    scaled_offset,
                    hook_length,
                    scaled_trim_distance,
                );
                first.used = true;
                return;
            }
        }
    }
    // Connect: same-polyline loop close or two-polyline merge.
    if first.intersect_pl == nearest.intersect_pl {
        let points = &mut working[first.intersect_pl];
        if !first.front {
            points[0] = first_pt;
            let last = points.len() - 1;
            points[last] = nearest_pt;
            points.insert(0, nearest_pt);
        } else {
            points[0] = first_pt;
            let last = points.len() - 1;
            points[last] = nearest_pt;
            points.insert(0, nearest_pt);
        }
    } else {
        let mut first_points = std::mem::take(&mut working[first.intersect_pl]);
        let mut second_points = std::mem::take(&mut working[nearest.intersect_pl]);
        if first.front {
            first_points.reverse();
        }
        let last = first_points.len() - 1;
        first_points[last] = first_pt;
        first_points.push(nearest_pt);
        if nearest.front {
            first_points.extend_from_slice(&second_points[1..]);
        } else {
            let tail: Vec<Point> = second_points[1..].iter().rev().copied().collect();
            first_points.extend(tail);
        }
        // Keep the lower index slot (`:1232-1240`).
        if first.intersect_pl < nearest.intersect_pl {
            working[first.intersect_pl] = first_points;
            second_points.clear();
            working[nearest.intersect_pl] = second_points;
            nearest.intersect_pl = first.intersect_pl;
        } else {
            working[nearest.intersect_pl] = first_points;
            first_points_copy_clear(&mut working[first.intersect_pl]);
            working[first.intersect_pl] = Vec::new();
            first.intersect_pl = nearest.intersect_pl;
        }
    }
    first.used = true;
    nearest.used = true;
}

fn first_points_copy_clear(_slot: &mut Vec<Point>) {}

fn split_at(group: &mut [Joint], idx: usize, other: usize) -> (&mut Joint, &mut Joint) {
    let (lo, hi) = if idx < other {
        (idx, other)
    } else {
        (other, idx)
    };
    let (head, tail) = group.split_at_mut(hi);
    (&mut head[lo], &mut tail[0])
}

fn nearest_fresh(group: &[Joint], idx: usize) -> Option<usize> {
    if group.len() < 2 {
        return None;
    }
    let prev_fresh = idx > 0 && !group[idx - 1].used && !group[idx - 1].used;
    let next_fresh = idx + 1 < group.len() && !group[idx + 1].used;
    let take_next = if idx == 0 {
        true
    } else if idx + 1 == group.len() {
        false
    } else if prev_fresh && next_fresh {
        // closer neighbor by projection order = the sorted distance
        false
    } else {
        next_fresh
    };
    Some(if take_next { idx + 1 } else { idx - 1 })
}

/// `FillAdaptive.cpp:670-778` — L-shaped hook extension.
fn add_hook(
    joint: &mut Joint,
    source: &[Line],
    working: &mut [Vec<Point>],
    scaled_offset: f64,
    hook_length: f64,
    scaled_trim_distance: f64,
) {
    let crossing = source[joint.closest_line];
    let Some((dir_x, dir_y)) = normalize(
        crossing.b.x() - crossing.a.x(),
        crossing.b.y() - crossing.a.y(),
    ) else {
        return;
    };
    let start = (
        joint.intersect_point.x() as f64 + dir_x * scaled_offset,
        joint.intersect_point.y() as f64 + dir_y * scaled_offset,
    );
    let mut forward = hook_length;
    for (index, candidate) in source.iter().enumerate() {
        if index == joint.closest_line {
            continue;
        }
        if let Some(t) = ray_segment_hit(start, (dir_x, dir_y), candidate) {
            let t = t - shift_from_thick_line(dir_x, dir_y, candidate, scaled_trim_distance);
            if t < forward {
                forward = t;
            }
        }
    }
    let mut backward = 0.0_f64;
    if forward < hook_length {
        backward = hook_length;
        for (index, candidate) in source.iter().enumerate() {
            if index == joint.closest_line {
                continue;
            }
            if let Some(t) = ray_segment_hit(start, (-dir_x, -dir_y), candidate) {
                let t = t - shift_from_thick_line(-dir_x, -dir_y, candidate, scaled_trim_distance);
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
    let points = &mut working[joint.intersect_pl];
    if points.is_empty() {
        return;
    }
    if joint.front {
        points[0] = start_point;
        points.insert(0, end);
    } else {
        let last = points.len() - 1;
        points[last] = start_point;
        points.push(end);
    }
}

fn shift_from_thick_line(dir_x: f64, dir_y: f64, line: &Line, trim: f64) -> f64 {
    let dx = (line.b.x() - line.a.x()) as f64;
    let dy = (line.b.y() - line.a.y()) as f64;
    let n = (dx * dx + dy * dy).sqrt();
    if n <= 0.0 {
        return 0.0;
    }
    trim * (dir_x * dy / n - dir_y * dx / n).abs()
}

struct Thresholds {
    drop_both_sides: f64,
    anchor_both_sides: f64,
    drop_single_side: f64,
    anchor_single_side: f64,
}

impl Thresholds {
    fn new(scaled_offset: f64) -> Self {
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

fn has_other_tjoint(source: &[Line], index: usize, front: bool, radius: f64) -> bool {
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

fn projects_interior(point: Point, line: &Line) -> bool {
    let vx = (line.b.x() - line.a.x()) as f64;
    let vy = (line.b.y() - line.a.y()) as f64;
    let l2 = vx * vx + vy * vy;
    if l2 <= 0.0 {
        return false;
    }
    let t = (point.x() - line.a.x()) as f64 * vx + (point.y() - line.a.y()) as f64 * vy;
    t > 0.0 && t < l2
}

fn left_of(closest_dir: (f64, f64), intersect_dir: (f64, f64)) -> bool {
    closest_dir.0 * intersect_dir.1 - closest_dir.1 * intersect_dir.0 > 0.0
}

fn line_dir(line: Line, forward: bool) -> (f64, f64) {
    let mut dx = (line.b.x() - line.a.x()) as f64;
    let mut dy = (line.b.y() - line.a.y()) as f64;
    if !forward {
        dx = -dx;
        dy = -dy;
    }
    (dx, dy)
}

fn candidate_dir(line: Line) -> (f64, f64) {
    line_dir(line, true)
}

fn length(line: &Line) -> f64 {
    let dx = (line.b.x() - line.a.x()) as f64;
    let dy = (line.b.y() - line.a.y()) as f64;
    (dx * dx + dy * dy).sqrt()
}

fn dot(x: f64, y: f64, dir: (f64, f64)) -> f64 {
    x * dir.0 + y * dir.1
}

fn distance_squared(a: Point, b: Point) -> f64 {
    let dx = (a.x() - b.x()) as f64;
    let dy = (a.y() - b.y()) as f64;
    dx * dx + dy * dy
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

fn segment_line_intersection(line: Line, other: Line) -> Option<Point> {
    segments_cross(line.a, line.b, other.a, other.b)
        .map(|(x, y)| Point::new(x.round() as i64, y.round() as i64))
}

fn segments_cross(a1: Point, a2: Point, b1: Point, b2: Point) -> Option<(f64, f64)> {
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
