//! `connect_base_support` main body — ported from
//! `OrcaSlicer/src/libslic3r/Fill/FillBase.cpp:2247-2697`
//! (phases: empty-contour perimeters `:2281-2296`, excess arches
//! `:2298-2324`, line extension `:1834`, then the linking phases in
//! [`super::link`] — vertical-arch consumption, cost-classified
//! selection, zig-zag traversal, left caps, T-joints and very-long
//! arches).
//!
//! Wires together `mark_boundary_segments_overlapping_infill`,
//! `base_support_extend_infill_lines` and `emit_loops_in_band` on
//! the shared connect-module graph.

use super::arches::{emit_empty_contour_perimeters, emit_excess_arches};
use super::caps::emit_very_long_arches;
use super::extend::base_support_extend_infill_lines;
use super::link::LinkPhases;
use super::mark::mark_boundary_segments_overlapping_infill;
use crate::fill::connect::graph::build_working_graph;
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
    let graph = build_working_graph(infill_ordered, boundary_source, bbox, spacing, scale)?;
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
    let mut linker = LinkPhases::new(
        &graph.boundary,
        &mut intersections,
        &mut paths,
        &mut merged_with,
        line_half_width,
        line_spacing,
        min_arch_length,
        trim_length,
    );

    // Vertical-arch consumption (`:2425-2455`).
    linker.consume_vertical_arches();

    // Cost-classified arch selection (`:2463-2502`); the cost table
    // is reused by the T-joint and very-long-arch phases without
    // recomputation (upstream keeps the same `arches` vector).
    let arches = linker.select_support_arches();

    // Zig-zag traversal (`:2577-2592`).
    linker.link_zigzag();

    // Left caps (`:2596-2607`).
    linker.close_left_caps();

    // T-joints (`:2625-2645`).
    linker.connect_t_joints(&arches);

    // Very long arches and reasonably long caps (`:2648-2691`).
    emit_very_long_arches(
        &graph.boundary,
        &mut intersections,
        &arches,
        &mut polylines_out,
        line_half_width,
        line_spacing,
    );

    // Final assembly (`:2693-2697`): the connected chains follow the
    // emitted loops/arches.
    polylines_out.extend(
        paths
            .into_iter()
            .flatten()
            .filter(|points| points.len() > 1)
            .map(Polyline::new),
    );
    Ok(polylines_out)
}

fn paths_as_polylines(paths: &[Option<Vec<Point>>]) -> Vec<Polyline> {
    paths
        .iter()
        .flatten()
        .map(|points| Polyline::new(points.clone()))
        .collect()
}
