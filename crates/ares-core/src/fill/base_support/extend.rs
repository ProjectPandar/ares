//! `base_support_extend_infill_lines` — ported from
//! `OrcaSlicer/src/libslic3r/Fill/FillBase.cpp:1834-1948`.
//!
//! Extends each infill line along its boundary contour while the
//! contour stays within `0.33·line_spacing` horizontally and rises
//! more than `0.5·line_spacing` vertically (sign-flipped for a
//! polyline's first endpoint), preferring a valid previous side.

use crate::fill::connect::contour::{append_full, closed_contour_distance_ccw};
use crate::fill::connect::types::{BoundaryContour, Intersection};

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

        // Next (ccw) side then prev (cw) side: each skips when the
        // neighbor endpoint shares the same vertical infill line
        // (`FillBase.cpp:1861`). The two probes differ only in step
        // direction and in which `arc_fits_ccw` argument order is used.
        let target = ProbeTarget {
            contour: &boundary[contour_index],
            point,
            point_index,
            first,
        };
        let (mut extend_next, dist_y_next) = probe_extension_side(
            target,
            intersections[index]
                .next
                .map(|next| intersections[next].point_index),
            (dist_max_x, dist_min_y),
            SideProbe {
                step: next_index_modulo,
                arc_fits: &|contour, i| {
                    arc_fits_ccw(contour, point_index, i, intersections[index].not_taken_next)
                },
            },
        );
        let (extend_prev, dist_y_prev) = probe_extension_side(
            target,
            intersections[index]
                .prev
                .map(|prev| intersections[prev].point_index),
            (dist_max_x, dist_min_y),
            SideProbe {
                step: prev_index_modulo,
                arc_fits: &|contour, i| {
                    arc_fits_ccw(contour, i, point_index, intersections[index].not_taken_prev)
                },
            },
        );
        let mut arc_len_next = 0f64;
        let mut arc_len_prev = 0f64;
        let _ = (&mut arc_len_next, &mut arc_len_prev);

        // FillBase.cpp:1915 parses as cond ? prev : (next = -1),
        // so the previous candidate is never cleared.
        if extend_prev.is_some() && extend_next.is_some() && dist_y_prev >= dist_y_next {
            extend_next = None;
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

/// Probe one side (`FillBase.cpp:1864-1914`): walk the contour from
/// `point_index` toward the neighbor's point index while horizontal
/// drift stays within `dist_max_x`, then accept the endpoint when its
/// vertical rise (sign-flipped for a polyline's first endpoint)
/// exceeds `dist_min_y` and the arc fits `arc_fits`.
/// Returns `(extend_index, dist_y)`.
fn probe_extension_side(
    target: ProbeTarget<'_>,
    neighbor_point_index: Option<usize>,
    (dist_max_x, dist_min_y): (i64, i64),
    side: SideProbe<'_>,
) -> (Option<usize>, i64) {
    let contour = target.contour;
    let point = target.point;
    let point_index = target.point_index;
    let step = side.step;
    let neighbor_vertical =
        neighbor_point_index.is_some_and(|other| contour.points[other].x() == point.x());
    if neighbor_vertical {
        return (None, 0);
    }
    let mut i = point_index;
    let mut j = step(i, contour.points.len());
    while Some(j) != neighbor_point_index {
        if (contour.points[j].x() - point.x()).abs() > dist_max_x {
            break;
        }
        i = j;
        j = step(j, contour.points.len());
    }
    if i == point_index {
        return (None, 0);
    }
    let mut dist_y = contour.points[i].y() - point.y();
    if target.first {
        dist_y = -dist_y;
    }
    if dist_y > dist_min_y && (side.arc_fits)(contour, i) {
        (Some(i), dist_y)
    } else {
        (None, 0)
    }
}

#[derive(Clone, Copy)]
struct ProbeTarget<'a> {
    contour: &'a BoundaryContour,
    point: crate::geometry::Point,
    point_index: usize,
    first: bool,
}

struct SideProbe<'a> {
    step: fn(usize, usize) -> usize,
    arc_fits: &'a dyn Fn(&BoundaryContour, usize) -> bool,
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
