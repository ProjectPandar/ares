//! Very-long-arch / cap emission of `connect_base_support` — ported
//! from `OrcaSlicer/src/libslic3r/Fill/FillBase.cpp:2648-2691`, plus
//! `Polyline::clip_start` / `clip_end` (`Polyline.cpp:52-101`).

use super::arches::{SCALED_EPSILON, SupportArcCost};
use crate::fill::connect::contour::append_limited;
use crate::fill::connect::types::{BoundaryContour, Intersection};
use crate::geometry::{Point, Polyline};

/// Add very long arches and reasonably long caps even if both end
/// points were already consumed (`:2648-2691`).
#[expect(
    clippy::too_many_arguments,
    reason = "the source arc emission keeps its trim scalars explicit"
)]
pub(super) fn emit_very_long_arches(
    boundary: &[BoundaryContour],
    intersections: &mut [Intersection],
    arches: &[SupportArcCost],
    polylines_out: &mut Vec<Polyline>,
    line_half_width: f64,
    line_spacing: f64,
) {
    let cap_cost = 0.5 * line_spacing;
    let cost_veryhigh = line_spacing * 3.0;
    for index in 0..intersections.len() {
        let cost_prev = arches[index * 2].cost;
        let cost_next = arches[index * 2 + 1].cost;
        emit_long_arc(
            boundary,
            intersections,
            index,
            true,
            cost_prev,
            cap_cost,
            cost_veryhigh,
            line_half_width,
            polylines_out,
        );
        emit_long_arc(
            boundary,
            intersections,
            index,
            false,
            cost_next,
            cap_cost,
            cost_veryhigh,
            line_half_width,
            polylines_out,
        );
    }
}

/// One long arc of `:2650-2691`: `prev` selects the clockwise
/// (prev) side, otherwise the counter-clockwise (next) side.
#[expect(
    clippy::too_many_arguments,
    reason = "the source arc emission keeps its trim scalars explicit"
)]
fn emit_long_arc(
    boundary: &[BoundaryContour],
    intersections: &mut [Intersection],
    index: usize,
    prev: bool,
    cost: f64,
    cap_cost: f64,
    cost_veryhigh: f64,
    line_half_width: f64,
    polylines_out: &mut Vec<Polyline>,
) {
    let other = index ^ 1;
    let neighbor = if prev {
        intersections[index].prev
    } else {
        intersections[index].next
    };
    let Some(neighbor) = neighbor else {
        return;
    };
    let self_loop = neighbor == other;
    let threshold = if self_loop { cap_cost } else { cost_veryhigh };
    let not_taken = if prev {
        intersections[index].not_taken_prev
    } else {
        intersections[index].not_taken_next
    };
    if not_taken <= SCALED_EPSILON || cost <= threshold {
        return;
    }
    if prev && !intersections[index].prev_trimmed {
        intersections[index].trim_prev(not_taken - line_half_width);
        intersections[neighbor].trim_next(0.0);
    }
    if !prev && !intersections[index].next_trimmed {
        intersections[index].trim_next(not_taken - line_half_width);
        intersections[neighbor].trim_prev(0.0);
    }
    let not_taken = if prev {
        intersections[index].not_taken_prev
    } else {
        intersections[index].not_taken_next
    };
    if not_taken <= SCALED_EPSILON {
        return;
    }
    let contour_index = intersections[index].contour_index.expect("connected arch");
    let contour = &boundary[contour_index];
    let mut points = vec![contour.points[intersections[index].point_index]];
    append_limited(
        &mut points,
        contour,
        intersections[index].point_index,
        intersections[neighbor].point_index,
        prev,
        not_taken,
        SCALED_EPSILON,
    );
    if prev {
        intersections[index].trim_prev(0.0);
    } else {
        intersections[index].trim_next(0.0);
    }
    clip_start(&mut points, line_half_width);
    if points.len() > 1 {
        polylines_out.push(Polyline::new(points));
    }
}

/// `Polyline::clip_start` (`Polyline.cpp:95-101`) — drop `distance`
/// from the front (reverse + clip_end + reverse).
fn clip_start(points: &mut Vec<Point>, distance: f64) {
    points.reverse();
    clip_end(points, distance);
    if points.len() >= 2 {
        points.reverse();
    }
}

/// `Polyline::clip_end` (`Polyline.cpp:52-91`), fitting-result
/// tracking aside.
fn clip_end(points: &mut Vec<Point>, mut distance: f64) {
    while distance > 0.0 {
        let Some(last_point) = points.pop() else {
            return;
        };
        let Some(previous) = points.last().copied() else {
            return;
        };
        let dx = (previous.x() - last_point.x()) as f64;
        let dy = (previous.y() - last_point.y()) as f64;
        let squared = dx * dx + dy * dy;
        if squared > distance * distance {
            let t = distance / squared.sqrt();
            points.push(Point::new(
                ((1.0 - t) * last_point.x() as f64 + t * previous.x() as f64) as i64,
                ((1.0 - t) * last_point.y() as f64 + t * previous.y() as f64) as i64,
            ));
            return;
        }
        distance -= squared.sqrt();
    }
}
