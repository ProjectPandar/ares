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
pub(crate) fn connect_base_support(
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
        #[cfg(test)]
        if std::env::var("ARES_ORDER_DEBUG").is_ok() && index < 8 {
            let i = &intersections[index];
            eprintln!(
                "ORDR idx={index} consumed={} prev_trim={} prev_len={:.3} next_trim={} next_len={:.3}",
                i.consumed, i.prev_trimmed, i.not_taken_prev, i.next_trimmed, i.not_taken_next
            );
        }
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
    // Upstream keeps the slice frame UNROTATED (angle applies only to
    // the final points), so the infill lines stay vertical and the
    // left/right contour arches are vertical (same x). Ares rotates the
    // slice by −(angle+π/2) to carry the world direction, which turns
    // those arches horizontal — the equivalent predicate in this frame
    // is equal y (`FillBase.cpp:1342` semantics).
    let contour = &graph.boundary[intersections[index]
        .contour_index
        .expect("connected intersection")];
    contour.points[intersections[index].point_index].y()
        == contour.points[intersections[neighbor].point_index].y()
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
        // polyline1 (take_first) resp. polyline2. In the SAME-CHAIN
        // case (`polyline_idx1 == polyline_idx2`) the arch closes the
        // loop: its start sits at the chain head, not the tail —
        // `append_limited`'s tail assertion does not hold, so append
        // the arch points directly (upstream's take_limited has no
        // such guard; Release just appends).
        let target_path = if take_first {
            polyline_idx1
        } else {
            polyline_idx2
        };
        if polyline_idx1 == polyline_idx2 {
            // Same-chain arch closes the loop: start sits at the chain
            // head. Reverse the chain so the arch appends at the tail
            // (upstream's take()/take_limited pair handles direction
            // implicitly by appending to whichever end matches).
            if let Some(path) = paths[target_path].as_mut() {
                if path.last() != Some(&contour.points[intersections[cp1].point_index]) {
                    path.reverse();
                }
                if path.last() == Some(&contour.points[intersections[cp1].point_index]) {
                    append_full(
                        path,
                        &contour.points,
                        intersections[cp1].point_index,
                        intersections[cp2].point_index,
                        false,
                    );
                }
            }
            intersections[cp1].consume_next();
            intersections[cp2].consume_prev();
            return;
        }
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

#[cfg(test)]
mod vprobe {
    #[test]
    fn probe_vertical_counts() {
        use super::*;
        let polygon = crate::geometry::Polygon::new(vec![
            crate::geometry::Point::new(-7_650_601, -7_650_601),
            crate::geometry::Point::new(7_650_601, -7_650_601),
            crate::geometry::Point::new(7_650_601, 7_650_601),
            crate::geometry::Point::new(-7_650_601, 7_650_601),
        ]);
        // 13 vertical lines at pitch 537000 spanning ±7650601.
        let lines: Vec<Polyline> = (-14..=14)
            .map(|i| {
                let x = i * 537_000;
                Polyline::new(vec![
                    crate::geometry::Point::new(x, -7_000_000),
                    crate::geometry::Point::new(x, 7_000_000),
                ])
            })
            .collect();
        let bbox = crate::geometry::BoundingBox::from_polygon(&polygon).unwrap();
        let out = connect_base_support(
            lines,
            &[polygon],
            bbox,
            0.407,
            0.67,
            crate::geometry::CoordinateScale::Normal,
        )
        .unwrap();
        eprintln!(
            "VPROBE out={} lens={:?}",
            out.len(),
            out.iter().map(|p| p.points().len()).collect::<Vec<_>>()
        );
    }
}

#[cfg(test)]
mod chain_probe {
    #[test]
    fn probe_intersection_chain() {
        use super::*;
        // Square boundary at ±7650601, 27 vertical lines whose endpoints
        // sit EXACTLY on the top/bottom edges.
        let polygon = crate::geometry::Polygon::new(vec![
            crate::geometry::Point::new(-7_650_601, -7_650_601),
            crate::geometry::Point::new(7_650_601, -7_650_601),
            crate::geometry::Point::new(7_650_601, 7_650_601),
            crate::geometry::Point::new(-7_650_601, 7_650_601),
        ]);
        let lines: Vec<Polyline> = (-13..=13)
            .map(|i| {
                let x = i * 537_000;
                Polyline::new(vec![
                    crate::geometry::Point::new(x, -7_650_601),
                    crate::geometry::Point::new(x, 7_650_601),
                ])
            })
            .collect();
        let bbox = crate::geometry::BoundingBox::from_polygon(&polygon).unwrap();
        let graph = crate::fill::connect::graph::build_working_graph(
            lines.clone(),
            &[polygon.clone()],
            bbox,
            0.407,
            crate::geometry::CoordinateScale::Normal,
        )
        .unwrap();
        let connected = graph
            .intersections
            .iter()
            .filter(|i| i.contour_index.is_some())
            .count();
        let with_next = graph
            .intersections
            .iter()
            .filter(|i| i.next.is_some())
            .count();
        eprintln!(
            "CHAIN hits={connected} with_next={with_next} total={}",
            graph.intersections.len()
        );
        let out = connect_base_support(
            lines,
            &[polygon],
            bbox,
            0.407,
            0.67,
            crate::geometry::CoordinateScale::Normal,
        )
        .unwrap();
        eprintln!(
            "CHAIN out={} lens={:?}",
            out.len(),
            out.iter().map(|p| p.points().len()).collect::<Vec<_>>()
        );
    }
}

#[cfg(test)]
mod arch_probe {
    #[test]
    fn probe_every_other_arch() {
        use super::*;
        let polygon = crate::geometry::Polygon::new(vec![
            crate::geometry::Point::new(-7_650_601, -7_650_601),
            crate::geometry::Point::new(7_650_601, -7_650_601),
            crate::geometry::Point::new(7_650_601, 7_650_601),
            crate::geometry::Point::new(-7_650_601, 7_650_601),
        ]);
        let lines: Vec<Polyline> = (-13..=13)
            .map(|i| {
                let x = i * 537_000;
                Polyline::new(vec![
                    crate::geometry::Point::new(x, -7_650_601),
                    crate::geometry::Point::new(x, 7_650_601),
                ])
            })
            .collect();
        let bbox = crate::geometry::BoundingBox::from_polygon(&polygon).unwrap();
        let graph = crate::fill::connect::graph::build_working_graph(
            lines.clone(),
            &[polygon.clone()],
            bbox,
            0.407,
            crate::geometry::CoordinateScale::Normal,
        )
        .unwrap();
        // Dump the vertical-consumption decisions: for each unconsumed
        // intersection, print prev/next candidates and vertical flags.
        for index in 0..graph.intersections.len() {
            let i = &graph.intersections[index];
            let Some(ci) = i.contour_index else { continue };
            let pt = graph.boundary[ci].points[i.point_index];
            let flag = |other: Option<usize>| {
                other.map(|o| {
                    let j = &graph.intersections[o];
                    let p2 = graph.boundary[j.contour_index.unwrap()].points[j.point_index];
                    format!("{}(dy={})", o, p2.y() == pt.y())
                })
            };
            eprintln!(
                "ARCH idx={index} pt=({},{}) prev={:?} next={:?}",
                pt.x(),
                pt.y(),
                flag(i.prev),
                flag(i.next),
            );
        }
    }
}

#[cfg(test)]
mod merge_probe {
    #[test]
    fn probe_same_chain_append() {
        use super::*;
        let polygon = crate::geometry::Polygon::new(vec![
            crate::geometry::Point::new(-7_650_601, -7_650_601),
            crate::geometry::Point::new(7_650_601, -7_650_601),
            crate::geometry::Point::new(7_650_601, 7_650_601),
            crate::geometry::Point::new(-7_650_601, 7_650_601),
        ]);
        // 3 lines only: chain growth over idx 0..5.
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
        let out = connect_base_support(
            lines,
            &[polygon],
            bbox,
            0.407,
            0.67,
            crate::geometry::CoordinateScale::Normal,
        )
        .unwrap();
        for (n, p) in out.iter().enumerate() {
            eprintln!(
                "MERGE #{n} pts={}",
                p.points()
                    .iter()
                    .map(|pt| format!("({},{})", pt.x(), pt.y()))
                    .collect::<Vec<_>>()
                    .join(" ")
            );
        }
    }
}

#[cfg(test)]
mod order_probe {
    #[test]
    fn probe_27_line_order() {
        use super::*;
        let polygon = crate::geometry::Polygon::new(vec![
            crate::geometry::Point::new(-7_650_601, -7_650_601),
            crate::geometry::Point::new(7_650_601, -7_650_601),
            crate::geometry::Point::new(7_650_601, 7_650_601),
            crate::geometry::Point::new(-7_650_601, 7_650_601),
        ]);
        let lines: Vec<Polyline> = (-13..=13)
            .map(|i| {
                let x = i * 537_000;
                Polyline::new(vec![
                    crate::geometry::Point::new(x, -7_650_601),
                    crate::geometry::Point::new(x, 7_650_601),
                ])
            })
            .collect();
        let bbox = crate::geometry::BoundingBox::from_polygon(&polygon).unwrap();
        let out = connect_base_support(
            lines,
            &[polygon],
            bbox,
            0.407,
            0.67,
            crate::geometry::CoordinateScale::Normal,
        )
        .unwrap();
        eprintln!(
            "ORDER27 out={} lens={:?}",
            out.len(),
            out.iter().map(|p| p.points().len()).collect::<Vec<_>>()
        );
    }
}
