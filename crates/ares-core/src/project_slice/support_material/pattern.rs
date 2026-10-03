//! `SupportMaterial.cpp::SupportGridPattern` Grid/Snug dispatch.

use crate::geometry::{
    ClipperError, Coord, CoordinateScale, JoinType, Polygon,
    mutable_polygon::smooth_outward,
    offset_paths,
    support_grid::{SupportGridParams, SupportGridPattern},
};

pub(super) enum SupportPattern {
    Grid(SupportGridPattern),
    Snug(Vec<Polygon>),
}

impl SupportPattern {
    pub(super) fn new(
        support: &[Polygon],
        trimming: &[Polygon],
        params: &SupportGridParams,
        scale: CoordinateScale,
    ) -> Result<Self, ClipperError> {
        if params.snug {
            let closing = (2.0 * params.units_per_mm) as f32;
            let expanded = offset_paths(support, closing, JoinType::Square, 0.0)?;
            let closed = offset_paths(&expanded, -closing, JoinType::Square, 0.0)?;
            let smoothing = (params.extrusion_width * params.units_per_mm) as f32;
            Ok(Self::Snug(smooth_outward(
                closed,
                smoothing as Coord,
                scale,
            )))
        } else {
            SupportGridPattern::new(support, trimming, params).map(Self::Grid)
        }
    }

    pub(super) fn extract_support(
        &self,
        support: &[Polygon],
        trimming: &[Polygon],
        offset: Coord,
        fill_holes: bool,
    ) -> Vec<Polygon> {
        match self {
            Self::Grid(grid) => grid.extract_support(support, trimming, offset, fill_holes),
            Self::Snug(polygons) => polygons.clone(),
        }
    }
}
