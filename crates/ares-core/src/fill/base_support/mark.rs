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

/// SCALED_EPSILON = scale_(EPSILON) = 1e-4 / 1e-6 = 100 lattice units.
const SCALED_EPSILON: f64 = 100.0;

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
            let mut inside = true;
            let mut i = point_index;
            while i != next_point_index {
                let j = next_index_modulo(i, contour.len());
                let segment = F64Segment::from_points(contour[i], contour[j]);
                let distance = point_line_distance_squared(&infill_line, contour[j]);
                if distance >= radius * radius {
                    // Not fully inside the tube: is it colliding?
                    // Upstream `:1375` passes the CONTOUR segment as the
                    // `line` and the INFILL as the `segment` — the
                    // interval is measured along the CONTOUR segment.
                    if let Some(interval) = rounded_thick_segment_collision(
                        segment,
                        infill_line,
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
                    // Same argument order as the next arc (upstream
                    // `:1375`): the CONTOUR segment is the line.
                    if let Some(interval) = rounded_thick_segment_collision(
                        segment,
                        infill_line,
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

#[cfg(test)]
mod overlap_probe {
    #[test]
    fn probe_overlap_walk_on_top_arch() {
        use super::*;
        use crate::fill::connect::graph::build_working_graph;
        use crate::geometry::{CoordinateScale, Polyline};

        let polygon = crate::geometry::Polygon::new(vec![
            crate::geometry::Point::new(-7_650_601, -7_650_601),
            crate::geometry::Point::new(7_650_601, -7_650_601),
            crate::geometry::Point::new(7_650_601, 7_650_601),
            crate::geometry::Point::new(-7_650_601, 7_650_601),
        ]);
        let lines: Vec<Polyline> = (-1..=1)
            .map(|i| {
                let x = i * 537_000;
                Polyline::new(vec![
                    crate::geometry::Point::new(x, -7_650_601),
                    crate::geometry::Point::new(x, 7_650_601),
                ])
            })
            .collect();
        let bbox = crate::geometry::BoundingBox::from_polygon(&polygon).unwrap();
        let graph = build_working_graph(
            lines.clone(),
            &[polygon],
            bbox,
            0.407,
            CoordinateScale::Normal,
        )
        .unwrap();
        let mut intersections = graph.intersections.clone();
        mark_boundary_segments_overlapping_infill(
            &graph.boundary,
            &mut intersections,
            &lines,
            407_086.0,
            100.0,
        );
        for idx in 0..6 {
            let i = &intersections[idx];
            if i.contour_index.is_some() {
                eprintln!(
                    "OVL idx={idx} next_trim={} next_len={:.0} prev_trim={} prev_len={:.0}",
                    i.next_trimmed, i.not_taken_next, i.prev_trimmed, i.not_taken_prev
                );
            }
        }
    }
}

#[cfg(test)]
mod radius_probe {
    #[test]
    fn probe_top_arch_distances() {
        // idx=1 = (-537000, 7650601) top of line 0. Its PREV arch ends
        // at idx=3 = (0, 7650601) top of line 1. Distance of that
        // endpoint to line 0 (the walk's final segment endpoint):
        let line_a = (crate::geometry::Point::new(-537_000, 7_650_601),);
        let line_b = crate::geometry::Point::new(-537_000, -7_650_601);
        let endpoint = crate::geometry::Point::new(0, 7_650_601);
        let d = (endpoint.x() - line_a.0.x()).abs();
        eprintln!(
            "RAD dist from line0 to idx3 endpoint = {} lattice = {} mm; radius = {} mm",
            d,
            d as f64 / 1e6,
            0.5 * (0.407_086_4 + 0.0001)
        );
    }
}

#[cfg(test)]
mod collision_probe {
    #[test]
    fn probe_top_arch_collision() {
        use crate::fill::connect::collision::{F64Segment, rounded_thick_segment_collision};
        // Contour segment: (-537000, 7650601) -> (0, 7650601) (top arch).
        // Infill line 0: (-537000, 7650601) -> (-537000, -7650601).
        let line = F64Segment::from_points(
            crate::geometry::Point::new(-537_000, 7_650_601),
            crate::geometry::Point::new(0, 7_650_601),
        );
        let infill = F64Segment::from_points(
            crate::geometry::Point::new(-537_000, 7_650_601),
            crate::geometry::Point::new(-537_000, -7_650_601),
        );
        let radius = 0.5 * (407_086.4 + 100.0);
        match rounded_thick_segment_collision(line, infill, radius, 100.0) {
            Some(interval) => eprintln!(
                "COLLIDE Some(start={}, end={})",
                interval.start, interval.end
            ),
            None => eprintln!("COLLIDE None"),
        }
    }
}
