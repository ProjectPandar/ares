//! `connect_lines_using_hooks` + `add_hook` + the T-joint pair-connect
//! core (`FillAdaptive.cpp:562-1262`): detect T-joints with the
//! drop/anchor/trim thresholds, connect intersection pairs through
//! offset anchor lines (this determines the polyline direction), and
//! extend single junctions with L-shaped hooks.

mod geometry;
mod hook;
mod pair;
mod touching;
mod trim;

#[cfg(test)]
mod tests;

use crate::geometry::{ExPolygon, Line, LineDistanceTree, Point, Polyline};
use geometry::{
    Thresholds, candidate_dir, distance_point_to_segment_squared, dot, left_of, line_dir,
    projects_interior,
};
use hook::{HookParams, add_hook, collision_segments};
use pair::{connect_pair, nearest_fresh, resolve_merged, split_at};
use trim::{polyline_length, trim_endpoint};

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

/// `FillAdaptive.cpp:641-647` — translate the offset line by
/// `perp(closest_line.vector().cast<double>().normalized()) *
/// (left ? offset : -offset)` cast to `coord_t` (`perp(v) = (-v.y, v.x)`,
/// `Point.hpp:119-122`), then extend it by `1.16·offset` along its own
/// direction. All casts truncate, matching Eigen's `cast<coord_t>`.
pub(super) fn create_offset_line(
    a: Point,
    b: Point,
    closest_line: Line,
    left: bool,
    offset: f64,
) -> Line {
    let (dx, dy) = translate_by_closest(closest_line, left, offset);
    let a = Point::new(a.x() + dx, a.y() + dy);
    let b = Point::new(b.x() + dx, b.y() + dy);
    // `Line::extend` (`Line.cpp:121-126`): integer offset first, then
    // truncate the per-component products along the offset line itself.
    let (vx, vy) = eigen_normalized(b.x() - a.x(), b.y() - a.y());
    let extend = (offset * 1.16) as i64;
    let ex = (extend as f64 * vx) as i64;
    let ey = (extend as f64 * vy) as i64;
    Line::new(
        Point::new(a.x() - ex, a.y() - ey),
        Point::new(b.x() + ex, b.y() + ey),
    )
}

/// The `create_offset_line` translate vector components
/// (`FillAdaptive.cpp:643-644`).
fn translate_by_closest(closest_line: Line, left: bool, offset: f64) -> (i64, i64) {
    let (vx, vy) = eigen_normalized(
        closest_line.b.x() - closest_line.a.x(),
        closest_line.b.y() - closest_line.a.y(),
    );
    // perp of the unit closest-line direction: (x, y) -> (-y, x).
    let sign = if left { 1.0 } else { -1.0 };
    ((offset * sign * -vy) as i64, (offset * sign * vx) as i64)
}

/// `Eigen::MatrixBase::normalized()` (Eigen 5.0.1 `src/Core/Dot.h`):
/// divide by `sqrt(squaredNorm())` only when the squared norm is positive;
/// a zero vector is returned unchanged.
fn eigen_normalized(x: i64, y: i64) -> (f64, f64) {
    let vx = x as f64;
    let vy = y as f64;
    let squared_norm = vx * vx + vy * vy;
    if squared_norm > 0.0 {
        let norm = squared_norm.sqrt();
        (vx / norm, vy / norm)
    } else {
        (vx, vy)
    }
}

/// `FillAdaptive.cpp:801-1262`.
pub(crate) fn connect_lines_using_hooks(
    lines: Vec<Polyline>,
    boundary: &ExPolygon,
    spacing: f64,
    (hook_length, hook_length_max): (f64, f64),
    units_per_mm: f64,
) -> Vec<Polyline> {
    if lines.len() <= 1 || hook_length <= 0.0 {
        return lines;
    }
    // `:809-811`: 19% overlap; 25% trim. `SCALED_EPSILON` =
    // `scale_(1e-4 mm)` (`libslic3r.h:96`).
    let scaled_offset = f64::from((spacing * 0.81) as f32);
    let scaled_trim_distance = f64::from((spacing * 0.5 * 0.75) as f32);
    let scaled_epsilon = 1e-4 * units_per_mm;

    let mut working: Vec<Vec<Point>> = lines.into_iter().map(Polyline::into_points).collect();
    let touching_pairs = touching::collect(&mut working);
    // `:874-878`: keep cleared collinear slots in the source-line index domain.
    let source: Vec<Line> = working
        .iter()
        .map(|points| {
            if points.is_empty() {
                Line::new(Point::new(0, 0), Point::new(0, 0))
            } else {
                Line::new(points[0], *points.last().unwrap())
            }
        })
        .collect();

    // T-joints (`:887-1010`): per endpoint the nearest line whose
    // interior projects the point; thresholds drop/anchor/trim.
    let thresholds = Thresholds::new(scaled_offset, scaled_epsilon);
    let mut joints: Vec<Joint> = Vec::new();
    for (index, line) in source.iter().enumerate() {
        if working[index].is_empty() {
            continue;
        }
        // `filter_end_point_connections` (`:939-954`), pointer order as slot order.
        let filter_touching = |candidate: Option<usize>| {
            candidate.filter(|&other| {
                touching_pairs
                    .binary_search(&(index.min(other), index.max(other)))
                    .is_err()
            })
        };
        let tjoint_front = filter_touching(nearest_tjoint(&source, &working, index, line.a));
        let tjoint_back = filter_touching(nearest_tjoint(&source, &working, index, line.b));
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
            continue;
        }
        if !anchor {
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
            continue;
        }
        for (tjoint, front) in [(tjoint_front, true), (tjoint_back, false)] {
            let Some(closest) = tjoint else {
                continue;
            };
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
    // `:1001-1008`: remove intersections pointing at a dropped line.
    joints.retain(|joint| !working[joint.closest_line].is_empty());

    // Pair-connect core (`:1102-1262`), grouped per (closest line, left),
    // sorted along the line direction; `merged_with` re-points polylines
    // merged into a lower slot (`:1042-1049`).
    let mut merged_with: Vec<usize> = (0..working.len()).collect();
    touching::merge(&mut working, &touching_pairs, &mut merged_with);
    let segments = collision_segments(&source, boundary);
    let hook_params = HookParams {
        source: &source,
        segments: &segments,
        index: LineDistanceTree::new(&segments),
        offset: scaled_offset,
        length: hook_length,
        maximum_length: hook_length_max,
        trim: scaled_trim_distance,
        epsilon: scaled_epsilon,
    };
    joints.sort_by_key(|j1| (j1.closest_line, j1.left));
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
                add_hook(&mut group[idx], &mut working, &hook_params);
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
                add_hook(&mut group[idx], &mut working, &hook_params);
                group[idx].used = true;
                continue;
            };
            // `:1181-1184`: could_connect requires a fresh neighbor
            // (the selection may still pick a used one).
            if group[nearest_idx].used || working[group[nearest_idx].intersect_pl].is_empty() {
                add_hook(&mut group[idx], &mut working, &hook_params);
                group[idx].used = true;
                continue;
            }
            let (first, rest) = split_at(group, idx, nearest_idx);
            connect_pair(first, rest, &hook_params, &mut working, &mut merged_with);
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
