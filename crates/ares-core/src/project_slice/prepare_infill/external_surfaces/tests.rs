mod errors;
mod helpers;
mod ownership;
mod pipeline;

use super::{ExpansionZone, expand_merge_surfaces};
use crate::{
    geometry::{ClipperError, CoordinateScale},
    project_slice::region_slices::{RegionSurface, RegionSurfaceKind},
};

type ExpandMergeSurfacesFn = fn(
    &mut [RegionSurface],
    RegionSurfaceKind,
    &mut [ExpansionZone],
    f32,
    f64,
    CoordinateScale,
) -> Result<Vec<RegionSurface>, ClipperError>;

pub(super) const EXPAND_MERGE_SURFACES: ExpandMergeSurfacesFn = expand_merge_surfaces;
