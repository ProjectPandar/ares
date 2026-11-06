//! Boundary resampling — the polygon/contour resampler and the inner
//! offset walker feeding `Boundary` grids (`avoid_crossing` family).

use super::{MITER_LIMIT, contours_bounds};
use crate::geometry::{
    ClipperError, Coord, CoordinateScale, EdgeGrid, ExPolygon, JoinType, Point, Polygon,
    offset_paths, union_expolygons,
};
use crate::project_slice::elephant_foot::distance::{
    DistanceThresholds, ResampledPoint, filtered_contour_distances,
};

pub(super) fn resample_polygon(
    polygon: &[Point],
    dist_from_vertex: f64,
    max_allowed_distance: f64,
) -> Result<Vec<Point>, ClipperError> {
    // `resample_polygon` (AvoidCrossingPerimeters.cpp:813–828) truncates
    // offset and inserted points, with the middle vector computed as integers.
    let coordinate = |value: f64| -> Result<Coord, ClipperError> { Ok(value as Coord) };
    let mut resampled = Vec::with_capacity(3 * polygon.len());
    for (point_index, &point) in polygon.iter().enumerate() {
        resampled.push(point);
        let next = polygon[(point_index + 1) % polygon.len()];
        let line = ((next.x() - point.x()) as f64, (next.y() - point.y()) as f64);
        let line_length = (line.0 * line.0 + line.1 * line.1).sqrt();
        if line_length == 0.0 {
            continue;
        }
        let offset = (
            line.0 / line_length * dist_from_vertex,
            line.1 / line_length * dist_from_vertex,
        );
        let offset_coordinate = (coordinate(offset.0)?, coordinate(offset.1)?);
        let moves = offset_coordinate.0 != 0 || offset_coordinate.1 != 0;
        if line_length > 2.0 * dist_from_vertex && moves {
            resampled.push(Point::new(
                point.x() + offset_coordinate.0,
                point.y() + offset_coordinate.1,
            ));
            let middle = (
                (next.x() - point.x() - 2 * offset_coordinate.0) as f64,
                (next.y() - point.y() - 2 * offset_coordinate.1) as f64,
            );
            let middle_length = (middle.0 * middle.0 + middle.1 * middle.1).sqrt();
            if middle_length > max_allowed_distance {
                let parts = (middle_length / max_allowed_distance).ceil() as usize;
                let anchor = resampled.last().copied().expect("just pushed a point");
                resampled.extend(fill_middle(anchor, middle, parts, coordinate)?);
            }
            resampled.push(Point::new(
                next.x() - offset_coordinate.0,
                next.y() - offset_coordinate.1,
            ));
        }
    }
    Ok(resampled)
}

fn fill_middle(
    anchor: Point,
    middle: (f64, f64),
    parts: usize,
    coordinate: impl Fn(f64) -> Result<Coord, ClipperError>,
) -> Result<Vec<Point>, ClipperError> {
    let mut points = Vec::with_capacity(parts.saturating_sub(1));
    for part in 1..parts {
        let parameter = part as f64 / parts as f64;
        points.push(Point::new(
            coordinate(anchor.x() as f64 + middle.0 * parameter)?,
            coordinate(anchor.y() as f64 + middle.1 * parameter)?,
        ));
    }
    Ok(points)
}

/// `inner_offset` (`AvoidCrossingPerimeters.cpp:1014-1098`): a variable-width
/// inward offset that keeps thin regions connected instead of splitting them.
pub(super) fn inner_offset(
    expolygons: &[ExPolygon],
    offset_dis: f64,
    scale: CoordinateScale,
) -> Result<Vec<ExPolygon>, ClipperError> {
    let scaled_epsilon = scale.scaled_delta(1.0e-4);
    let hole_probe = scale.checked_scale(0.1).unwrap_or(100_000) as f32;
    let mut result = Vec::with_capacity(expolygons.len());
    for expolygon in expolygons {
        // Remove too small holes: a 0.1 mm outward offset collapses them.
        let mut ex_poly = expolygon.clone();
        let holes = ex_poly
            .holes()
            .iter()
            .filter(|hole| {
                !offset_paths(&[(*hole).clone()], hole_probe, JoinType::Round, MITER_LIMIT)
                    .unwrap_or_default()
                    .is_empty()
            })
            .cloned()
            .collect::<Vec<_>>();
        if holes.len() != ex_poly.holes().len() {
            ex_poly = ExPolygon::new(ex_poly.contour().clone(), holes);
        }
        let resample = offset_dis / 2.0;
        let tolerance = scale.checked_scale(0.5).unwrap_or(500) as f64;
        let mut ex_poly = resample_expolygon(&ex_poly, resample, tolerance)?;
        // Filter out expolygons smaller than 0.1 mm^2 (bbox estimate).
        let (min, max) = polygon_bounds(ex_poly.contour());
        let filter_extent = scale.checked_scale(0.1).unwrap_or(100_000);
        let width = max.x().saturating_sub(min.x());
        let height = max.y().saturating_sub(min.y());
        if width < filter_extent && height < filter_extent {
            continue;
        }
        let widths = [
            offset_dis / 2.0,
            offset_dis,
            2.0 * offset_dis + scaled_epsilon,
        ];
        'widths: for (width_index, &min_contour_width) in widths.iter().enumerate() {
            let search_radius = 2.0 * (offset_dis + min_contour_width);
            let contours = std::iter::once(ex_poly.contour().points())
                .chain(ex_poly.holes().iter().map(|hole| hole.points()))
                .map(|points| points.to_vec())
                .collect::<Vec<_>>();
            let resolution = (0.7 * search_radius) as Coord;
            if resolution <= 0 {
                continue;
            }
            let (min, max) = contours_bounds(&contours);
            let Ok(grid) = EdgeGrid::new_from_contours(
                contours.iter().map(|contour| contour.as_slice()),
                min,
                max,
                resolution,
            ) else {
                continue;
            };
            let thresholds = DistanceThresholds::new(offset_dis, search_radius, scaled_epsilon);
            let mut offsets = Vec::with_capacity(ex_poly.holes().len() + 1);
            for (contour_index, contour) in contours.iter().enumerate() {
                let parameters = own_parameters(contour);
                let mut distances = filtered_contour_distances(
                    &grid,
                    contour_index,
                    contour,
                    &parameters,
                    thresholds,
                )?;
                map_distances(&mut distances, min_contour_width, offset_dis);
                offsets.push(distances);
            }
            let offset_ex_poly =
                crate::geometry::variable_offset_inner_ex(&ex_poly, &offsets, MITER_LIMIT)?;
            // Accept matching topology immediately; changed topology or split
            // results are only accepted after the last width (1066–1091).
            let unchanged_topology = offset_ex_poly.len() == 1
                && offset_ex_poly[0].holes().len() == ex_poly.holes().len();
            if !unchanged_topology && width_index + 1 < widths.len() {
                continue;
            }
            // max_by keeps the last tie; reverse to retain the source's first.
            let Some((_, selected)) = offset_ex_poly
                .into_iter()
                .map(|candidate| (candidate.area(), candidate))
                .rev()
                .max_by(|left, right| left.0.total_cmp(&right.0))
            else {
                continue;
            };
            ex_poly = selected;
            break 'widths;
        }
        result.push(ex_poly);
    }
    union_expolygons(&result, &[])
}

pub(super) fn resample_expolygon(
    expolygon: &ExPolygon,
    dist_from_vertex: f64,
    max_allowed: f64,
) -> Result<ExPolygon, ClipperError> {
    let contour = Polygon::new(resample_polygon(
        expolygon.contour().points(),
        dist_from_vertex,
        max_allowed,
    )?);
    let mut holes = Vec::with_capacity(expolygon.holes().len());
    for hole in expolygon.holes() {
        let resampled = resample_polygon(hole.points(), dist_from_vertex, max_allowed)?;
        holes.push(Polygon::new(resampled));
    }
    Ok(ExPolygon::new(contour, holes))
}

/// Parameters for `filtered_contour_distances` when the grid contours are the
/// query contours themselves: every point anchors its own segment with the
/// cumulative length as curve parameter.
pub(super) fn own_parameters(contour: &[Point]) -> Vec<ResampledPoint> {
    let mut parameters = Vec::with_capacity(contour.len() + 1);
    let mut total = 0.0_f32;
    for (index, &point) in contour.iter().enumerate() {
        parameters.push(ResampledPoint {
            source_index: index,
            interpolated: false,
            step_length: 0.0,
            curve_parameter: f64::from(total),
        });
        let next = contour[(index + 1) % contour.len()];
        let dx = (next.x() - point.x()) as f64;
        let dy = (next.y() - point.y()) as f64;
        total += (dx * dx + dy * dy).sqrt() as f32;
    }
    // The closed-curve total is a parameter, not another geometry vertex.
    parameters.push(ResampledPoint {
        source_index: contour.len(),
        interpolated: false,
        step_length: 0.0,
        curve_parameter: f64::from(total),
    });
    parameters
}

/// The upstream distance-to-delta mapping (`AvoidCrossingPerimeters.cpp:
/// 1051-1061`), identical to the elephant-foot compensation mapping.
pub(super) fn map_distances(distances: &mut [f32], min_contour_width: f64, offset_dis: f64) {
    let compensated_width = min_contour_width + 2.0 * offset_dis;
    for distance in distances {
        if f64::from(*distance) < min_contour_width {
            *distance = 0.0;
        } else if f64::from(*distance) > compensated_width {
            *distance = -(offset_dis as f32);
        } else {
            *distance = -(*distance - min_contour_width as f32) / 2.0;
        }
    }
}

pub(super) fn polygon_bounds(contour: &Polygon) -> (Point, Point) {
    let first = contour.points()[0];
    let mut min = first;
    let mut max = first;
    for &point in contour.points().iter().skip(1) {
        min = Point::new(min.x().min(point.x()), min.y().min(point.y()));
        max = Point::new(max.x().max(point.x()), max.y().max(point.y()));
    }
    (min, max)
}
