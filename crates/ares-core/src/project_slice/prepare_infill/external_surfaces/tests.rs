mod detect_bridge_directions;
mod errors;
mod expand_expolygons;
mod group_bridges;
mod helpers;
mod ownership;
mod pipeline;

use super::{
    Bridge, ExpansionResult, ExpansionZone, expand_expolygons as expand_expolygons_fn,
    expand_merge_surfaces, get_grouped_bridges as get_grouped_bridges_fn, group_id as group_id_fn,
};
use crate::{
    geometry::{ClipperError, CoordinateScale},
    project_slice::region_slices::{RegionSurface, RegionSurfaceKind},
};

type DetectBridgeDirectionsFn = fn(
    &[crate::geometry::WaveSeed],
    &mut [Bridge],
    &[ExpansionZone],
    CoordinateScale,
) -> Result<(), ClipperError>;
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
type GetGroupedBridgesFn = fn(
    Vec<crate::geometry::ExPolygon>,
    &[crate::geometry::RegionExpansionEx],
) -> Result<Vec<Bridge>, ClipperError>;
type GroupIdFn = fn(&mut [Bridge], u32) -> u32;

pub(super) const DETECT_BRIDGE_DIRECTIONS: DetectBridgeDirectionsFn =
    super::detect_bridge_directions;
pub(super) const EXPAND_EXPOLYGONS: ExpandExPolygonsFn = expand_expolygons_fn;
pub(super) const EXPAND_MERGE_SURFACES: ExpandMergeSurfacesFn = expand_merge_surfaces;
pub(super) const GET_GROUPED_BRIDGES: GetGroupedBridgesFn = get_grouped_bridges_fn;
pub(super) const GROUP_ID: GroupIdFn = group_id_fn;
