//! `base_support_extend_infill_lines` — ported from
//! `OrcaSlicer/src/libslic3r/Fill/FillBase.cpp:1834-1948`.
//!
//! Extends each infill line along its boundary contour while the
//! contour stays within `0.33·line_spacing` horizontally and rises
//! more than `0.5·line_spacing` vertically (sign-flipped for a
//! polyline's first endpoint), preferring the longer side.

use crate::fill::connect::contour::{append_full, closed_contour_distance_ccw};
use crate::fill::connect::types::{BoundaryContour, Intersection};
use crate::geometry::Point;

#[expect(dead_code, reason = "wired by connect_base_support in the next slice")]
pub(super) fn base_support_extend_infill_lines(
    paths: &mut [Option<Vec<crate::geometry::Point>>],
    boundary: &[BoundaryContour],
    intersections: &mut [Intersection],
    spacing: f64,
    density: f32,
) {
    let line_spacing = spacing / f64::from(density);
    let dist_max_x = (line_spacing * 0.33) as i64;
    let dist_min_y = (line_spacing * 0.5) as i64;

    for index in 0..intersections.len() {
        let Some(contour_index) = intersections[index].contour_index else {
            continue;
        };
        let point_index = intersections[index].point_index;
        let point = boundary[contour_index].points[point_index];
        let first = index % 2 == 0;

        let mut extend_next = None;
        let mut extend_prev = None;
        let mut dist_y_next = 0i64;
        let mut dist_y_prev = 0i64;
        let mut arc_len_next = 0f64;
        let mut arc_len_prev = 0f64;
        let _ = (&mut arc_len_next, &mut arc_len_prev);

        // Next (ccw) side: skip when the neighbor endpoint shares the
        // same vertical infill line (`FillBase.cpp:1861`).
        let next = intersections[index].next;
        let next_point_index = next.map(|next| intersections[next].point_index);
        let next_vertical = next_point_index
            .is_some_and(|other| boundary[contour_index].points[other].x() == point.x());
        if !next_vertical {
            let contour = &boundary[contour_index];
            let mut i = point_index;
            let mut j = next_index_modulo(i, contour.points.len());
            while Some(j) != next_point_index {
                if (contour.points[j].x() - point.x()).abs() > dist_max_x {
                    break;
                }
                i = j;
                j = next_index_modulo(j, contour.points.len());
            }
            if i != point_index {
                let mut dist_y = contour.points[i].y() - point.y();
                if first {
                    dist_y = -dist_y;
                }
                if dist_y > dist_min_y {
                    let length =
                        arc_fits_ccw(contour, point_index, i, intersections[index].not_taken_next);
                    if length {
                        extend_next = Some(i);
                        dist_y_next = dist_y;
                    }
                }
            }
        }

        // Prev (cw) side.
        let prev = intersections[index].prev;
        let prev_point_index = prev.map(|prev| intersections[prev].point_index);
        let prev_vertical = prev_point_index
            .is_some_and(|other| boundary[contour_index].points[other].x() == point.x());
        if !prev_vertical {
            let contour = &boundary[contour_index];
            let mut i = point_index;
            let mut j = prev_index_modulo(i, contour.points.len());
            while Some(j) != prev_point_index {
                if (contour.points[j].x() - point.x()).abs() > dist_max_x {
                    break;
                }
                i = j;
                j = prev_index_modulo(j, contour.points.len());
            }
            if i != point_index {
                let mut dist_y = contour.points[i].y() - point.y();
                if first {
                    dist_y = -dist_y;
                }
                if dist_y > dist_min_y {
                    let length =
                        arc_fits_ccw(contour, i, point_index, intersections[index].not_taken_prev);
                    if length {
                        extend_prev = Some(i);
                        dist_y_prev = dist_y;
                    }
                }
            }
        }

        // Drop the shorter side (`:1925`: the ternary assigns −1 to
        // the variable with the smaller dist_y).
        if extend_prev.is_some() && extend_next.is_some() {
            if dist_y_prev < dist_y_next {
                extend_prev = None;
            } else {
                extend_next = None;
            }
        }

        let path_index =
            crate::fill::connect::types::WorkingGraph::path_index_for_intersection(index);
        let contour_points = boundary[contour_index].points.clone();
        let contour_params = boundary[contour_index].params.clone();
        if let Some(extend_index) = extend_prev {
            let arc = closed_contour_distance_ccw(
                contour_params[extend_index],
                contour_params[point_index],
                contour_params[contour_params.len() - 1],
            );
            arc_len_prev = arc;
            let Some(points) = paths[path_index].as_mut() else {
                return;
            };
            if first {
                points.reverse();
            }
            append_full(points, &contour_points, point_index, extend_index, true);
            if first {
                points.reverse();
            }
            intersections[index].point_index = extend_index;
            if intersections[index].prev_trimmed {
                intersections[index].not_taken_prev -= arc_len_prev;
            } else {
                let prev = intersections[index]
                    .prev
                    .expect("prev vertical check requires a link");
                let remaining = closed_contour_distance_ccw(
                    contour_params[intersections[prev].point_index],
                    contour_params[extend_index],
                    contour_params[contour_params.len() - 1],
                );
                intersections[index].not_taken_prev = remaining;
                intersections[prev].not_taken_next = remaining;
            }
            intersections[index].trim_next(0.0);
            if let Some(next) = intersections[index].next {
                intersections[next].prev_trimmed = true;
            }
        } else if let Some(extend_index) = extend_next {
            let arc = closed_contour_distance_ccw(
                contour_params[point_index],
                contour_params[extend_index],
                contour_params[contour_params.len() - 1],
            );
            arc_len_next = arc;
            let Some(points) = paths[path_index].as_mut() else {
                return;
            };
            if first {
                points.reverse();
            }
            append_full(points, &contour_points, point_index, extend_index, false);
            if first {
                points.reverse();
            }
            intersections[index].point_index = extend_index;
            intersections[index].trim_prev(0.0);
            if let Some(prev) = intersections[index].prev {
                intersections[prev].next_trimmed = true;
            }
            if intersections[index].next_trimmed {
                intersections[index].not_taken_next -= arc_len_next;
            } else {
                let next = intersections[index]
                    .next
                    .expect("next vertical check requires a link");
                let remaining = closed_contour_distance_ccw(
                    contour_params[extend_index],
                    contour_params[intersections[next].point_index],
                    contour_params[contour_params.len() - 1],
                );
                intersections[index].not_taken_next = remaining;
                intersections[next].not_taken_prev = remaining;
            }
        }
    }
}

/// ccw arc from `point_index` to `extend_index` fits within `limit`.
fn arc_fits_ccw(
    contour: &BoundaryContour,
    point_index: usize,
    extend_index: usize,
    limit: f64,
) -> bool {
    let length = closed_contour_distance_ccw(
        contour.params[point_index],
        contour.params[extend_index],
        contour.params[contour.params.len() - 1],
    );
    length < limit
}

fn next_index_modulo(index: usize, count: usize) -> usize {
    if index + 1 == count { 0 } else { index + 1 }
}

fn prev_index_modulo(index: usize, count: usize) -> usize {
    if index == 0 { count - 1 } else { index - 1 }
}
