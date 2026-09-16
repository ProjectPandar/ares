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

/// SCALED_EPSILON equivalent in scaled units.
const SCALED_EPSILON: f64 = 16.0;

#[expect(
    clippy::too_many_arguments,
    reason = "the source marker keeps its arc-walk scalars explicit"
)]
pub(super) fn mark_boundary_segments_overlapping_infill(
    boundary: &[crate::fill::connect::types::BoundaryContour],
    intersections: &mut [Intersection],
    infill: &[Polyline],
    spacing: f64,
    scaled_epsilon: f64,
) {
    let radius = 0.5 * (spacing + SCALED_EPSILON);
    for index in 0..intersections.len() {
        let Some(contour_index) = intersections[index].contour_index else {
            continue;
        };
        let point_index = intersections[index].point_index;
        let polyline = &infill[WorkingGraph::path_index_for_intersection(index)];
        debug_assert_eq!(polyline.points().len(), 2);
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
            let mut inside = true;
            let mut i = point_index;
            while i != next_point_index {
                let j = next_index_modulo(i, contour.len());
                let segment = F64Segment::from_points(contour[i], contour[j]);
                let distance = point_line_distance_squared(&infill_line, contour[j]);
                if distance >= radius * radius {
                    // Not fully inside the tube: is it colliding?
                    if let Some(interval) = rounded_thick_segment_collision(
                        infill_line,
                        segment,
                        radius,
                        scaled_epsilon,
                    ) {
                        let len_out = closed_contour_distance_ccw(
                            contour_params[point_index],
                            contour_params[i],
                            last(&contour_params),
                        ) + interval.end;
                        if len_out < not_taken_next {
                            // The contour leaves the infill tube before
                            // the arc ends: keep this segment.
                            inside = false;
                            break;
                        }
                    }
                }
                if closed_contour_distance_ccw(
                    contour_params[point_index],
                    contour_params[j],
                    last(&contour_params),
                ) >= not_taken_next
                {
                    break;
                }
                i = j;
            }
            if inside {
                if !intersections[next].prev_trimmed {
                    intersections[next].trim_prev(0.0);
                }
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
            let mut inside = true;
            let mut i = point_index;
            while i != prev_point_index {
                let j = prev_index_modulo(i, contour.len());
                let segment = F64Segment::from_points(contour[i], contour[j]);
                let distance = point_line_distance_squared(&infill_line, contour[j]);
                if distance >= radius * radius {
                    if let Some(interval) = rounded_thick_segment_collision(
                        infill_line,
                        segment,
                        radius,
                        scaled_epsilon,
                    ) {
                        let len_out = closed_contour_distance_cw(
                            contour_params[point_index],
                            contour_params[i],
                            last(&contour_params),
                        ) + interval.end;
                        if len_out < not_taken_prev {
                            inside = false;
                            break;
                        }
                    }
                }
                if closed_contour_distance_cw(
                    contour_params[point_index],
                    contour_params[j],
                    last(&contour_params),
                ) >= not_taken_prev
                {
                    break;
                }
                i = j;
            }
            if inside {
                if !intersections[prev].next_trimmed {
                    intersections[prev].trim_next(0.0);
                }
                intersections[index].trim_prev(0.0);
            }
        } else {
            intersections[index].trim_prev(0.0);
        }
    }
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

// Silence unused warnings until the caller lands in the next slice.
