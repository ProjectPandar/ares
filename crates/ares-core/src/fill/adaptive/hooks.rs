//! `connect_lines_using_hooks` + `add_hook` + the T-joint pair-connect
//! core (`FillAdaptive.cpp:562-1262`): detect T-joints with the
//! drop/anchor/trim thresholds, connect intersection pairs through
//! offset anchor lines (this determines the polyline direction), and
//! extend single junctions with L-shaped hooks.

mod geometry;
mod pair;

use crate::geometry::{ExPolygon, Line, Point, Polyline};
use geometry::{
    Thresholds, candidate_dir, distance_point_to_segment_squared, dot, left_of, length, line_dir,
    normalize, projects_interior, ray_segment_hit, segment_line_intersection,
    shift_from_thick_line,
};
use pair::{connect_pair, nearest_fresh, resolve_merged, split_at};

/// `FillAdaptive.cpp:562-570`.
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
/// side (`perp(v) = (-v.y, v.x)`, `Point.hpp:119-122`) and extended by
/// `1.16·offset` to guarantee collisions. All casts truncate, matching
/// Eigen's `cast<coord_t>`.
pub(super) fn create_offset_line(a: Point, b: Point, left: bool, offset: f64) -> Line {
    let vx = (b.x() - a.x()) as f64;
    let vy = (b.y() - a.y()) as f64;
    let n = (vx * vx + vy * vy).sqrt();
    let (vx, vy) = if n > 0.0 {
        (vx / n, vy / n)
    } else {
        (1.0, 0.0)
    };
    // perp of the unit closest-line direction: (x, y) -> (-y, x).
    let (px, py) = (-vy, vx);
    let sign = if left { 1.0 } else { -1.0 };
    let dx = (offset * sign * px) as i64;
    let dy = (offset * sign * py) as i64;
    let a = Point::new(a.x() + dx, a.y() + dy);
    let b = Point::new(b.x() + dx, b.y() + dy);
    // `Line::extend` (`Line.cpp:121-126`): integer offset first, then
    // truncate the per-component products.
    let extend = (offset * 1.16) as i64;
    let ex = (extend as f64 * vx) as i64;
    let ey = (extend as f64 * vy) as i64;
    Line::new(
        Point::new(a.x() - ex, a.y() - ey),
        Point::new(b.x() + ex, b.y() + ey),
    )
}

/// `FillAdaptive.cpp:801-1262`.
pub(crate) fn connect_lines_using_hooks(
    lines: Vec<Polyline>,
    _boundary: &ExPolygon,
    spacing: f64,
    hook_length: f64,
    hook_length_max: f64,
    units_per_mm: f64,
) -> Vec<Polyline> {
    if lines.len() <= 1 || hook_length <= 0.0 {
        return lines;
    }
    // `:809-811`: 19% overlap; 25% trim. `SCALED_EPSILON` =
    // `scale_(1e-4 mm)` (`libslic3r.h:96`).
    let scaled_offset = 0.81 * spacing;
    let scaled_trim_distance = 0.5 * spacing * 0.75;
    let scaled_epsilon = 1e-4 * units_per_mm;

    // The immutable 2-point source lines (`lines_src`, `:864-868`).
    let source: Vec<Line> = lines
        .iter()
        .filter(|l| l.points().len() >= 2)
        .map(|l| Line::new(l.points()[0], l.points()[1]))
        .collect();

    // The mutable working set (upstream `lines`, mutated by
    // drop/trim/hooks/connect).
    let mut working: Vec<Vec<Point>> = lines.iter().map(|l| l.points().to_vec()).collect();

    // T-joints (`:887-1010`): per endpoint the nearest line whose
    // interior projects the point; thresholds drop/anchor/trim.
    let thresholds = Thresholds::new(scaled_offset, scaled_epsilon);
    let mut joints: Vec<Joint> = Vec::new();
    for (index, line) in source.iter().enumerate() {
        let tjoint_front = nearest_tjoint(&source, &working, index, line.a);
        let tjoint_back = nearest_tjoint(&source, &working, index, line.b);
        let num_tjoints = usize::from(tjoint_front.is_some()) + usize::from(tjoint_back.is_some());
        if num_tjoints == 0 {
            continue;
        }
        let line_len = polyline_length(&working[index]);
        let (drop, anchor) = if num_tjoints == 1 {
            (
                line_len < thresholds.drop_single_side,
                line_len > thresholds.anchor_single_side,
            )
        } else {
            (
                line_len < thresholds.drop_both_sides,
                line_len > thresholds.anchor_both_sides,
            )
        };
        if drop {
            // `:966-969`: a very short line connected to another infill
            // line is dropped outright.
            working[index].clear();
        } else if anchor {
            for (tjoint, front) in [(tjoint_front, true), (tjoint_back, false)] {
                if let Some(closest) = tjoint {
                    joints.push(Joint {
                        closest_line: closest,
                        intersect_line: index,
                        intersect_pl: index,
                        intersect_point: if front { line.a } else { line.b },
                        front,
                        left: left_of(
                            candidate_dir(source[closest]),
                            line_dir(source[index], front),
                        ),
                        used: false,
                    });
                }
            }
        } else {
            // Trim (`:993-1000`): pull each joint endpoint inward by
            // `1.155·trim` along the current line direction. The front is
            // trimmed first; the back then uses the trimmed front point
            // (upstream holds live references into the polyline).
            trim_endpoint(
                &mut working[index],
                true,
                tjoint_front.is_some(),
                scaled_trim_distance,
            );
            trim_endpoint(
                &mut working[index],
                false,
                tjoint_back.is_some(),
                scaled_trim_distance,
            );
        }
    }
    // `:1001-1008`: remove intersections pointing at a dropped line.
    joints.retain(|joint| !working[joint.closest_line].is_empty());

    // Pair-connect core (`:1102-1262`), grouped per (closest line, left),
    // sorted along the line direction; `merged_with` re-points polylines
    // merged into a lower slot (`:1042-1049`).
    let mut merged_with: Vec<usize> = (0..working.len()).collect();
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
            // `update_merged_polyline` (`:1066-1076`): follow the merge
            // chain, then refresh `front` from the surviving polyline.
            resolve_merged(&mut merged_with, &mut group[idx], &working);
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
            // Resolve both neighbors BEFORE choosing (`:1156-1158`) so
            // their freshness and `front` reflect merged polylines.
            if idx > 0 {
                resolve_merged(&mut merged_with, &mut group[idx - 1], &working);
            }
            if idx + 1 < group.len() {
                resolve_merged(&mut merged_with, &mut group[idx + 1], &working);
            }
            // Nearest neighbor (`get_nearest_intersection`, `:620-637`).
            let nearest_idx = nearest_fresh(group, &working, idx, dir);
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
            // `:1181-1184`: could_connect requires a fresh neighbor
            // (the selection may still pick a used one).
            if group[nearest_idx].used || working[group[nearest_idx].intersect_pl].is_empty() {
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
            let (first, rest) = split_at(group, idx, nearest_idx);
            connect_pair(
                first,
                rest,
                &source,
                &mut working,
                &mut merged_with,
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

/// `has_tjoint` (`:899-925`): the nearest non-dropped line whose interior
/// projects the point, within the 1000-unit T-joint radius.
fn nearest_tjoint(
    source: &[Line],
    working: &[Vec<Point>],
    index: usize,
    point: Point,
) -> Option<usize> {
    let mut best: Option<usize> = None;
    let mut best_d2 = f64::INFINITY;
    for (other, candidate) in source.iter().enumerate() {
        if other == index || working[other].is_empty() {
            continue;
        }
        let d2 = distance_point_to_segment_squared(point, candidate);
        if d2 <= 1000.0 * 1000.0 && d2 < best_d2 && projects_interior(point, candidate) {
            best = Some(other);
            best_d2 = d2;
        }
    }
    best
}

fn polyline_length(points: &[Point]) -> f64 {
    points
        .windows(2)
        .map(|pair| length(&Line::new(pair[0], pair[1])))
        .sum()
}

/// `:993-1000` — move `points`' front (or back) endpoint inward by
/// `1.155·trim` toward the current opposite endpoint (truncating casts).
fn trim_endpoint(points: &mut [Point], front: bool, active: bool, scaled_trim_distance: f64) {
    if !active || points.len() < 2 {
        return;
    }
    let endpoint = if front {
        points[0]
    } else {
        points[points.len() - 1]
    };
    let other = if front {
        points[points.len() - 1]
    } else {
        points[0]
    };
    let dx = (other.x() - endpoint.x()) as f64;
    let dy = (other.y() - endpoint.y()) as f64;
    let n = (dx * dx + dy * dy).sqrt();
    if n <= 0.0 {
        return;
    }
    let step = 1.155 * scaled_trim_distance;
    let mx = (dx / n * step) as i64;
    let my = (dy / n * step) as i64;
    let moved = Point::new(endpoint.x() + mx, endpoint.y() + my);
    if front {
        points[0] = moved;
    } else {
        let last = points.len() - 1;
        points[last] = moved;
    }
}

#[allow(clippy::too_many_arguments)]
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
                (joint.intersect_point.x() as f64 + dir_x * scaled_offset) as i64,
                (joint.intersect_point.y() as f64 + dir_y * scaled_offset) as i64,
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
        (start.0 + length * dir_x) as i64,
        (start.1 + length * dir_y) as i64,
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
