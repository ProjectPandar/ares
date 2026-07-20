use crate::geometry::{ClipperError, EdgeGrid, GridEdge, Point};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ResampledPoint {
    pub(crate) source_index: usize,
    pub(crate) interpolated: bool,
    pub(crate) step_length: f64,
    pub(crate) curve_parameter: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct DistanceThresholds {
    scaled_compensation: f64,
    search_radius: f64,
    scaled_epsilon: f64,
}

impl DistanceThresholds {
    pub(crate) const fn new(
        scaled_compensation: f64,
        search_radius: f64,
        scaled_epsilon: f64,
    ) -> Self {
        Self {
            scaled_compensation,
            search_radius,
            scaled_epsilon,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ClosestHit {
    pub(crate) distance: f64,
    pub(crate) edge: GridEdge,
}

#[derive(Clone, Copy)]
struct PointQuery {
    contour_index: usize,
    point_index: usize,
    point: Point,
    inward: (f64, f64),
}

#[derive(Clone, Copy)]
struct ProjectedHit {
    hit: ClosestHit,
    parameter: f64,
    bisector: (f64, f64),
    segment_length: f64,
}

pub(crate) fn resample_polygon(
    contour: &[Point],
    interval: f64,
) -> Result<(Vec<Point>, Vec<ResampledPoint>), ClipperError> {
    let mut points = Vec::new();
    let mut parameters = Vec::new();
    if contour.len() <= 2 {
        return Ok((points, parameters));
    }

    let mut previous = *contour
        .last()
        .expect("a resampled contour with more than two points has a last point");
    for (source_index, &point) in contour.iter().enumerate() {
        let vector = cast_first_vector(previous, point);
        let length = norm(vector);
        let steps = (length / interval).ceil() as usize;
        let step_length = length / steps as f64;
        for step in 1..steps {
            let parameter = step as f64 / steps as f64;
            let x = previous.x() as f64 + vector.0 * parameter;
            let y = previous.y() as f64 + vector.1 * parameter;
            points.push(Point::new(checked_coord(x)?, checked_coord(y)?));
            parameters.push(ResampledPoint {
                source_index,
                interpolated: true,
                step_length,
                curve_parameter: step_length,
            });
        }
        points.push(point);
        parameters.push(ResampledPoint {
            source_index,
            interpolated: false,
            step_length,
            curve_parameter: step_length,
        });
        previous = point;
    }
    for index in 1..parameters.len() {
        parameters[index].curve_parameter += parameters[index - 1].curve_parameter;
    }
    Ok((points, parameters))
}

pub(crate) fn filtered_contour_distances(
    grid: &EdgeGrid,
    contour_index: usize,
    contour: &[Point],
    parameters: &[ResampledPoint],
    thresholds: DistanceThresholds,
) -> Result<Vec<f32>, ClipperError> {
    Ok(
        filtered_closest_hits(grid, contour_index, contour, parameters, thresholds)?
            .into_iter()
            .map(|hit| {
                hit.map_or(thresholds.search_radius, |hit| {
                    hit.distance.min(thresholds.search_radius)
                }) as f32
            })
            .collect(),
    )
}

pub(crate) fn filtered_closest_hits(
    grid: &EdgeGrid,
    contour_index: usize,
    contour: &[Point],
    parameters: &[ResampledPoint],
    thresholds: DistanceThresholds,
) -> Result<Vec<Option<ClosestHit>>, ClipperError> {
    if contour.len() <= 2 {
        return Ok(Vec::new());
    }

    let radius = checked_coord(thresholds.search_radius)?;
    let mut hits = Vec::with_capacity(contour.len());
    for (point_index, &point) in contour.iter().enumerate() {
        let query_min = Point::new(
            point
                .x()
                .checked_sub(radius)
                .ok_or(ClipperError::CoordinateOutOfRange)?,
            point
                .y()
                .checked_sub(radius)
                .ok_or(ClipperError::CoordinateOutOfRange)?,
        );
        let query_max = Point::new(
            point
                .x()
                .checked_add(radius)
                .ok_or(ClipperError::CoordinateOutOfRange)?,
            point
                .y()
                .checked_add(radius)
                .ok_or(ClipperError::CoordinateOutOfRange)?,
        );
        let query = PointQuery {
            contour_index,
            point_index,
            point,
            inward: inward_direction(contour, point_index),
        };
        let mut closest: Option<ClosestHit> = None;
        grid.visit_cells_intersecting_box(query_min, query_max, |_, _, edges| {
            let cell_closest = closest_in_cell(grid, edges, query, parameters, thresholds);
            if cell_closest
                .is_some_and(|hit| closest.is_none_or(|current| hit.distance < current.distance))
            {
                closest = cell_closest;
            }
            true
        });
        hits.push(closest);
    }
    Ok(hits)
}

fn closest_in_cell(
    grid: &EdgeGrid,
    edges: &[GridEdge],
    query: PointQuery,
    parameters: &[ResampledPoint],
    thresholds: DistanceThresholds,
) -> Option<ClosestHit> {
    let mut closest: Option<ClosestHit> = None;
    for &edge in edges {
        let candidate = project_hit(grid, edge, query.point);
        let is_closer = closest.is_none_or(|hit| candidate.hit.distance < hit.distance);
        let inward = dot(query.inward, candidate.bisector) > 0.0;
        let eligible = is_closer && inward;
        let accepted = edge.contour_index != query.contour_index
            || (eligible && same_contour_accepts(grid, candidate, query, parameters, thresholds));
        if eligible && accepted {
            closest = Some(candidate.hit);
        }
    }
    closest
}

fn project_hit(grid: &EdgeGrid, edge: GridEdge, point: Point) -> ProjectedHit {
    let (segment_start, segment_end) = grid.segment(edge);
    let segment = vector(segment_start, segment_end);
    let from_start = vector(segment_start, point);
    let length_squared = dot(segment, segment);
    let parameter = if length_squared == 0.0 {
        0.0
    } else {
        (dot(from_start, segment) / length_squared).clamp(0.0, 1.0)
    };
    let foot = (
        segment_start.x() as f64 + parameter * segment.0,
        segment_start.y() as f64 + parameter * segment.1,
    );
    let bisector = (foot.0 - point.x() as f64, foot.1 - point.y() as f64);
    ProjectedHit {
        hit: ClosestHit {
            distance: norm(bisector),
            edge,
        },
        parameter,
        bisector,
        segment_length: length_squared.sqrt(),
    }
}

fn same_contour_accepts(
    grid: &EdgeGrid,
    candidate: ProjectedHit,
    query: PointQuery,
    parameters: &[ResampledPoint],
    thresholds: DistanceThresholds,
) -> bool {
    let edge = candidate.hit.edge;
    let source_contour = grid.contour(edge.contour_index);
    let source_parameter_index = parameters.partition_point(|parameter| {
        parameter.source_index < edge.segment_index
            || (parameter.source_index == edge.segment_index && parameter.interpolated)
    });
    let source_parameter = parameters[source_parameter_index];
    debug_assert_eq!(source_parameter.source_index, edge.segment_index);
    debug_assert!(!source_parameter.interpolated);

    let mut parameter_low = parameters[query.point_index].curve_parameter;
    let mut parameter_high = candidate.parameter * candidate.segment_length;
    if edge.segment_index + 1 < source_contour.len() {
        parameter_high += source_parameter.curve_parameter;
    }
    if parameter_low > parameter_high {
        std::mem::swap(&mut parameter_low, &mut parameter_high);
    }
    let parameter_end = parameters
        .last()
        .expect("a filtered contour has resampling parameters")
        .curve_parameter;
    let distance_along =
        (parameter_high - parameter_low).min(parameter_low + parameter_end - parameter_high);
    let accept_distance = 0.5 * thresholds.scaled_compensation * std::f64::consts::PI;
    if distance_along < accept_distance {
        return false;
    }
    if candidate.hit.distance >= thresholds.search_radius + thresholds.scaled_epsilon {
        return true;
    }

    let inside = if candidate.parameter == 0.0 {
        inside_corner(source_contour, edge.segment_index, query.point)
    } else if candidate.parameter == 1.0 {
        inside_corner(
            source_contour,
            (edge.segment_index + 1) % source_contour.len(),
            query.point,
        )
    } else {
        left_of_segment(source_contour, edge.segment_index, query.point)
    };
    inside && distance_along > 0.6 * std::f64::consts::PI * candidate.hit.distance
}

pub(crate) fn left_of_segment(contour: &[Point], segment_index: usize, opposite: Point) -> bool {
    let start = contour[segment_index];
    let end = contour[(segment_index + 1) % contour.len()];
    cross(vector(start, end), vector(start, opposite)) > 0.0
}

pub(crate) fn inside_corner(contour: &[Point], corner_index: usize, opposite: Point) -> bool {
    let previous = contour[(corner_index + contour.len() - 1) % contour.len()];
    let current = contour[corner_index];
    let next = contour[(corner_index + 1) % contour.len()];
    let incoming = vector(previous, current);
    let outgoing = vector(current, next);
    let left_of_incoming = cross(incoming, vector(previous, opposite)) > 0.0;
    let left_of_outgoing = cross(outgoing, vector(current, opposite)) > 0.0;
    if cross(incoming, outgoing) > 0.0 {
        left_of_incoming && left_of_outgoing
    } else {
        left_of_incoming || left_of_outgoing
    }
}

fn inward_direction(contour: &[Point], index: usize) -> (f64, f64) {
    let previous = contour[(index + contour.len() - 1) % contour.len()];
    let current = contour[index];
    let next = contour[(index + 1) % contour.len()];
    let incoming = vector(previous, current);
    let outgoing = vector(current, next);
    (-incoming.1 - outgoing.1, incoming.0 + outgoing.0)
}

fn vector(from: Point, to: Point) -> (f64, f64) {
    ((to.x() - from.x()) as f64, (to.y() - from.y()) as f64)
}

fn cast_first_vector(from: Point, to: Point) -> (f64, f64) {
    (
        to.x() as f64 - from.x() as f64,
        to.y() as f64 - from.y() as f64,
    )
}

fn dot(left: (f64, f64), right: (f64, f64)) -> f64 {
    left.0 * right.0 + left.1 * right.1
}

fn cross(left: (f64, f64), right: (f64, f64)) -> f64 {
    left.0 * right.1 - left.1 * right.0
}

fn norm(vector: (f64, f64)) -> f64 {
    dot(vector, vector).sqrt()
}

fn checked_coord(value: f64) -> Result<i64, ClipperError> {
    const MIN_COORD: f64 = i64::MIN as f64;
    const MAX_COORD_EXCLUSIVE: f64 = -MIN_COORD;
    if value.is_finite() && (MIN_COORD..MAX_COORD_EXCLUSIVE).contains(&value) {
        Ok(value.trunc() as i64)
    } else {
        Err(ClipperError::CoordinateOutOfRange)
    }
}
