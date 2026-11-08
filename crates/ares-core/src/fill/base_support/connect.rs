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

/// Boundary and coordinate adapter for `FillBase.hpp::Fill::connect_base_support`
/// (`FillBase.cpp:2247-2697`); upstream supplies the polygons and bounding box separately.
pub(crate) struct BaseSupportGeometry<'a> {
    pub(crate) boundary_source: &'a [crate::geometry::Polygon],
    pub(crate) bounding_box: BoundingBox,
    pub(crate) scale: crate::geometry::CoordinateScale,
}

pub(crate) fn connect_base_support(
    infill_ordered: Vec<Polyline>,
    BaseSupportGeometry {
        boundary_source,
        bounding_box: bbox,
        scale,
    }: BaseSupportGeometry<'_>,
    spacing: f64,
    params: &crate::fill::FillParams,
) -> Result<Vec<Polyline>, crate::geometry::ClipperError> {
    let density = params.density;
    let scaled_spacing = spacing / scale.factor();
    let scaled_epsilon = scale.scaled_delta(0.0001);
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
        scaled_epsilon,
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
        min_arch_length,
        trim_length,
        scaled_epsilon,
    );

    // Vertical-arch consumption (`:2425-2455`).
    linker.consume_vertical_arches();

    // Cost-classified arch selection (`:2703-2740`); the cost table
    // is reused by the T-joint and very-long-arch phases without
    // recomputation (upstream keeps the same `arches` vector).
    let cost_low = line_spacing * 1.3;
    let cost_high = line_spacing * 2.0;
    let cost_very_high = line_spacing * 3.0;
    let arches = linker.select_support_arches(cost_low, cost_high);

    // Zig-zag traversal (`:2577-2592`).
    linker.link_zigzag();

    // Left caps (`:2596-2607`).
    linker.close_left_caps();

    // T-joints (`:2625-2645`).
    linker.connect_t_joints(&arches, cost_high);

    // Very long arches and reasonably long caps (`:2648-2691`).
    emit_very_long_arches(
        &graph.boundary,
        &mut intersections,
        &arches,
        &mut polylines_out,
        line_half_width,
        line_spacing,
        cost_very_high,
        scaled_epsilon,
    );

    // Final assembly (`:2693-2697`): the connected chains follow the
    // emitted loops/arches.
    let final_paths: Vec<Vec<Point>> = paths
        .into_iter()
        .flatten()
        .filter(|points| points.len() > 1)
        .collect();
    polylines_out.extend(final_paths.into_iter().map(Polyline::new));
    Ok(polylines_out)
}

fn paths_as_polylines(paths: &[Option<Vec<Point>>]) -> Vec<Polyline> {
    paths
        .iter()
        .flatten()
        .map(|points| Polyline::new(points.clone()))
        .collect()
}
