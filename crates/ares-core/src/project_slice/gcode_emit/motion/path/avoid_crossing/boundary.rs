//! Avoid-crossing boundary construction — a source-cited port of
//! `GCode/AvoidCrossingPerimeters.cpp` (`inner_offset` :1014-1098,
//! `resample_polygon` :825-840, `init_boundary` :1197-1229,
//! `get_boundary` :1099-1134).

mod resample;

use crate::geometry::{
    ClipperError, Coord, CoordinateScale, EdgeGrid, ExPolygon, JoinType, Point, Polygon,
    difference_ex, offset_expolygons, offset_paths, union_expolygons,
};
use crate::project_slice::elephant_foot::distance::{
    DistanceThresholds, ResampledPoint, filtered_contour_distances,
};
use crate::project_slice::gcode_emit::motion::state::AvoidCrossingGeometry;
use resample::{inner_offset, resample_expolygon, resample_polygon};

pub(super) const SCALED_EPSILON: f64 = 1.0e-4;
pub(super) const MITER_LIMIT: f64 = 2.0;

/// The routing boundary: the inner-offset slice union as contours, an edge
/// grid over them, and per-contour cumulative distances.
#[derive(Clone)]
pub(super) struct Boundary {
    pub(super) scaled_spacing: f32,
    pub(super) contours: Vec<Vec<Point>>,
    pub(super) grid: EdgeGrid,
    pub(super) contour_lengths: Vec<Vec<f64>>,
    bounds: (Point, Point),
}

#[expect(clippy::large_enum_variant, reason = "avoid a transient allocation")]
pub(super) enum BuildResult {
    Unavailable,
    Empty,
    Ready(Boundary),
}

impl Boundary {
    /// `get_boundary_external` + the first `init_boundary` overload
    /// (`AvoidCrossingPerimeters.cpp:1137-1189, 1187-1203`): every hole of
    /// every print object at this print z, made CCW and expanded by half
    /// the cross-object average perimeter spacing (miter join), then
    /// reversed so normals point outward; the bbox is padded only by
    /// `SCALED_EPSILON`, so travels clear of every hole stay outside and
    /// route straight.
    pub(in crate::project_slice::gcode_emit) fn build_external(
        chunk_slices: &[ExPolygon],
        perimeter_spacing_mm: f64,
        scale: CoordinateScale,
        endpoints: [Point; 2],
    ) -> Result<BuildResult, ClipperError> {
        let unit = |millimetres: f64| scale.checked_scale(millimetres);
        let Some(scaled_spacing) = unit(perimeter_spacing_mm) else {
            return Ok(BuildResult::Unavailable);
        };
        let scaled_spacing = scaled_spacing as f32;
        // CW holes reversed to CCW before the positive offset
        // (`polygons_reverse(holes_per_obj)`).
        let holes = chunk_slices
            .iter()
            .flat_map(|expolygon| expolygon.holes())
            .map(|hole| {
                let mut points = hole.points().to_vec();
                points.reverse();
                Polygon::new(points)
            })
            .collect::<Vec<_>>();
        if holes.is_empty() {
            return Ok(BuildResult::Empty);
        }
        let expanded = offset_paths(&holes, 0.5 * scaled_spacing, JoinType::Miter, 3.0)?;
        if expanded.is_empty() {
            return Ok(BuildResult::Empty);
        }
        // Reverse every contour so the router's inward vertex offsets
        // steer travels around the hole instead of into it.
        let contours = expanded
            .iter()
            .map(|polygon| {
                let mut points = polygon.points().to_vec();
                points.reverse();
                points
            })
            .collect::<Vec<_>>();
        let (mut min, mut max) = contours_bounds(&contours);
        for point in endpoints {
            min = Point::new(min.x().min(point.x()), min.y().min(point.y()));
            max = Point::new(max.x().max(point.x()), max.y().max(point.y()));
        }
        let epsilon = scale.checked_scale(SCALED_EPSILON).unwrap_or(100) as Coord;
        let padded_min = Point::new(
            min.x().saturating_sub(epsilon),
            min.y().saturating_sub(epsilon),
        );
        let padded_max = Point::new(
            max.x().saturating_add(epsilon),
            max.y().saturating_add(epsilon),
        );
        let grid_resolution = unit(1.0).unwrap_or(1_000_000);
        let grid = EdgeGrid::new_from_contours(
            contours.iter().map(|contour| contour.as_slice()),
            padded_min,
            padded_max,
            grid_resolution,
        )?;
        let contour_lengths = contours
            .iter()
            .map(|contour| cumulative_distances(contour))
            .collect();
        Ok(BuildResult::Ready(Boundary {
            scaled_spacing,
            contours,
            grid,
            contour_lengths,
            bounds: (padded_min, padded_max),
        }))
    }

    pub(super) fn contour(&self, index: usize) -> &[Point] {
        &self.contours[index]
    }

    pub(super) fn lengths(&self, index: usize) -> &[f64] {
        &self.contour_lengths[index]
    }

    pub(super) fn contains(&self, point: Point) -> bool {
        let (min, max) = self.bounds;
        point.x() >= min.x() && point.y() >= min.y() && point.x() <= max.x() && point.y() <= max.y()
    }

    /// `get_boundary` + `init_boundary`: `union_ex(inner_offset(lslices,
    /// 1.5 * perimeter_spacing))`, minus an inset of the top fill surfaces,
    /// gridded at 1 mm cells.
    pub(in crate::project_slice::gcode_emit) fn build(
        geometry: &AvoidCrossingGeometry<'_>,
        scale: CoordinateScale,
        endpoints: [Point; 2],
    ) -> Result<BuildResult, ClipperError> {
        if geometry.layer_slices.is_empty() || geometry.perimeter_spacing <= 0.0 {
            return Ok(BuildResult::Unavailable);
        }
        let unit = |millimetres: f64| scale.checked_scale(millimetres);
        // `Flow::scaled_spacing` truncates before `get_perimeter_spacing`
        // converts to float; retain scaled units for all boundary radii.
        let Some(scaled_spacing) = unit(f64::from(geometry.perimeter_spacing)) else {
            return Ok(BuildResult::Unavailable);
        };
        let scaled_spacing = scaled_spacing as f32;
        let offset_dis = 1.5 * f64::from(scaled_spacing);
        let mut boundary = inner_offset(geometry.layer_slices, offset_dis, scale)?;
        if !geometry.top_surfaces.is_empty() {
            // perimeter_offset = spacing / 2; the diff insets the top
            // surfaces by 1.2 * perimeter_offset.
            let inset_by = 0.6 * f64::from(scaled_spacing);
            let inset = offset_expolygons(
                &geometry
                    .top_surfaces
                    .iter()
                    .map(|expolygon| (*expolygon).clone())
                    .collect::<Vec<_>>(),
                -(inset_by as f32),
                JoinType::Round,
                MITER_LIMIT,
            )?;
            boundary = difference_ex(&boundary, &inset)?;
        }
        if boundary.is_empty() {
            return Ok(BuildResult::Empty);
        }
        let contours = boundary
            .iter()
            .flat_map(|expolygon| {
                std::iter::once(expolygon.contour())
                    .chain(expolygon.holes())
                    .map(|polygon| polygon.points().to_vec())
            })
            .collect::<Vec<_>>();
        let (mut min, mut max) = contours_bounds(&contours);
        for point in endpoints {
            min = Point::new(min.x().min(point.x()), min.y().min(point.y()));
            max = Point::new(max.x().max(point.x()), max.y().max(point.y()));
        }
        // `init_boundary(boundary, polygons, merge_points)` pads the bounds by
        // the bbox radius so travel endpoints outside the contours stay in
        // the grid (`AvoidCrossingPerimeters.cpp:1216-1229`).
        let radius =
            (((max.x() - min.x()) as f64).hypot((max.y() - min.y()) as f64) / 2.0) as Coord;
        let padded_min = Point::new(
            min.x().saturating_sub(radius),
            min.y().saturating_sub(radius),
        );
        let padded_max = Point::new(
            max.x().saturating_add(radius),
            max.y().saturating_add(radius),
        );
        let grid_resolution = unit(1.0).unwrap_or(1_000_000);
        let grid = EdgeGrid::new_from_contours(
            contours.iter().map(|contour| contour.as_slice()),
            padded_min,
            padded_max,
            grid_resolution,
        )?;
        let contour_lengths = contours
            .iter()
            .map(|contour| cumulative_distances(contour))
            .collect();
        Ok(BuildResult::Ready(Boundary {
            scaled_spacing,
            contours,
            grid,
            contour_lengths,
            bounds: (padded_min, padded_max),
        }))
    }
}

fn cumulative_distances(contour: &[Point]) -> Vec<f64> {
    let mut lengths = Vec::with_capacity(contour.len() + 1);
    lengths.push(0.0);
    let mut total = 0.0;
    for pair in contour.windows(2) {
        total += distance(pair[0], pair[1]);
        lengths.push(total);
    }
    if let Some(&last) = contour.last() {
        total += distance(last, contour[0]);
        lengths.push(total);
    }
    lengths
}

pub(super) fn distance(first: Point, second: Point) -> f64 {
    let dx = second.x() as f64 - first.x() as f64;
    let dy = second.y() as f64 - first.y() as f64;
    dx.hypot(dy)
}

pub(super) fn contours_bounds(contours: &[Vec<Point>]) -> (Point, Point) {
    let first = contours
        .iter()
        .flatten()
        .copied()
        .next()
        .expect("nonempty contours");
    let mut min = first;
    let mut max = first;
    for &point in contours.iter().flatten() {
        min = Point::new(min.x().min(point.x()), min.y().min(point.y()));
        max = Point::new(max.x().max(point.x()), max.y().max(point.y()));
    }
    (min, max)
}
