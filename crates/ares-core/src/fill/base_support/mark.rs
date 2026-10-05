//! `mark_boundary_segments_overlapping_infill` — ported from
//! `OrcaSlicer/src/libslic3r/Fill/FillBase.cpp:1352-1435`.
//!
//! After the touching pass, some very short boundary segments that
//! fully overlap an infill line remain unmarked; this pass walks
//! each arc (next/ccw and prev/cw) and marks arcs that stay inside
//! the `0.5·(spacing + ε)` tube of their infill line as trimmed.

use crate::fill::connect::collision::{F64Segment, rounded_thick_segment_collision};
use crate::fill::connect::contour::{closed_contour_distance_ccw, closed_contour_distance_cw};
use crate::fill::connect::types::{Intersection, WorkingGraph};
use crate::geometry::{Point, Polyline};

pub(super) fn mark_boundary_segments_overlapping_infill(
    boundary: &[crate::fill::connect::types::BoundaryContour],
    intersections: &mut [Intersection],
    infill: &[Polyline],
    spacing: f64,
    scaled_epsilon: f64,
) {
    let radius = 0.5 * (spacing + scaled_epsilon);
    for index in 0..intersections.len() {
        let Some(contour_index) = intersections[index].contour_index else {
            continue;
        };
        let point_index = intersections[index].point_index;
        let polyline = &infill[WorkingGraph::path_index_for_intersection(index)];
        debug_assert_eq!(polyline.points().len(), 2);
        // Upstream `Linef infill_line{points.front(), points.back()}`
        // — a full LINE for distance tests, but the COLLISION call
        // passes `infill_line.a`/`.b` as the SEGMENT arguments.
        let infill_line = F64Segment::from_points(polyline.points()[0], polyline.points()[1]);
        let contour = boundary[contour_index].points.clone();
        let contour_params = boundary[contour_index].params.clone();

        // Next (ccw) arc.
        let (next, not_taken_next) = {
            let intersection = &intersections[index];
            (intersection.next, intersection.not_taken_next)
        };
        if intersections[index].could_take_next(scaled_epsilon) {
            let Some(next) = next else { continue };
            let next_point_index = intersections[next].point_index;
            let inside = arc_inside_tube(
                ArcWalk {
                    contour: &contour,
                    contour_params: &contour_params,
                    point_index,
                    target_point_index: next_point_index,
                    not_taken: not_taken_next,
                },
                &infill_line,
                radius,
                scaled_epsilon,
                Direction {
                    step: next_index_modulo,
                    arc_distance: closed_contour_distance_ccw,
                },
            );
            if inside && !intersections[next].prev_trimmed {
                intersections[next].trim_prev(0.0);
            }
            if inside {
                intersections[index].trim_next(0.0);
            }
        } else {
            intersections[index].trim_next(0.0);
        }

        // Prev (cw) arc — mirror of the next arc.
        let (prev, not_taken_prev) = {
            let intersection = &intersections[index];
            (intersection.prev, intersection.not_taken_prev)
        };
        if intersections[index].could_take_prev(scaled_epsilon) {
            let Some(prev) = prev else { continue };
            let prev_point_index = intersections[prev].point_index;
            let inside = arc_inside_tube(
                ArcWalk {
                    contour: &contour,
                    contour_params: &contour_params,
                    point_index,
                    target_point_index: prev_point_index,
                    not_taken: not_taken_prev,
                },
                &infill_line,
                radius,
                scaled_epsilon,
                Direction {
                    step: prev_index_modulo,
                    arc_distance: closed_contour_distance_cw,
                },
            );
            if inside && !intersections[prev].next_trimmed {
                intersections[prev].trim_next(0.0);
            }
            if inside {
                intersections[index].trim_prev(0.0);
            }
        } else {
            intersections[index].trim_prev(0.0);
        }
    }
}

struct Direction {
    step: fn(usize, usize) -> usize,
    arc_distance: fn(f64, f64, f64) -> f64,
}

struct ArcWalk<'a> {
    contour: &'a [Point],
    contour_params: &'a [f64],
    point_index: usize,
    target_point_index: usize,
    not_taken: f64,
}

/// Walks the contour arc from `point_index` toward `target_point_index`.
/// Returns `false` when a segment leaves the `radius` tube of
/// `infill_line` before `not_taken` arc length is consumed
/// (`FillBase.cpp:1362-1420`). `step` is `next_index_modulo` (ccw) or
/// `prev_index_modulo` (cw); `arc_distance` is the matching
/// `closed_contour_distance_*`.
fn arc_inside_tube(
    walk: ArcWalk<'_>,
    infill_line: &F64Segment,
    radius: f64,
    scaled_epsilon: f64,
    direction: Direction,
) -> bool {
    let step = direction.step;
    let arc_distance = direction.arc_distance;
    let contour = walk.contour;
    let contour_params = walk.contour_params;
    let point_index = walk.point_index;
    let not_taken = walk.not_taken;
    let mut i = point_index;
    while i != walk.target_point_index {
        let j = step(i, contour.len());
        let segment = F64Segment::from_points(contour[i], contour[j]);
        let distance = point_line_distance_squared(infill_line, contour[j]);
        if distance >= radius * radius {
            // Not fully inside the tube: is it colliding?
            // Upstream `:1375` passes the CONTOUR segment as the
            // `line` and the INFILL as the `segment` — the
            // interval is measured along the CONTOUR segment.
            // Upstream's default-constructed interval remains (0, 0)
            // when the collision function returns false; it still tests len_out.
            let end =
                rounded_thick_segment_collision(segment, *infill_line, radius, scaled_epsilon)
                    .map_or(0.0, |interval| interval.end);
            let len_out = arc_distance(
                contour_params[point_index],
                contour_params[i],
                last(contour_params),
            ) + end;
            if len_out < not_taken {
                // The contour leaves the infill tube before
                // the arc ends: keep this segment.
                return false;
            }
        }
        if arc_distance(
            contour_params[point_index],
            contour_params[j],
            last(contour_params),
        ) >= not_taken
        {
            break;
        }
        i = j;
    }
    true
}

fn last(params: &[f64]) -> f64 {
    params[params.len() - 1]
}

fn next_index_modulo(index: usize, count: usize) -> usize {
    if index + 1 == count { 0 } else { index + 1 }
}

fn prev_index_modulo(index: usize, count: usize) -> usize {
    if index == 0 { count - 1 } else { index - 1 }
}

/// `line_alg::distance_to_squared(line, point)`.
fn point_line_distance_squared(line: &F64Segment, point: Point) -> f64 {
    let (ax, ay) = (line.a.x, line.a.y);
    let (bx, by) = (line.b.x, line.b.y);
    let (px, py) = (point.x() as f64, point.y() as f64);
    let (dx, dy) = (bx - ax, by - ay);
    let length_squared = dx * dx + dy * dy;
    if length_squared == 0.0 {
        let (ex, ey) = (px - ax, py - ay);
        return ex * ex + ey * ey;
    }
    let t = (((px - ax) * dx) + ((py - ay) * dy)) / length_squared;
    let t = t.clamp(0.0, 1.0);
    let (ex, ey) = (px - (ax + t * dx), py - (ay + t * dy));
    ex * ex + ey * ey
}
