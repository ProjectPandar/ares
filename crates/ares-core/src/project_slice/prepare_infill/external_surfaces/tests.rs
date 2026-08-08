mod errors;
mod expand_expolygons;
mod helpers;
mod ownership;
mod pipeline;

use super::{
    ExpansionResult, ExpansionZone, expand_expolygons as expand_expolygons_fn,
    expand_merge_surfaces,
};
use crate::{
    geometry::{ClipperError, CoordinateScale},
    project_slice::region_slices::{RegionSurface, RegionSurfaceKind},
};

type ExpandExPolygonsFn = fn(
    &[crate::geometry::ExPolygon],
    &mut [ExpansionZone],
    CoordinateScale,
) -> Result<ExpansionResult, ClipperError>;
type ExpandMergeSurfacesFn = fn(
    &mut [RegionSurface],
    RegionSurfaceKind,
    &mut [ExpansionZone],
    f32,
    f64,
    CoordinateScale,
) -> Result<Vec<RegionSurface>, ClipperError>;

pub(super) const EXPAND_EXPOLYGONS: ExpandExPolygonsFn = expand_expolygons_fn;
pub(super) const EXPAND_MERGE_SURFACES: ExpandMergeSurfacesFn = expand_merge_surfaces;
