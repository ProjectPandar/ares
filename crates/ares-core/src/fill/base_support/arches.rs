//! Arch evaluation and emission phases of `connect_base_support`
//! (`FillBase.cpp:2173-2243` arc costs, `:2281-2296` empty-contour
//! perimeters, `:2298-2324` excess-arch band emission).

use crate::fill::connect::contour::{append_full, append_limited};
use crate::fill::connect::types::Intersection;
use crate::geometry::{Point, Polyline};

/// SupportArcCost (`:2173-2182` + `evaluate_support_arches`
/// `:2203-2243`): the max deviation of each candidate arc from its
#[derive(Clone, Copy)]
/// chord line and its y-band.
pub(super) struct SupportArcCost {
    pub(super) cost: f64,
}

pub(super) fn evaluate_support_arches(
    boundary: &[crate::fill::connect::types::BoundaryContour],
    intersections: &[Intersection],
    paths: &[Option<Vec<Point>>],
    scaled_epsilon: f64,
) -> Vec<SupportArcCost> {
    let mut arches = vec![SupportArcCost { cost: 0.0 }; intersections.len() * 2];
    let mut scratch = Vec::new();
    for index in 0..intersections.len() {
        let Some(contour_index) = intersections[index].contour_index else {
            continue;
        };
        let contour = &boundary[contour_index];
        let point = contour.points[intersections[index].point_index];

        if intersections[index].not_taken_next > scaled_epsilon
            && let Some(next) = intersections[index].next
        {
            scratch.clear();
            scratch.push(point);
            if intersections[index].next_trimmed {
                append_limited(
                    &mut scratch,
                    contour,
                    intersections[index].point_index,
                    intersections[next].point_index,
                    false,
                    intersections[index].not_taken_next,
                    scaled_epsilon,
                );
            } else {
                append_full(
                    &mut scratch,
                    &contour.points,
                    intersections[index].point_index,
                    intersections[next].point_index,
                    false,
                );
            }
            arches[index * 2 + 1].cost = evaluate_support_arch_cost(&scratch);
        }
        if intersections[index].not_taken_prev > scaled_epsilon
            && let Some(prev) = intersections[index].prev
        {
            scratch.clear();
            scratch.push(point);
            if intersections[index].prev_trimmed {
                append_limited(
                    &mut scratch,
                    contour,
                    intersections[index].point_index,
                    intersections[prev].point_index,
                    true,
                    intersections[index].not_taken_prev,
                    scaled_epsilon,
                );
            } else {
                append_full(
                    &mut scratch,
                    &contour.points,
                    intersections[index].point_index,
                    intersections[prev].point_index,
                    true,
                );
            }
            arches[index * 2].cost = evaluate_support_arch_cost(&scratch);
        }
        let _ = paths;
    }
    arches
}

/// `evaluate_support_arch_cost` (`:2184-2201`).
fn evaluate_support_arch_cost(points: &[Point]) -> f64 {
    let front = points[0];
    let back = points[points.len() - 1];
    let (mut ymin, mut ymax) = (front.y(), back.y());
    if ymin > ymax {
        std::mem::swap(&mut ymin, &mut ymax);
    }
    let (ax, ay) = (front.x() as f64, front.y() as f64);
    let (bx, by) = (back.x() as f64, back.y() as f64);
    // `line_alg::distance_to_squared` (`Line.hpp:43-71`), literally: v/va/l2,
    // `l2 == 0.0 -> va.squaredNorm()`, `t = va.dot(v)/l2`, `t <= 0.0 ->
    // va.squaredNorm()`, `t >= 1.0 -> (p-b).squaredNorm()`, otherwise
    // `(t*v - va).squaredNorm()`; plain `x*x + y*y` norms, no fma, no clamp.
    let (vx, vy) = (bx - ax, by - ay);
    let l2 = vx * vx + vy * vy;
    let mut dmax = 0f64;
    for point in points {
        let (px, py) = (point.x() as f64, point.y() as f64);
        let (vax, vay) = (px - ax, py - ay);
        let line_distance = if l2 == 0.0 {
            (vax * vax + vay * vay).sqrt()
        } else {
            let t = (vax * vx + vay * vy) / l2;
            if t <= 0.0 {
                (vax * vax + vay * vay).sqrt()
            } else if t >= 1.0 {
                let (ex, ey) = (px - bx, py - by);
                (ex * ex + ey * ey).sqrt()
            } else {
                let (ex, ey) = (t * vx - vax, t * vy - vay);
                (ex * ex + ey * ey).sqrt()
            }
        };
        dmax = dmax
            .max(line_distance)
            .max((point.y() - ymax) as f64)
            .max((ymin - point.y()) as f64);
    }
    dmax
}

pub(super) fn emit_empty_contour_perimeters(
    boundary: &[crate::fill::connect::types::BoundaryContour],
    intersections: &[Intersection],
    trim_length: f64,
    line_spacing: f64,
    polylines_out: &mut Vec<Polyline>,
) {
    let mut counts = vec![0usize; boundary.len()];
    for intersection in intersections {
        if let Some(contour_index) = intersection.contour_index {
            counts[contour_index] += 1;
        }
    }
    for (index, contour) in boundary.iter().enumerate() {
        if counts[index] == 0
            && *contour.params.last().expect("params exist") > trim_length + 0.5 * line_spacing
        {
            let mut points = contour.points.clone();
            points.push(points[0]);
            let mut polyline = Polyline::new(points);
            clip_polyline_end(&mut polyline, trim_length);
            if polyline.points().len() > 1 {
                polylines_out.push(polyline);
            }
        }
    }
}

fn clip_polyline_end(polyline: &mut Polyline, distance: f64) {
    let mut points = polyline.clone().into_points();
    let mut remaining = distance;
    while points.len() > 1 {
        let last = points.len() - 1;
        // `Polyline::clip_end` (`Polyline.cpp:56-72`): the new endpoint is
        // `last + (prev - last) * (remaining / len)` with Eigen's
        // `.cast<coord_t>()` truncating toward zero.
        let last_point = points[last];
        let previous = points[last - 1];
        let (vx, vy) = (
            (previous.x() - last_point.x()) as f64,
            (previous.y() - last_point.y()) as f64,
        );
        let lsqr = vx * vx + vy * vy;
        if lsqr > remaining * remaining {
            let factor = remaining / lsqr.sqrt();
            points[last] = Point::new(
                (last_point.x() as f64 + vx * factor) as i64,
                (last_point.y() as f64 + vy * factor) as i64,
            );
            *polyline = Polyline::new(points);
            return;
        }
        remaining -= lsqr.sqrt();
        points.pop();
    }
    *polyline = Polyline::new(points);
}

pub(super) fn emit_excess_arches(
    boundary: &[crate::fill::connect::types::BoundaryContour],
    intersections: &[Intersection],
    line_half_width: f64,
    line_spacing: f64,
    polylines_out: &mut Vec<Polyline>,
) {
    for index in 0..intersections.len() {
        let Some(next) = intersections[index].next else {
            continue;
        };
        if !(intersections[index].next_trimmed && intersections[next].prev_trimmed) {
            continue;
        }
        let Some(contour_index) = intersections[index].contour_index else {
            continue;
        };
        let contour = &boundary[contour_index];
        let first = index % 2 == 0;
        let x = contour.points[intersections[index].point_index].x();
        let (left, right) = if first {
            (
                x as f64 + line_half_width,
                x as f64 + line_spacing - line_half_width,
            )
        } else {
            (
                x as f64 - (line_spacing - line_half_width),
                x as f64 - line_half_width,
            )
        };
        let contour_length = *contour.params.last().expect("params exist");
        let mut param_start = intersections[index].param + intersections[index].not_taken_next;
        let mut param_end = intersections[next].param - intersections[next].not_taken_prev;
        if param_start >= contour_length {
            param_start -= contour_length;
        }
        if param_end < 0.0 {
            param_end += contour_length;
        }
        super::emit_loops_in_band(
            left as i64,
            right as i64,
            &contour.points,
            &contour.params,
            param_start,
            param_end,
            0.5 * line_spacing,
            polylines_out,
        );
    }
}
