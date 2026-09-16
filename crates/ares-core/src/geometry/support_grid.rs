//! Support island grid stretching (`SupportGridPattern`, smsGrid
//! path), ported from `OrcaSlicer/src/libslic3r/Support/SupportMaterial.cpp:637-836`.
//!
//! Construction (`:662-738`): a tight, grid-aligned bounding box
//! (`bbox.offset(20)` + `align_to_grid`) feeds the EdgeGrid; the
//! signed distance field comes from `EdgeGrid::calculate_sdf`.
//! `extract_support` (`:739-836`): `contours_simplified` snaps the
//! islands onto the grid, `diff_ex` trims them, and only islands
//! containing a sampled input point survive (`island_samples`,
//! `:1070-1095`: up to 4 samples from the first shrunk contour).
//!
//! Deferred: `support_angle != 0` rotation (KSR default 0°),
//! non-grid styles, and the AGG rasterizer variant.

use crate::geometry::clipper::{
    FillRule, JoinType, difference_ex_polygons, intersection_ex, offset_expolygons, union_ex,
};
use crate::geometry::edge_grid::{EdgeGrid, SignedDistanceField};
use crate::geometry::{Coord, ExPolygon, Point, Polygon};

pub(crate) struct SupportGridPattern {
    grid: EdgeGrid,
    field: SignedDistanceField,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct SupportGridParams {
    /// `support_base_pattern_spacing + flow.spacing()`
    /// (`SupportMaterial.cpp:620`).
    pub(crate) grid_resolution: Coord,
    /// `flow.scaled_spacing() / 2 + 5` (`:625`).
    pub(crate) expansion_to_slice: Coord,
    /// `-3` (`:626`).
    pub(crate) expansion_to_propagate: Coord,
}

impl SupportGridParams {
    pub(crate) fn new(grid_resolution: Coord, flow_spacing: Coord) -> Self {
        Self {
            grid_resolution,
            expansion_to_slice: flow_spacing / 2 + 5,
            expansion_to_propagate: -3,
        }
    }
}

impl SupportGridPattern {
    pub(crate) fn new(
        support_polygons: &[Polygon],
        trimming_polygons: &[Polygon],
        params: &SupportGridParams,
    ) -> Result<Self, crate::geometry::ClipperError> {
        let (min_x, min_y, max_x, max_y) = bounds_of(support_polygons);
        // `bbox.offset(20); bbox.align_to_grid(grid_resolution);`
        let (min_x, min_y) = (min_x - 20, min_y - 20);
        let (max_x, max_y) = (max_x + 20, max_y + 20);
        let resolution = params.grid_resolution.max(1);
        let min_x = align_down(min_x, resolution);
        let min_y = align_down(min_y, resolution);
        let max_x = min_x + div_ceil(max_x - min_x, resolution) * resolution;
        let max_y = min_y + div_ceil(max_y - min_y, resolution) * resolution;

        let contours = support_polygons
            .iter()
            .chain(trimming_polygons)
            .map(|polygon| polygon.points().to_vec())
            .collect::<Vec<_>>();
        let grid = EdgeGrid::new_from_contours(
            contours.iter().map(Vec::as_slice),
            Point::new(min_x, min_y),
            Point::new(max_x, max_y),
            resolution,
        )?;
        let field = grid.calculate_sdf();
        Ok(Self { grid, field })
    }

    /// `extract_support` (`SupportMaterial.cpp:739`): grid-snapped,
    /// trimmed islands that contain at least one sampled input point.
    pub(crate) fn extract_support(
        &self,
        support_polygons: &[Polygon],
        trimming_polygons: &[Polygon],
        offset_in_grid: Coord,
        fill_holes: bool,
    ) -> Vec<Polygon> {
        let simplified = self
            .grid
            .contours_simplified(&self.field, offset_in_grid, fill_holes);
        if simplified.is_empty() {
            return simplified;
        }
        let simplified_ex = union_ex(&simplified, FillRule::NonZero).unwrap_or_default();
        let islands = match difference_ex_polygons(&simplified_ex, trimming_polygons) {
            Ok(islands) => islands,
            Err(_) => return simplified,
        };
        if islands.is_empty() {
            return Vec::new();
        }

        // `island_samples` (`:1070`): up to 4 samples of the first
        // shrunk contour per input expolygon.
        let sampled = if offset_in_grid > 0 {
            union_ex(support_polygons, FillRule::NonZero).unwrap_or_default()
        } else {
            let support_ex = union_ex(support_polygons, FillRule::NonZero).unwrap_or_default();
            intersection_ex(&support_ex, &islands).unwrap_or_default()
        };
        let samples = island_samples(&sampled);
        if samples.is_empty() {
            return Vec::new();
        }

        // Keep islands containing a sample (`:776-820`).
        islands
            .into_iter()
            .filter(|island| {
                samples
                    .iter()
                    .any(|sample| island_contains(island, *sample))
            })
            .map(|island| island.into_parts().0)
            .collect()
    }
}

/// One sample point set per expolygon: the first `-20` shrunk contour
/// sampled at `min(len, 4)` evenly strided points
/// (`SupportMaterial.cpp:1070-1095`).
pub(crate) fn island_samples(expolygons: &[ExPolygon]) -> Vec<Point> {
    let mut points = Vec::new();
    for expolygon in expolygons {
        let shrunk =
            match offset_expolygons(std::slice::from_ref(expolygon), -20.0, JoinType::Miter, 3.0) {
                Ok(shrunk) => shrunk,
                Err(_) => continue,
            };
        let Some(polygon) = shrunk
            .iter()
            .map(ExPolygon::contour)
            .find(|contour| !contour.points().is_empty())
        else {
            continue;
        };
        let count = polygon.points().len();
        let stride = count / count.min(4);
        points.extend(
            (0..count)
                .step_by(stride)
                .map(|index| polygon.points()[index]),
        );
    }
    points
}

fn island_contains(island: &ExPolygon, point: Point) -> bool {
    island.contour().contains(&point) && !island.holes().iter().any(|hole| hole.contains(&point))
}

fn bounds_of(polygons: &[Polygon]) -> (Coord, Coord, Coord, Coord) {
    let (mut min_x, mut min_y) = (Coord::MAX, Coord::MAX);
    let (mut max_x, mut max_y) = (Coord::MIN, Coord::MIN);
    for point in polygons.iter().flat_map(|polygon| polygon.points()) {
        min_x = min_x.min(point.x());
        min_y = min_y.min(point.y());
        max_x = max_x.max(point.x());
        max_y = max_y.max(point.y());
    }
    (min_x, min_y, max_x, max_y)
}

fn align_down(value: Coord, resolution: Coord) -> Coord {
    value.div_euclid(resolution) * resolution
}

fn div_ceil(value: Coord, divisor: Coord) -> Coord {
    (value + divisor - 1).div_euclid(divisor)
}

#[cfg(test)]
mod tests;
