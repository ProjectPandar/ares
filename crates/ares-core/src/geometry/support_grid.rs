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

use crate::geometry::agg_raster;
use crate::geometry::clipper::{
    FillRule, JoinType, difference_ex_polygons, intersection_ex, offset_expolygons, union_ex,
};
use crate::geometry::{Coord, ExPolygon, Point, Polygon};

/// The AGG-rasterizer `SupportGridPattern` state (`SupportMaterial.cpp` with
/// `SUPPORT_USE_AGG_RASTERIZER` defined at `:21`): a byte grid `grid2`
/// rasterized at `pixel_size` over `bbox`, seed-filled to the dilated
/// trimming mask. `extract_support` walks it with the static
/// `contours_simplified`.
pub(crate) struct SupportGridPattern {
    grid2: Vec<u8>,
    grid_w: usize,
    grid_h: usize,
    pixel_size: f64,
    left_bottom: Point,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct SupportGridParams {
    /// `support_base_pattern_spacing + flow.spacing()`
    /// (`SupportMaterial.cpp:620`) — `m_support_spacing`.
    pub(crate) grid_resolution: Coord,
    /// `flow.scaled_spacing()` — `m_extrusion_width` (`:622`).
    pub(crate) extrusion_width: Coord,
    /// `flow.scaled_spacing() / 2 + 5` (`:625`).
    pub(crate) expansion_to_slice: Coord,
    /// `-3` (`:626`).
    pub(crate) expansion_to_propagate: Coord,
}

impl SupportGridParams {
    pub(crate) fn new(grid_resolution: Coord, flow_spacing: Coord) -> Self {
        Self {
            grid_resolution,
            extrusion_width: flow_spacing,
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
        let mut bx0 = align_down(min_x, resolution);
        let mut by0 = align_down(min_y, resolution);
        let mut bx1 = bx0 + div_ceil(max_x - bx0, resolution) * resolution;
        let mut by1 = by0 + div_ceil(max_y - by0, resolution) * resolution;

        // AGG path (`:676-696`): oversample the grid.
        let extrusion_width_scaled = params.extrusion_width.max(1);
        let oversampling =
            ((params.grid_resolution as i64) / (extrusion_width_scaled + 100)).clamp(1, 8) as usize;
        let pixel_size = ((extrusion_width_scaled + 21) as f64)
            .max(params.grid_resolution as f64 / oversampling as f64);
        // `m_bbox.offset(m_pixel_size)`.
        bx0 -= pixel_size as Coord;
        by0 -= pixel_size as Coord;
        bx1 += pixel_size as Coord;
        by1 += pixel_size as Coord;
        let grid_w_raw = (((bx1 - bx0) as f64) / pixel_size).ceil() as usize;
        let grid_h_raw = (((by1 - by0) as f64) / pixel_size).ceil() as usize;
        let grid_blocks_x = (grid_w_raw + oversampling - 1 - 2) / oversampling;
        let grid_blocks_y = (grid_h_raw + oversampling - 1 - 2) / oversampling;
        let grid_w = grid_blocks_x * oversampling + 2;
        let grid_h = grid_blocks_y * oversampling + 2;
        let left_bottom = Point::new(bx0, by0);

        let mut grid2 = agg_raster::rasterize_polygons(
            grid_w,
            grid_h,
            pixel_size,
            left_bottom,
            support_polygons,
        );
        let trimming_grid = agg_raster::rasterize_polygons(
            grid_w,
            grid_h,
            pixel_size,
            left_bottom,
            trimming_polygons,
        );
        let dilated = agg_raster::dilate_trimming_region(&trimming_grid, grid_w, grid_h);
        agg_raster::seed_fill_block(
            &mut grid2,
            grid_w,
            &dilated,
            grid_blocks_x,
            grid_blocks_y,
            oversampling,
        );
        Ok(Self {
            grid2,
            grid_w,
            grid_h,
            pixel_size,
            left_bottom,
        })
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
        let simplified = agg_raster::contours_simplified(
            self.grid_w,
            self.grid_h,
            self.pixel_size,
            self.left_bottom,
            &self.grid2,
            offset_in_grid,
            fill_holes,
        );
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
