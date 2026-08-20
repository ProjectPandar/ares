pub(super) mod distance;
pub(super) mod profile;

use crate::geometry::{
    ClipperError, Coord, CoordinateScale, EdgeGrid, ExPolygon, Point, Polygon,
    append_simplified_expolygon, variable_offset_inner_ex,
};

use distance::{ClosestHit, DistanceThresholds, ResampledPoint};

type ResampleOperation =
    fn(&[Point], f64) -> Result<(Vec<Point>, Vec<ResampledPoint>), ClipperError>;
type DistanceOperation = fn(
    &EdgeGrid,
    usize,
    &[Point],
    &[ResampledPoint],
    DistanceThresholds,
) -> Result<Vec<f32>, ClipperError>;
type HitOperation = fn(
    &EdgeGrid,
    usize,
    &[Point],
    &[ResampledPoint],
    DistanceThresholds,
) -> Result<Vec<Option<ClosestHit>>, ClipperError>;
type MapOperation = fn(&mut [f32], f64, f64);
type SmoothOperation = fn(&[Point], &mut [f32], f32, f32, usize);
type PredicateOperation = fn(&[Point], usize, Point) -> bool;
type CompensationOperation =
    fn(&[ExPolygon], f64, f64, CoordinateScale) -> Result<Vec<ExPolygon>, ClipperError>;

const _: ResampleOperation = distance::resample_polygon;
const _: DistanceOperation = distance::filtered_contour_distances;
const _: HitOperation = distance::filtered_closest_hits;
const _: fn(f64, f64, f64) -> DistanceThresholds = DistanceThresholds::new;
const _: PredicateOperation = distance::left_of_segment;
const _: PredicateOperation = distance::inside_corner;
const _: MapOperation = profile::map_distances_to_compensation;
const _: SmoothOperation = profile::smooth_compensation_banded;
const _: CompensationOperation = compensate_expolygons;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct ElephantFootGeometry {
    minimum_width: f64,
    scaled_compensation: f64,
    compensated_width: f64,
    search_radius: f64,
    scaled_epsilon: f64,
    resample_interval: f64,
    epsilon_coordinate: Coord,
    grid_resolution: Coord,
    smoothing_band: f32,
}

pub(super) struct PreparedOffset {
    expolygon: ExPolygon,
    deltas: Vec<Vec<f32>>,
}

impl PreparedOffset {
    pub(super) fn as_parts(&self) -> (&ExPolygon, &[Vec<f32>]) {
        (&self.expolygon, &self.deltas)
    }
}

pub(super) fn derive_geometry(
    minimum_width_mm: f64,
    compensation_mm: f64,
    scale: CoordinateScale,
) -> Result<ElephantFootGeometry, ClipperError> {
    let factor = scale.factor();
    let minimum_width = minimum_width_mm / factor;
    let scaled_compensation = compensation_mm / factor;
    let compensated_width = minimum_width + 2.0 * scaled_compensation;
    let search_radius = compensated_width + 0.5 * minimum_width;
    let scaled_epsilon = 0.0001 / factor;
    let resample_interval = 0.5 / factor;
    let grid_resolution_value = 0.7 * search_radius;
    let smoothing_band = (0.8 * resample_interval) as f32;
    let area_threshold = compensated_width * compensated_width * 5.0;
    let extent_threshold = compensated_width + scaled_epsilon;
    let narrowed = [
        minimum_width as f32,
        scaled_compensation as f32,
        search_radius as f32,
        smoothing_band,
    ];
    if ![
        minimum_width_mm,
        compensation_mm,
        minimum_width,
        scaled_compensation,
        compensated_width,
        search_radius,
        scaled_epsilon,
        resample_interval,
        grid_resolution_value,
        area_threshold,
        extent_threshold,
    ]
    .into_iter()
    .all(f64::is_finite)
        || !narrowed.into_iter().all(f32::is_finite)
    {
        return Err(ClipperError::CoordinateOutOfRange);
    }

    let epsilon_coordinate = checked_coordinate(scaled_epsilon)?;
    let grid_resolution = checked_coordinate(grid_resolution_value)?;
    if grid_resolution <= 0 {
        return Err(ClipperError::CoordinateOutOfRange);
    }
    Ok(ElephantFootGeometry {
        minimum_width,
        scaled_compensation,
        compensated_width,
        search_radius,
        scaled_epsilon,
        resample_interval,
        epsilon_coordinate,
        grid_resolution,
        smoothing_band,
    })
}

pub(super) fn prepare_offset(
    input: &ExPolygon,
    geometry: ElephantFootGeometry,
) -> Result<PreparedOffset, ClipperError> {
    let mut simplified = Vec::new();
    append_simplified_expolygon(input.clone(), geometry.scaled_epsilon, &mut simplified)?;
    let simplified = simplified
        .into_iter()
        .next()
        .expect("valid elephant-foot input must simplify to an ExPolygon");
    let (bounds_min, bounds_max) = expanded_contour_bounds(&simplified, geometry)?;
    let grid = EdgeGrid::new(
        &simplified,
        bounds_min,
        bounds_max,
        geometry.grid_resolution,
    )?;
    let (contour, contour_deltas) = prepare_contour(&grid, 0, geometry)?;
    let mut holes = Vec::with_capacity(simplified.holes().len());
    let mut deltas = Vec::with_capacity(simplified.holes().len() + 1);
    deltas.push(contour_deltas);
    for contour_index in 1..=simplified.holes().len() {
        let (hole, hole_deltas) = prepare_contour(&grid, contour_index, geometry)?;
        holes.push(hole);
        deltas.push(hole_deltas);
    }
    Ok(PreparedOffset {
        expolygon: ExPolygon::new(contour, holes),
        deltas,
    })
}

pub(super) fn compensate_expolygon(
    input: &ExPolygon,
    geometry: ElephantFootGeometry,
) -> Result<ExPolygon, ClipperError> {
    if is_tiny(input, geometry)? {
        return Ok(input.clone());
    }

    let prepared = prepare_offset(input, geometry)?;
    let (expolygon, deltas) = prepared.as_parts();
    let mut result = variable_offset_inner_ex(expolygon, deltas, 2.0)?;
    Ok(if result.len() == 1 {
        result
            .pop()
            .expect("variable offset result length was checked")
    } else {
        input.clone()
    })
}

pub(super) fn compensate_expolygons(
    input: &[ExPolygon],
    minimum_width_mm: f64,
    compensation_mm: f64,
    scale: CoordinateScale,
) -> Result<Vec<ExPolygon>, ClipperError> {
    let geometry = derive_geometry(minimum_width_mm, compensation_mm, scale)?;
    input
        .iter()
        .map(|expolygon| compensate_expolygon(expolygon, geometry))
        .collect()
}

fn is_tiny(input: &ExPolygon, geometry: ElephantFootGeometry) -> Result<bool, ClipperError> {
    let (min, max) = contour_bounds(input);
    let width = max
        .x()
        .checked_sub(min.x())
        .ok_or(ClipperError::CoordinateOutOfRange)?;
    let height = max
        .y()
        .checked_sub(min.y())
        .ok_or(ClipperError::CoordinateOutOfRange)?;
    let minimum_extent = geometry.compensated_width + geometry.scaled_epsilon;
    let mut area = input.contour().area();
    for hole in input.holes() {
        area -= -hole.area();
    }
    Ok((width as f64) < minimum_extent
        || (height as f64) < minimum_extent
        || area < geometry.compensated_width * geometry.compensated_width * 5.0)
}

fn prepare_contour(
    grid: &EdgeGrid,
    contour_index: usize,
    geometry: ElephantFootGeometry,
) -> Result<(Polygon, Vec<f32>), ClipperError> {
    let (points, parameters) =
        distance::resample_polygon(grid.contour(contour_index), geometry.resample_interval)?;
    let thresholds = DistanceThresholds::new(
        geometry.scaled_compensation,
        geometry.search_radius,
        geometry.scaled_epsilon,
    );
    let mut deltas = distance::filtered_contour_distances(
        grid,
        contour_index,
        &points,
        &parameters,
        thresholds,
    )?;
    profile::map_distances_to_compensation(
        &mut deltas,
        geometry.minimum_width,
        geometry.scaled_compensation,
    );
    profile::smooth_compensation_banded(&points, &mut deltas, geometry.smoothing_band, 0.3, 3);
    Ok((Polygon::new(points), deltas))
}

fn expanded_contour_bounds(
    input: &ExPolygon,
    geometry: ElephantFootGeometry,
) -> Result<(Point, Point), ClipperError> {
    let (min, max) = contour_bounds(input);
    let epsilon = geometry.epsilon_coordinate;
    Ok((
        Point::new(
            min.x()
                .checked_sub(epsilon)
                .ok_or(ClipperError::CoordinateOutOfRange)?,
            min.y()
                .checked_sub(epsilon)
                .ok_or(ClipperError::CoordinateOutOfRange)?,
        ),
        Point::new(
            max.x()
                .checked_add(epsilon)
                .ok_or(ClipperError::CoordinateOutOfRange)?,
            max.y()
                .checked_add(epsilon)
                .ok_or(ClipperError::CoordinateOutOfRange)?,
        ),
    ))
}

fn contour_bounds(input: &ExPolygon) -> (Point, Point) {
    let mut points = input.contour().points().iter().copied();
    let first = points
        .next()
        .expect("valid elephant-foot input must have a contour");
    points.fold((first, first), |(min, max), point| {
        (
            Point::new(min.x().min(point.x()), min.y().min(point.y())),
            Point::new(max.x().max(point.x()), max.y().max(point.y())),
        )
    })
}

fn checked_coordinate(value: f64) -> Result<Coord, ClipperError> {
    const MIN_COORDINATE: f64 = i64::MIN as f64;
    const MAX_COORDINATE_EXCLUSIVE: f64 = -MIN_COORDINATE;
    if value.is_finite() && (MIN_COORDINATE..MAX_COORDINATE_EXCLUSIVE).contains(&value) {
        Ok(value.trunc() as Coord)
    } else {
        Err(ClipperError::CoordinateOutOfRange)
    }
}
