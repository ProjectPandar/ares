//! `fill_expolygons_with_sheath_generate_paths` (`SupportCommon.cpp:705-757`)
//! and `draw_perimeters` (`:459-475`).

use super::geom::clip_polyline_end;
use super::{RaftEntity, RaftFillSpec, closing_polygons, fill_expolygons};
use crate::geometry::{
    ClipperError, CoordinateScale, ExPolygon, JoinType, Point, Polygon, Polyline,
};

/// The integer `Flow` metrics the sheath paths use.
pub(super) struct SheathFlow {
    /// `flow.scaled_width()` (`Flow.hpp:62`): `Flow` stores the width as
    /// `float` (`Flow.cpp:129-143`), truncated to `coord_t`.
    pub(super) scaled_width: f64,
    /// `flow.scaled_spacing()` (`:733`), likewise truncated.
    pub(super) spacing: f64,
}

impl SheathFlow {
    pub(super) fn new(spec: &RaftFillSpec, scale: CoordinateScale) -> Self {
        let scaled = |mm: f64| (f64::from(mm as f32) / scale.factor()) as i64 as f64;
        Self {
            scaled_width: scaled(spec.width_mm),
            spacing: scaled(spec.sheath_spacing),
        }
    }

    /// `closing_ex(polygons, float(SCALED_EPSILON),
    /// float(SCALED_EPSILON + 0.5·scaled_width))` (`:738`, `:589`): a net
    /// shrink of half a line width, so the loops sit inside the boundary.
    /// `SCALED_EPSILON = scale_(1e-4)` (`libslic3r.h:52,96`).
    pub(super) fn regions(
        &self,
        polygons: &[Polygon],
        scale: CoordinateScale,
    ) -> Result<Vec<ExPolygon>, ClipperError> {
        let epsilon = scale.scaled_delta(1e-4);
        closing_polygons(
            polygons,
            epsilon as f32,
            (epsilon + 0.5 * self.scaled_width) as f32,
        )
    }
}

/// `draw_perimeters(expoly, clip_length)`: the contour, then each hole
/// reversed, each closed and clipped at the end.
pub(super) fn draw_perimeters(expolygon: &ExPolygon, clip_length: f64) -> Vec<Polyline> {
    let loops = std::iter::once((expolygon.contour(), false))
        .chain(expolygon.holes().iter().map(|hole| (hole, true)));
    let mut out = Vec::with_capacity(expolygon.holes().len() + 1);
    for (polygon, hole) in loops {
        let mut points = polygon.points().to_vec();
        points.push(points[0]);
        if hole {
            points.reverse();
        }
        let mut polyline = Polyline::new(points);
        clip_polyline_end(&mut polyline, clip_length);
        if polyline.is_valid() {
            out.push(polyline);
        }
    }
    out
}

/// The `density > 0` sheath branch (`:727-754`): per region the clipped
/// loops, then the fill on `offset_ex(expoly, -0.4·spacing)`.
pub(super) fn fill_with_sheath(
    polygons: &[Polygon],
    spec: RaftFillSpec,
    scale: CoordinateScale,
) -> Result<Vec<RaftEntity>, ClipperError> {
    let flow = SheathFlow::new(&spec, scale);
    let mut out = Vec::new();
    for expolygon in flow.regions(polygons, scale)? {
        let mut region = draw_perimeters(&expolygon, 0.15 * flow.spacing);
        let fill_region = crate::geometry::offset_expolygon(
            &expolygon,
            (-0.4 * flow.spacing) as f32,
            JoinType::Miter,
            3.0,
        )?;
        region.extend(fill_expolygons(fill_region, spec, Point::new(0, 0), scale)?);
        if !spec.no_sort {
            out.extend(region.into_iter().map(RaftEntity::Path));
        } else if !region.is_empty() {
            out.push(RaftEntity::NoSort(region));
        }
    }
    Ok(out)
}
