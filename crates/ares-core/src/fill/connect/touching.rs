// Ports the active occupied-boundary visitor in OrcaSlicer FillBase.cpp:995-1241.

use super::collision::{
    EuclideanInterval, F64Segment, clipped_infill_segments, collision_interval_prefiltered,
    thick_trace_line,
};
use super::contour::closed_contour_distance_ccw;
use super::scale::{coord_from_completed, inflate_bbox_round_delta, scaled_epsilon, scaled_f64};
use super::types::{BoundaryContour, Intersection};
use crate::geometry::{BoundingBox, ClipperError, CoordinateScale, EdgeGrid, GridEdge, Point};

#[expect(
    clippy::too_many_arguments,
    clippy::excessive_nesting,
    reason = "the source visitor preserves path, segment, trace, cell, and edge traversal order"
)]
pub(super) fn mark_boundary_segments_touching_infill(
    boundary: &[BoundaryContour],
    intersections: &mut [Intersection],
    contour_intersections: &[Vec<usize>],
    boundary_bbox: BoundingBox,
    paths: &[Option<Vec<Point>>],
    clip_distance: f64,
    distance_colliding: f64,
    scale: CoordinateScale,
) -> Result<(), ClipperError> {
    debug_assert_eq!(boundary.len(), contour_intersections.len());
    debug_assert!(
        boundary
            .iter()
            .all(|contour| contour.points.len() + 1 == contour.params.len())
    );

    let grid = touching_grid(
        boundary,
        boundary_bbox,
        clip_distance,
        distance_colliding,
        scale,
    )?;
    let epsilon = scaled_epsilon(scale);

    for points in paths.iter().flatten() {
        for infill in clipped_infill_segments(points, clip_distance) {
            for negative_perpendicular in [true, false] {
                let (trace_start, trace_end) =
                    thick_trace_line(infill, distance_colliding, negative_perpendicular)?;
                debug_assert!(grid_contains(&grid, trace_start));
                debug_assert!(grid_contains(&grid, trace_end));
                grid.visit_cells_intersecting_line(trace_start, trace_end, |_, _, cell_edges| {
                    for &edge in cell_edges {
                        mark_edge_overlap(
                            &grid,
                            edge,
                            infill,
                            distance_colliding,
                            epsilon,
                            boundary,
                            intersections,
                            contour_intersections,
                        );
                    }
                    true
                })?;
            }
        }
    }

    Ok(())
}

fn grid_contains(grid: &EdgeGrid, point: Point) -> bool {
    let (min, max) = grid.bounds();
    point.x() >= min.x() && point.x() <= max.x() && point.y() >= min.y() && point.y() <= max.y()
}

fn touching_grid(
    boundary: &[BoundaryContour],
    boundary_bbox: BoundingBox,
    clip_distance: f64,
    distance_colliding: f64,
    scale: CoordinateScale,
) -> Result<EdgeGrid, ClipperError> {
    let bounds = inflate_bbox_round_delta(boundary_bbox, distance_colliding * 1.43)?;
    let resolution =
        coord_from_completed(clip_distance.max(distance_colliding) + scaled_f64(10.0, scale))?;
    EdgeGrid::new_from_contours(
        boundary.iter().map(|contour| contour.points.as_slice()),
        bounds.min,
        bounds.max,
        resolution,
    )
}

#[expect(
    clippy::too_many_arguments,
    reason = "the source collision visitor keeps grid, geometry, and mutable interval state explicit"
)]
fn mark_edge_overlap(
    grid: &EdgeGrid,
    edge: GridEdge,
    infill: F64Segment,
    radius: f64,
    epsilon: f64,
    boundary: &[BoundaryContour],
    intersections: &mut [Intersection],
    contour_intersections: &[Vec<usize>],
) {
    let ordered = &contour_intersections[edge.contour_index];
    if ordered.is_empty() {
        return;
    }

    let (start, end) = grid.segment(edge);
    let Some(interval) = collision_interval_prefiltered(
        F64Segment::from_points(start, end),
        infill,
        radius,
        epsilon,
    ) else {
        return;
    };
    mark_interval(
        &boundary[edge.contour_index],
        edge.segment_index,
        interval,
        intersections,
        ordered,
    );
}

fn mark_interval(
    contour: &BoundaryContour,
    segment_index: usize,
    interval: EuclideanInterval,
    intersections: &mut [Intersection],
    ordered: &[usize],
) {
    debug_assert!(interval.start >= 0.0);
    debug_assert!(interval.end >= 0.0);
    debug_assert!(interval.start <= interval.end);

    let segment_start = contour.params[segment_index];
    let segment_end = contour.params[segment_index + 1];
    let overlap_start = cpp_min(segment_end, segment_start + interval.start);
    let overlap_end = cpp_min(segment_end, segment_start + interval.end);
    let contour_length = *contour
        .params
        .last()
        .expect("a parameterized boundary must contain its total length");

    let (low, high) = intersection_span(intersections, ordered, overlap_start, overlap_end);
    consume_interior(intersections, low, high);

    let low_trim =
        closed_contour_distance_ccw(intersections[low].param, overlap_start, contour_length);
    intersections[low].trim_next(low_trim);
    let high_trim =
        closed_contour_distance_ccw(overlap_end, intersections[high].param, contour_length);
    intersections[high].trim_prev(high_trim);
    debug_assert_eq!(
        intersections[low].next_trimmed,
        intersections[high].prev_trimmed
    );
}

fn intersection_span(
    intersections: &[Intersection],
    ordered: &[usize],
    overlap_start: f64,
    overlap_end: f64,
) -> (usize, usize) {
    if ordered.len() == 1 {
        return (ordered[0], ordered[0]);
    }

    let mut low = lower_bound_wrapping(intersections, ordered, overlap_start);
    let high = lower_bound_wrapping(intersections, ordered, overlap_end);
    if intersections[low].param != overlap_start {
        low = intersections[low]
            .prev
            .expect("connected boundary intersections must be linked");
    }
    debug_assert_ne!(low, high);
    (low, high)
}

fn lower_bound_wrapping(
    intersections: &[Intersection],
    ordered: &[usize],
    parameter: f64,
) -> usize {
    let position = ordered.partition_point(|&index| intersections[index].param < parameter);
    ordered.get(position).copied().unwrap_or(ordered[0])
}

fn consume_interior(intersections: &mut [Intersection], low: usize, high: usize) {
    let mut current = intersections[low]
        .next
        .expect("connected boundary intersections must be linked");
    while current != high {
        let next = intersections[current]
            .next
            .expect("connected boundary intersections must be linked");
        intersections[current].consume_prev();
        intersections[current].consume_next();
        current = next;
    }
}

fn cpp_min(left: f64, right: f64) -> f64 {
    if right < left { right } else { left }
}
