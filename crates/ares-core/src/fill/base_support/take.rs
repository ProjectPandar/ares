//! `take_next` — the arch-consuming connector of `connect_base_support`,
//! ported from `OrcaSlicer/src/libslic3r/Fill/FillBase.cpp:2364-2432`
//! (lambda `take_next`), plus the `merged_with` union-find walk
//! `get_and_update_merged_with` (`:2337-2351`).

use crate::fill::connect::contour::{
    append_full, append_limited, path_length_along_contour_ccw, take_full_arc,
};
use crate::fill::connect::types::{Intersection, WorkingGraph};
use crate::geometry::Point;

const SCALED_EPSILON: f64 = 16.0;

#[expect(
    clippy::too_many_arguments,
    reason = "the source take_next keeps its arc-trimming scalars explicit"
)]
pub(super) fn take_next(
    boundary: &[crate::fill::connect::types::BoundaryContour],
    intersections: &mut [Intersection],
    paths: &mut [Option<Vec<Point>>],
    merged_with: &mut [usize],
    index: usize,
    take_first: bool,
    line_half_width: f64,
    trim_length: f64,
) {
    let next = intersections[index].next.expect("connected arc");
    // Upstream `:2384-2386`: cp1 = the passed index, cp2 = its
    // next_on_contour — ALWAYS, regardless of take_first. The
    // take_first flag only controls the trim direction and which
    // polyline receives the arc, NOT the cp1/cp2 assignment.
    let (cp1, cp2) = (index, next);
    if intersections[if take_first { cp1 } else { cp2 }].consumed {
        return;
    }
    let polyline_idx1 = resolve_merged(merged_with, cp1);
    let polyline_idx2 = resolve_merged(merged_with, cp2);
    let contour_index = intersections[cp1].contour_index.expect("connected");
    let contour = &boundary[contour_index];

    let mut trimmed = if take_first {
        intersections[cp1].next_trimmed
    } else {
        intersections[cp2].prev_trimmed
    };
    if !trimmed {
        trimmed = cp1 == cp2
            || polyline_idx1 == polyline_idx2
            || intersections[if take_first { cp2 } else { cp1 }].consumed;
        if !trimmed {
            let cp1_first = cp1 % 2 == 0;
            let cp1_other = if cp1_first { cp1 + 1 } else { cp1 - 1 };
            trimmed = cp2 == cp1_other;
        }
        if trimmed {
            let length = if cp1 == cp2 {
                *contour.params.last().expect("params exist")
            } else {
                path_length_along_contour_ccw(
                    &intersections[cp1],
                    &intersections[cp2],
                    *contour.params.last().expect("params exist"),
                )
            };
            if take_first {
                intersections[cp1].trim_next((length - trim_length - SCALED_EPSILON).max(0.0));
                intersections[cp2].trim_prev(0.0);
            } else {
                intersections[cp1].trim_next(0.0);
                intersections[cp2].trim_prev((length - trim_length - SCALED_EPSILON).max(0.0));
            }
        }
    }
    if trimmed {
        // `take_limited` with a 1e10 budget (`:2411-2414`), applied to
        // polyline1 (take_first) resp. polyline2; the same-chain arc
        // closes the loop via its `cp_start == cp_end` branch and the
        // front-growth `add_at_start` path (`:578-670`).
        let target_path = if take_first {
            polyline_idx1
        } else {
            polyline_idx2
        };
        let (start, end, clockwise) = if take_first {
            (cp1, cp2, false)
        } else {
            (cp2, cp1, true)
        };
        if let Some(path) = paths[target_path].as_mut() {
            take_limited(
                boundary,
                intersections,
                path,
                start,
                end,
                clockwise,
                1e10,
                line_half_width,
            );
        }
    } else if !intersections[cp1].consumed && !intersections[cp2].consumed {
        let mut polyline1 = paths[polyline_idx1].take().unwrap_or_default();
        let mut polyline2 = paths[polyline_idx2].take().unwrap_or_default();
        if polyline1.first() == Some(&contour.points[intersections[cp1].point_index]) {
            polyline1.reverse();
        }
        if polyline2.last() == Some(&contour.points[intersections[cp2].point_index]) {
            polyline2.reverse();
        }
        let mut merged = polyline1;
        take_full_arc(
            &mut merged,
            &polyline2,
            &contour.points,
            intersections,
            cp1,
            cp2,
            false,
        );
        if polyline_idx2 < polyline_idx1 {
            // Upstream `:2425-2427`: the merged chain lands in polyline2's
            // slot, polyline1 clears, and idx1 follows idx2.
            paths[polyline_idx2] = Some(merged);
            paths[polyline_idx1] = None;
            merged_with[polyline_idx1] = merged_with[polyline_idx2];
        } else {
            // Upstream `:2428-2430`: take() appended into polyline1 —
            // the merged chain stays in polyline1's slot, polyline2
            // clears, and idx2 follows idx1.
            paths[polyline_idx1] = Some(merged);
            paths[polyline_idx2] = None;
            merged_with[polyline_idx2] = merged_with[polyline_idx1];
        }
    }
}

pub(super) fn resolve_merged(merged_with: &[usize], index: usize) -> usize {
    let mut last = WorkingGraph::path_index_for_intersection(index);
    loop {
        let lower = merged_with[last];
        if lower == last {
            return last;
        }
        last = lower;
    }
}

/// `take_limited` (`FillBase.cpp:578-670`): append (or prepend, via
/// the `add_at_start` front-growth path) the contour arc from
/// `cp_start` towards `cp_end`, clamped by the contour lengths not
/// yet taken.
#[expect(
    clippy::too_many_arguments,
    reason = "the source take_limited keeps its arc scalars explicit"
)]
fn take_limited(
    boundary: &[crate::fill::connect::types::BoundaryContour],
    intersections: &mut [Intersection],
    target: &mut Vec<Point>,
    cp_start: usize,
    cp_end: usize,
    clockwise: bool,
    take_max_length: f64,
    line_half_width: f64,
) {
    let could_take = if clockwise {
        !intersections[cp_start].consumed && intersections[cp_start].not_taken_prev > SCALED_EPSILON
    } else {
        !intersections[cp_start].consumed && intersections[cp_start].not_taken_next > SCALED_EPSILON
    };
    if !could_take {
        return;
    }
    let contour_index = intersections[cp_start].contour_index.expect("connected");
    let contour = &boundary[contour_index];
    let add_at_start = target.first() == Some(&contour.points[intersections[cp_start].point_index]);
    let pl_tmp = if add_at_start {
        std::mem::take(target)
    } else {
        Vec::new()
    };
    let length = *contour.params.last().expect("a contour has parameters");
    let mut length_to_go = take_max_length;
    intersections[cp_start].consumed = true;
    if cp_start == cp_end {
        length_to_go = length_to_go.min((length - line_half_width).max(0.0));
        length_to_go = length_to_go.min(if clockwise {
            intersections[cp_start].not_taken_prev
        } else {
            intersections[cp_start].not_taken_next
        });
        intersections[cp_start].consume_prev();
        intersections[cp_start].consume_next();
        if length_to_go > SCALED_EPSILON {
            let index = intersections[cp_start].point_index;
            append_limited(
                target,
                contour,
                index,
                index,
                clockwise,
                length_to_go,
                SCALED_EPSILON,
            );
        }
    } else {
        let mut cp = cp_start;
        while cp != cp_end {
            let neighbor = if clockwise {
                intersections[cp].prev
            } else {
                intersections[cp].next
            };
            let Some(neighbor) = neighbor else {
                break;
            };
            let l = if clockwise {
                crate::fill::connect::contour::closed_contour_distance_cw(
                    intersections[cp].param,
                    intersections[neighbor].param,
                    length,
                )
            } else {
                crate::fill::connect::contour::closed_contour_distance_ccw(
                    intersections[cp].param,
                    intersections[neighbor].param,
                    length,
                )
            };
            length_to_go = length_to_go.min(if clockwise {
                intersections[cp].not_taken_prev
            } else {
                intersections[cp].not_taken_next
            });
            length_to_go = (length_to_go.min(l - line_half_width)).max(0.0);
            if clockwise {
                intersections[cp].consume_prev();
            } else {
                intersections[cp].consume_next();
            }
            if l >= length_to_go {
                if length_to_go > SCALED_EPSILON {
                    if clockwise {
                        intersections[neighbor].trim_next(l - length_to_go);
                    } else {
                        intersections[neighbor].trim_prev(l - length_to_go);
                    }
                    append_limited(
                        target,
                        contour,
                        intersections[cp].point_index,
                        intersections[neighbor].point_index,
                        clockwise,
                        length_to_go,
                        SCALED_EPSILON,
                    );
                }
                break;
            }
            if clockwise {
                intersections[neighbor].trim_next(0.0);
            } else {
                intersections[neighbor].trim_prev(0.0);
            }
            append_full(
                target,
                &contour.points,
                intersections[cp].point_index,
                intersections[neighbor].point_index,
                clockwise,
            );
            length_to_go -= l;
            cp = neighbor;
        }
    }
    if add_at_start {
        target.reverse();
        target.extend(pl_tmp);
    }
}
