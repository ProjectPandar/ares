//! Borrowed `Layer`/`LayerRegion` inputs used by support contact geometry.

use super::remove_bridges::RegionBridges;
use crate::geometry::{ExPolygon, Polygon};

pub(super) struct RegionInput<'a> {
    pub(super) slices: &'a [ExPolygon],
    pub(super) raw_slices: &'a [ExPolygon],
    pub(super) external_width_scaled: f32,
    pub(super) bridges: RegionBridges<'a>,
}

pub(super) struct LayerInput<'a> {
    pub(super) slices: &'a [ExPolygon],
    pub(super) regions: &'a [RegionInput<'a>],
    pub(super) height: f64,
}

pub(super) struct LayerAnnotations<'a> {
    // Some(empty) still means buildplate-only; None means that mode is disabled.
    pub(super) buildplate_covered: Option<&'a [Polygon]>,
    pub(super) enforcers: &'a [Polygon],
    pub(super) blockers: &'a [Polygon],
}

pub(super) fn to_polygons(expolygons: &[ExPolygon]) -> Vec<Polygon> {
    expolygons
        .iter()
        .flat_map(|expolygon| std::iter::once(expolygon.contour()).chain(expolygon.holes()))
        .cloned()
        .collect()
}
