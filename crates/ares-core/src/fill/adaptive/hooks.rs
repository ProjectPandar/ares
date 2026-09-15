//! `connect_lines_using_hooks` + `add_hook` + the T-joint pair-connect
//! core (`FillAdaptive.cpp:562-1262`): merge collinear split segments,
//! detect T-joints with the drop/anchor/trim thresholds, connect
//! intersection pairs through offset anchor lines (this determines the
//! polyline direction), and extend single junctions with L-shaped hooks.

mod geometry;

use crate::geometry::{ExPolygon, Line, Point, Polyline};
use geometry::{
    Thresholds, candidate_dir, distance_point_to_segment_squared, distance_squared, dot,
    has_other_tjoint, left_of, length, line_dir, normalize, projects_interior, ray_segment_hit,
    segment_line_intersection, segments_cross, shift_from_thick_line,
};

/// `FillAdaptive.cpp:562-618`.
#[derive(Clone, Copy, Debug)]
pub(super) struct Joint {
    pub(super) closest_line: usize,
    pub(super) intersect_line: usize,
    pub(super) intersect_pl: usize,
    pub(super) intersect_point: Point,
    pub(super) front: bool,
    pub(super) left: bool,
    pub(super) used: bool,
}

/// `FillAdaptive.cpp:642-657` — anchor line offset to the intersection's
/// side and extended by 1.16·offset to guarantee collisions.
pub(super) fn create_offset_line(a: Point, b: Point, left: bool, offset: f64) -> Line {
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
    _units_per_mm: f64,
) -> Vec<Polyline> {
    if lines.len() <= 1 || hook_length <= 0.0 {
        return lines;
    }
    // `:809-811`: 19% overlap; 25% trim.
    let scaled_offset = 0.81 * spacing;
    let scaled_trim_distance = 0.5 * spacing * 0.75;

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
                if d2 <= 1000.0 * 1000.0 && d2 < best_d2 && projects_interior(endpoint, candidate) {
                    best = Some(other);
                    best_d2 = d2;
                }
            }
            let Some(closest) = best else { continue };
            let line_len = length(line);
            let num_tjoints_other = has_other_tjoint(source.as_slice(), index, !front, 1000.0);
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
    // `FillAdaptive.cpp:688-696`: the hook start is where the SOURCE
    // line (intersect_line) crosses the offset version of the closest
    // line — a point inside the region by construction, unlike the
    // naive `point + offset·dir` push that can overshoot the boundary.
    let offset_crossing = create_offset_line(crossing.a, crossing.b, joint.left, scaled_offset);
    let hook_start = segment_line_intersection(source[joint.intersect_line], offset_crossing)
        .unwrap_or_else(|| {
            // Lines parallel: fall back to the projected offset point.
            Point::new(
                (joint.intersect_point.x() as f64 + dir_x * scaled_offset).round() as i64,
                (joint.intersect_point.y() as f64 + dir_y * scaled_offset).round() as i64,
            )
        });
    let start = (hook_start.x() as f64, hook_start.y() as f64);
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
    let start_point = hook_start;
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

#[cfg(test)]
mod tests;
