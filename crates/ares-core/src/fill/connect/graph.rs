use super::{
    contour::{closed_contour_distance_ccw, contour_parameters},
    scale::{
        coord_from_completed, inflate_bbox_round_delta, scaled_coord_f64, scaled_epsilon,
        scaled_f64,
    },
    touching::mark_boundary_segments_touching_infill,
    types::{BoundaryContour, Intersection, WorkingGraph},
};
use crate::geometry::{
    BoundingBox, ClipperError, CoordinateScale, EdgeGrid, Point, Polygon, Polyline,
    fixed_msvc_sort_by,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct EndpointHit {
    pub(super) contour_index: usize,
    pub(super) segment_index: usize,
    pub(super) t: f64,
    pub(super) endpoint_index: usize,
}

pub(super) fn build_working_graph(
    infill_ordered: Vec<Polyline>,
    boundary_source: &[Polygon],
    boundary_bbox: BoundingBox,
    spacing: f64,
    scale: CoordinateScale,
) -> Result<WorkingGraph, ClipperError> {
    let paths = infill_ordered
        .into_iter()
        .map(|polyline| Some(polyline.into_points()))
        .collect::<Vec<_>>();
    let mut intersections = vec![Intersection::unconnected(); paths.len() * 2];
    let hits = endpoint_hits(&paths, boundary_source, boundary_bbox, scale)?;
    let (boundary, contour_intersections) =
        split_boundary_working_copy(boundary_source, &paths, &hits, &mut intersections);

    let scaled_spacing = scaled_f64(spacing, scale);
    mark_boundary_segments_touching_infill(
        &boundary,
        &mut intersections,
        &contour_intersections,
        boundary_bbox,
        &paths,
        1.7 * scaled_spacing,
        0.8 * scaled_spacing,
        scale,
    )?;

    Ok(WorkingGraph {
        boundary,
        intersections,
        parents: (0..paths.len()).collect(),
        paths,
        line_half_width: 0.5 * scaled_spacing,
    })
}

fn endpoint_hits(
    paths: &[Option<Vec<Point>>],
    boundary_source: &[Polygon],
    boundary_bbox: BoundingBox,
    scale: CoordinateScale,
) -> Result<Vec<EndpointHit>, ClipperError> {
    let epsilon = scaled_epsilon(scale);
    let bounds = inflate_bbox_round_delta(boundary_bbox, epsilon)?;
    let grid = EdgeGrid::new_from_contours(
        boundary_source.iter().map(|polygon| polygon.points()),
        bounds.min,
        bounds.max,
        scaled_coord_f64(10.0, scale)?,
    )?;
    let search_radius = coord_from_completed(epsilon)?;
    let mut hits = Vec::with_capacity(paths.len() * 2);
    for (path_index, path) in paths.iter().enumerate() {
        let path = path.as_ref().expect("a new working path must be present");
        for endpoint_offset in 0..2 {
            let point = if endpoint_offset == 0 {
                *path.first().expect("an infill path must be nonempty")
            } else {
                *path.last().expect("an infill path must be nonempty")
            };
            if let Some(closest) = grid.closest_point_signed_distance(point, search_radius)? {
                debug_assert!(closest.distance <= 3.0);
                hits.push(EndpointHit {
                    contour_index: closest.contour_index,
                    segment_index: closest.segment_index,
                    t: closest.t,
                    endpoint_index: path_index * 2 + endpoint_offset,
                });
            }
        }
    }
    sort_endpoint_hits(&mut hits);
    Ok(hits)
}

pub(super) fn sort_endpoint_hits(hits: &mut [EndpointHit]) {
    fixed_msvc_sort_by(hits, |left, right| {
        left.contour_index < right.contour_index
            || (left.contour_index == right.contour_index
                && (left.segment_index < right.segment_index
                    || (left.segment_index == right.segment_index && left.t < right.t)))
    });
}

#[expect(
    clippy::excessive_nesting,
    reason = "the source copy inserts sorted endpoint hits while walking contours and points"
)]
fn split_boundary_working_copy(
    boundary_source: &[Polygon],
    paths: &[Option<Vec<Point>>],
    hits: &[EndpointHit],
    intersections: &mut [Intersection],
) -> (Vec<BoundaryContour>, Vec<Vec<usize>>) {
    let mut boundary = Vec::with_capacity(boundary_source.len());
    let mut contour_intersections = Vec::with_capacity(boundary_source.len());
    let mut hit_index = 0;

    for (contour_index, source) in boundary_source.iter().enumerate() {
        let mut points = Vec::new();
        let mut intersection_indices = Vec::new();
        for (source_point_index, &source_point) in source.points().iter().enumerate() {
            if points.last() != Some(&source_point) {
                points.push(source_point);
            }
            while hit_index < hits.len()
                && hits[hit_index].contour_index == contour_index
                && hits[hit_index].segment_index == source_point_index
            {
                let hit = hits[hit_index];
                let path = paths[hit.endpoint_index / 2]
                    .as_ref()
                    .expect("a new working path must be present");
                let endpoint = if hit.endpoint_index & 1 == 0 {
                    *path.first().expect("an infill path must be nonempty")
                } else {
                    *path.last().expect("an infill path must be nonempty")
                };
                let mut point_index = 0;
                if source_point_index + 1 < source.points().len()
                    || points.first() != Some(&endpoint)
                {
                    if points.last() != Some(&endpoint) {
                        points.push(endpoint);
                    }
                    point_index = points.len() - 1;
                }
                intersections[hit.endpoint_index] =
                    Intersection::connected(contour_index, point_index);
                intersection_indices.push(hit.endpoint_index);
                hit_index += 1;
            }
        }

        let params = contour_parameters(&points);
        link_and_measure_intersections(&params, &intersection_indices, intersections);
        boundary.push(BoundaryContour { params, points });
        contour_intersections.push(intersection_indices);
    }
    debug_assert_eq!(hit_index, hits.len());
    (boundary, contour_intersections)
}

fn link_and_measure_intersections(
    params: &[f64],
    intersection_indices: &[usize],
    intersections: &mut [Intersection],
) {
    for (position, &intersection_index) in intersection_indices.iter().enumerate() {
        let previous = intersection_indices
            [(position + intersection_indices.len() - 1) % intersection_indices.len()];
        let next = intersection_indices[(position + 1) % intersection_indices.len()];
        let intersection = &mut intersections[intersection_index];
        intersection.prev = Some(previous);
        intersection.next = Some(next);
        intersection.param = params[intersection.point_index];
    }

    let contour_length = *params.last().expect("a contour has parameters");
    for &intersection_index in intersection_indices {
        let intersection = &intersections[intersection_index];
        let previous = intersection
            .prev
            .expect("a connected intersection has a previous link");
        let next = intersection
            .next
            .expect("a connected intersection has a next link");
        let (not_taken_prev, not_taken_next) = if next == intersection_index {
            debug_assert_eq!(previous, intersection_index);
            (contour_length, contour_length)
        } else {
            debug_assert_ne!(previous, intersection_index);
            (
                closed_contour_distance_ccw(
                    intersections[previous].param,
                    intersection.param,
                    contour_length,
                ),
                closed_contour_distance_ccw(
                    intersection.param,
                    intersections[next].param,
                    contour_length,
                ),
            )
        };
        intersections[intersection_index].not_taken_prev = not_taken_prev;
        intersections[intersection_index].not_taken_next = not_taken_next;
    }
}
