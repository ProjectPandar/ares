//! `connect_base_support` main body — ported from
//! `OrcaSlicer/src/libslic3r/Fill/FillBase.cpp:2247-2502`
//! (phases: empty-contour perimeters `:2281-2296`, excess arches
//! `:2298-2324`, line extension `:1834`, vertical-arch consumption
//! `:2425-2455`, cost-classified arch selection
//! `evaluate_support_arches` `:2203-2243` + selection `:2463-2502`).
//!
//! Wires together `mark_boundary_segments_overlapping_infill`,
//! `base_support_extend_infill_lines` and `emit_loops_in_band` on
//! the shared connect-module graph.

use super::arches::{emit_empty_contour_perimeters, emit_excess_arches, evaluate_support_arches};
use super::extend::base_support_extend_infill_lines;
use super::mark::mark_boundary_segments_overlapping_infill;
use crate::fill::connect::contour::{
    append_full, append_limited, path_length_along_contour_ccw, take_full_arc,
};
use crate::fill::connect::graph::build_working_graph;
use crate::fill::connect::types::{Intersection, WorkingGraph};
use crate::geometry::{BoundingBox, Point, Polyline};

const SCALED_EPSILON: f64 = 16.0;

#[expect(dead_code, reason = "wired by the raft fills slice")]
pub(super) fn connect_base_support(
    infill_ordered: Vec<Polyline>,
    boundary_source: &[crate::geometry::Polygon],
    bbox: BoundingBox,
    spacing: f64,
    density: f32,
    scale: crate::geometry::CoordinateScale,
) -> Result<Vec<Polyline>, crate::geometry::ClipperError> {
    let scaled_spacing = spacing / scale.factor();
    let mut graph = build_working_graph(infill_ordered, boundary_source, bbox, spacing, scale)?;
    let mut paths = graph.paths.clone();
    let mut intersections = graph.intersections.clone();

    let line_half_width = 0.5 * scaled_spacing;
    let line_spacing = scaled_spacing / f64::from(density);
    let min_arch_length = 1.3 * line_spacing;
    let trim_length = line_half_width * 0.3;

    mark_boundary_segments_overlapping_infill(
        &graph.boundary,
        &mut intersections,
        &paths_as_polylines(&paths),
        scaled_spacing,
        SCALED_EPSILON,
    );

    let mut polylines_out = Vec::new();

    // Empty-contour perimeter loops (`:2281-2296`).
    emit_empty_contour_perimeters(
        &graph.boundary,
        &intersections,
        trim_length,
        line_spacing,
        &mut polylines_out,
    );

    // Excess arches (`:2298-2324`).
    emit_excess_arches(
        &graph.boundary,
        &intersections,
        line_half_width,
        line_spacing,
        &mut polylines_out,
    );

    base_support_extend_infill_lines(
        &mut paths,
        &graph.boundary,
        &mut intersections,
        scaled_spacing,
        density,
    );

    let mut merged_with = (0..paths.len()).collect::<Vec<_>>();

    // Vertical-arch consumption (`:2425-2455`).
    for index in 0..intersections.len() {
        if intersections[index].consumed {
            continue;
        }
        let other = index ^ 1;
        let (prev, next) = (intersections[index].prev, intersections[index].next);
        let can_take_prev = prev.is_some_and(|prev| {
            vertical_dir(&graph, &intersections, index, prev)
                && !intersections[prev].consumed
                && prev != other
        });
        let can_take_next = next.is_some_and(|next| {
            vertical_dir(&graph, &intersections, index, next)
                && !intersections[next].consumed
                && next != other
        });
        if can_take_prev && (!can_take_next || take_vertical_prev(&intersections, index)) {
            let prev = intersections[index].prev.expect("checked above");
            if !intersections[index].prev_trimmed
                || intersections[index].not_taken_prev > min_arch_length
            {
                take_next(
                    &graph.boundary,
                    &mut intersections,
                    &mut paths,
                    &mut merged_with,
                    prev,
                    false,
                    line_half_width,
                    trim_length,
                );
            }
        } else if can_take_next {
            if !intersections[index].next_trimmed
                || intersections[index].not_taken_next > min_arch_length
            {
                take_next(
                    &graph.boundary,
                    &mut intersections,
                    &mut paths,
                    &mut merged_with,
                    index,
                    true,
                    line_half_width,
                    trim_length,
                );
            }
        }
    }

    // Cost-classified arch selection (`:2463-2502`).
    let arches = evaluate_support_arches(&graph.boundary, &intersections, &paths, SCALED_EPSILON);
    let cost_low = line_spacing * 1.3;
    let cost_high = line_spacing * 2.0;
    let mut selected = Vec::new();
    for index in 0..intersections.len() {
        if intersections[index].consumed {
            continue;
        }
        let (cost_prev, cost_next) = (arches[index * 2].cost, arches[index * 2 + 1].cost);
        let (cost_min, cost_max) = if cost_prev < cost_next {
            (cost_prev, cost_next)
        } else {
            (cost_next, cost_prev)
        };
        if cost_max < cost_low || cost_min > cost_high {
            continue;
        }
        if (cost_max - cost_min) / cost_max < 0.25 {
            continue;
        }
        if cost_prev > cost_low {
            selected.push((cost_prev, index, true));
        }
        if cost_next > cost_low {
            selected.push((cost_next, index, false));
        }
    }
    // Take the longest arch first.
    selected.sort_by(|left, right| right.0.partial_cmp(&left.0).expect("finite costs"));
    for (_, index, prev) in selected {
        if intersections[index].consumed {
            continue;
        }
        if prev {
            let prev_index = intersections[index].prev.expect("selected arcs have links");
            take_next(
                &graph.boundary,
                &mut intersections,
                &mut paths,
                &mut merged_with,
                prev_index,
                false,
                line_half_width,
                trim_length,
            );
        } else {
            take_next(
                &graph.boundary,
                &mut intersections,
                &mut paths,
                &mut merged_with,
                index,
                true,
                line_half_width,
                trim_length,
            );
        }
    }

    Ok(paths
        .into_iter()
        .flatten()
        .filter(|points| points.len() > 1)
        .map(Polyline::new)
        .chain(polylines_out)
        .collect())
}

fn paths_as_polylines(paths: &[Option<Vec<Point>>]) -> Vec<Polyline> {
    paths
        .iter()
        .flatten()
        .map(|points| Polyline::new(points.clone()))
        .collect()
}

/// `vertical(dir)`: the neighbor sits on the same vertical infill
/// line (`FillBase.cpp:1342`: same x).
fn vertical_dir(
    graph: &WorkingGraph,
    intersections: &[Intersection],
    index: usize,
    neighbor: usize,
) -> bool {
    let contour = &graph.boundary[intersections[index]
        .contour_index
        .expect("connected intersection")];
    contour.points[intersections[index].point_index].x()
        == contour.points[intersections[neighbor].point_index].x()
}

/// `take_vertical_prev` (`:2353-2359`): prefer the untrimmed side,
/// else the longer contour.
fn take_vertical_prev(intersections: &[Intersection], index: usize) -> bool {
    if intersections[index].prev_trimmed == intersections[index].next_trimmed {
        intersections[index].not_taken_prev > intersections[index].not_taken_next
    } else {
        !intersections[index].prev_trimmed && intersections[index].next_trimmed
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "the source take_next keeps its arc-trimming scalars explicit"
)]
fn take_next(
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
    let (cp1, cp2) = if take_first {
        (index, next)
    } else {
        (next, index)
    };
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
        // polyline1 (take_first) resp. polyline2.
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
            append_limited(
                path,
                contour,
                intersections[start].point_index,
                intersections[end].point_index,
                clockwise,
                1e10,
                SCALED_EPSILON,
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
            paths[polyline_idx2] = Some(merged);
            merged_with[polyline_idx1] = merged_with[polyline_idx2];
        } else {
            paths[polyline_idx2] = Some(merged);
            paths[polyline_idx1] = None;
            merged_with[polyline_idx2] = merged_with[polyline_idx1];
        }
    }
}

fn resolve_merged(merged_with: &[usize], index: usize) -> usize {
    let mut last = WorkingGraph::path_index_for_intersection(index);
    loop {
        let lower = merged_with[last];
        if lower == last {
            return last;
        }
        last = lower;
    }
}
