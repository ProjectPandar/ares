mod expand_merge;
#[cfg(test)]
mod tests;
mod types;

use crate::{
    geometry::{ClipperError, CoordinateScale},
    project_slice::region_slices::{RegionSurface, RegionSurfaceKind},
};

pub(in crate::project_slice) use expand_merge::expand_merge_surfaces;
pub(in crate::project_slice) use types::ExpansionZone;

type ExpandMergeSurfacesFn = fn(
    &mut [RegionSurface],
    RegionSurfaceKind,
    &mut [ExpansionZone],
    f32,
    f64,
    CoordinateScale,
) -> Result<Vec<RegionSurface>, ClipperError>;

const _: ExpandMergeSurfacesFn = expand_merge_surfaces;
const _: fn(
    Vec<crate::geometry::ExPolygon>,
    crate::geometry::RegionExpansionParameters,
) -> ExpansionZone = ExpansionZone::new;
