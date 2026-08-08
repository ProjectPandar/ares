mod expand_expolygons;
mod expand_merge;
#[cfg(test)]
mod tests;
mod types;

use crate::{
    geometry::{ClipperError, CoordinateScale, ExPolygon},
    project_slice::region_slices::{RegionSurface, RegionSurfaceKind},
};

pub(in crate::project_slice) use expand_expolygons::expand_expolygons;
pub(in crate::project_slice) use expand_merge::expand_merge_surfaces;
pub(in crate::project_slice) use types::{ExpansionResult, ExpansionZone};

type ExpandExPolygonsFn = fn(
    &[ExPolygon],
    &mut [ExpansionZone],
    CoordinateScale,
) -> Result<ExpansionResult, ClipperError>;
type ExpansionResultPartsFn = fn(
    ExpansionResult,
) -> (
    Vec<crate::geometry::WaveSeed>,
    Vec<crate::geometry::RegionExpansionEx>,
);
type ExpandMergeSurfacesFn = fn(
    &mut [RegionSurface],
    RegionSurfaceKind,
    &mut [ExpansionZone],
    f32,
    f64,
    CoordinateScale,
) -> Result<Vec<RegionSurface>, ClipperError>;

const _: ExpandExPolygonsFn = expand_expolygons;
const _: ExpansionResultPartsFn = |result| (result.anchors, result.expansions);
const _: ExpandMergeSurfacesFn = expand_merge_surfaces;
const _: fn(
    Vec<crate::geometry::ExPolygon>,
    crate::geometry::RegionExpansionParameters,
) -> ExpansionZone = ExpansionZone::new;
