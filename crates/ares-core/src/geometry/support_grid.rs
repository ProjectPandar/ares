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
//! Rotation follows `:659-666,841-842`; the active rasterizer is AGG.
//! Deferred: non-grid styles.

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
    angle: f64,
    rotated: Option<(Vec<Polygon>, Vec<Polygon>)>,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct SupportGridParams {
    pub(crate) snug: bool,
    pub(crate) angle: f64,
    /// `support_base_pattern_spacing + flow.spacing()`
    /// (`SupportMaterial.cpp:620`) — `m_support_spacing`, kept as a
    /// double mm like upstream (`:636`, `grid_resolution: double`);
    /// `scale_` truncates to `coord_t` at the point of use (`:670`,
    /// `:680`, `:958`).
    pub(crate) grid_resolution: f64,
    /// `support_material_flow.spacing()` — `m_extrusion_width`
    /// (`:623`, `extrusion_width: double`), scaled via
    /// `scale_(extrusion_width)` inside `create`.
    pub(crate) extrusion_width: f64,
    /// `flow.scaled_spacing() / 2 + 5` (`:625`): `coord_t` of the
    /// scaled spacing, integer-halved, plus 5.
    pub(crate) expansion_to_slice: Coord,
    /// Lattice units per millimetre (`1 / CoordinateScale::factor()`),
    /// applying `scale_` to the mm-valued fields.
    pub(crate) units_per_mm: f64,
}

impl SupportGridParams {
    /// `grid_resolution` and `flow_spacing` are millimetres; lattice
    /// scaling happens at use, matching upstream's double params.
    pub(crate) fn new(grid_resolution: f64, flow_spacing: f64, units_per_mm: f64) -> Self {
        let flow_spacing_scaled = (flow_spacing * units_per_mm) as Coord;
        Self {
            snug: false,
            angle: 0.0,
            grid_resolution,
            extrusion_width: flow_spacing,
            expansion_to_slice: flow_spacing_scaled / 2 + 5,
            units_per_mm,
        }
    }
}

impl SupportGridPattern {
    pub(crate) fn new(
        support_polygons: &[Polygon],
        trimming_polygons: &[Polygon],
        params: &SupportGridParams,
    ) -> Result<Self, crate::geometry::ClipperError> {
        let rotated = (params.angle != 0.0).then(|| {
            let mut support = support_polygons.to_vec();
            let mut trimming = trimming_polygons.to_vec();
            rotate(&mut support, -params.angle);
            rotate(&mut trimming, -params.angle);
            (support, trimming)
        });
        let (support_polygons, trimming_polygons) = rotated.as_ref().map_or(
            (support_polygons, trimming_polygons),
            |(support, trimming)| (support.as_slice(), trimming.as_slice()),
        );
        // `coord_t grid_resolution = coord_t(scale_(m_support_spacing))`
        // (`:670`): trunc to the integer lattice.
        let resolution = (params.grid_resolution * params.units_per_mm) as Coord;
        let resolution = resolution.max(1);
        // Native align_to_grid changes only the minimum of a defined box.
        let (mut bx0, mut by0, mut bx1, mut by1) = match bounds_of(support_polygons) {
            Some((min_x, min_y, max_x, max_y)) => (
                align_down(min_x - 20, resolution),
                align_down(min_y - 20, resolution),
                max_x + 20,
                max_y + 20,
            ),
            None => (-20, -20, 20, 20),
        };

        // AGG path (`:676-696`): oversample the grid.
        // `scale_(params.extrusion_width)` trunc (`:679`).
        let extrusion_width_scaled = (params.extrusion_width * params.units_per_mm) as Coord;
        let extrusion_width_scaled = extrusion_width_scaled.max(1);
        // `int(scale_(m_support_spacing) / (extrusion_width_scaled + 100))`
        // (`:680`): int() truncates the DOUBLE quotient of the raw
        // scaled spacing, not the truncated grid_resolution.
        let spacing_scaled = params.grid_resolution * params.units_per_mm;
        let oversampling =
            ((spacing_scaled / (extrusion_width_scaled + 100) as f64) as i64).clamp(1, 8) as usize;
        // `scale_(m_support_spacing / oversampling)` (`:681`): the mm
        // spacing is divided first, then scaled — keep that order.
        let pixel_size = ((extrusion_width_scaled + 21) as f64)
            .max((params.grid_resolution / oversampling as f64) * params.units_per_mm);
        // `m_bbox.offset(m_pixel_size)` (`:684`): `BoundingBox::offset`
        // constructs `Point(delta, delta)`, whose double constructor
        // rounds (`BoundingBox.cpp:202-208`, `Point.hpp:196`) — the
        // bbox expansion rounds even though `m_pixel_size` stays a
        // double for the grid-size division below.
        let pixel_rounded = pixel_size.round() as Coord;
        bx0 -= pixel_rounded;
        by0 -= pixel_rounded;
        bx1 += pixel_rounded;
        by1 += pixel_rounded;
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
            (grid_blocks_x, grid_blocks_y),
            oversampling,
        );
        Ok(Self {
            angle: params.angle,
            rotated,
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
        let (support_polygons, trimming_polygons) = self.rotated.as_ref().map_or(
            (support_polygons, trimming_polygons),
            |(support, trimming)| (support.as_slice(), trimming.as_slice()),
        );
        let simplified = agg_raster::contours_simplified(
            agg_raster::RasterGeometry {
                width: self.grid_w,
                height: self.grid_h,
                pixel_size: self.pixel_size,
                left_bottom: self.left_bottom,
            },
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
        let mut output = islands
            .into_iter()
            .filter(|island| {
                samples
                    .iter()
                    .any(|sample| island_contains(island, *sample))
            })
            .flat_map(|island| {
                let (contour, holes) = island.into_parts();
                std::iter::once(contour).chain(holes)
            })
            .collect::<Vec<_>>();
        if self.angle != 0.0 {
            rotate(&mut output, self.angle);
        }
        output
    }
}

fn rotate(polygons: &mut [Polygon], angle: f64) {
    let cos_angle = angle.cos();
    let sin_angle = angle.sin();
    for polygon in polygons {
        polygon.rotate(cos_angle, sin_angle);
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

fn bounds_of(polygons: &[Polygon]) -> Option<(Coord, Coord, Coord, Coord)> {
    let mut points = polygons.iter().flat_map(|polygon| polygon.points());
    let first = points.next()?;
    let (mut min_x, mut min_y) = (first.x(), first.y());
    let (mut max_x, mut max_y) = (min_x, min_y);
    for point in points {
        min_x = min_x.min(point.x());
        min_y = min_y.min(point.y());
        max_x = max_x.max(point.x());
        max_y = max_y.max(point.y());
    }
    Some((min_x, min_y, max_x, max_y))
}

fn align_down(value: Coord, resolution: Coord) -> Coord {
    value.div_euclid(resolution) * resolution
}
