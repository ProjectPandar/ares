mod detect_bridge_directions;
mod expand_expolygons;
mod expand_merge;
mod group_bridges;
#[cfg(test)]
mod tests;
mod types;

use crate::{
    geometry::{ClipperError, CoordinateScale, ExPolygon},
    project_slice::region_slices::{RegionSurface, RegionSurfaceKind},
};

pub(in crate::project_slice) use detect_bridge_directions::detect_bridge_directions;
pub(in crate::project_slice) use expand_expolygons::expand_expolygons;
pub(in crate::project_slice) use expand_merge::expand_merge_surfaces;
pub(in crate::project_slice) use group_bridges::{get_grouped_bridges, group_id};
pub(in crate::project_slice) use types::{Bridge, ExpansionResult, ExpansionZone};

type BridgePartsFn = fn(Bridge) -> (ExPolygon, u32, usize, Option<f64>);
type DetectBridgeDirectionsFn = fn(
    &[crate::geometry::WaveSeed],
    &mut [Bridge],
    &[ExpansionZone],
    CoordinateScale,
) -> Result<(), ClipperError>;
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
type GetGroupedBridgesFn =
    fn(Vec<ExPolygon>, &[crate::geometry::RegionExpansionEx]) -> Result<Vec<Bridge>, ClipperError>;
type GroupIdFn = fn(&mut [Bridge], u32) -> u32;

const _: DetectBridgeDirectionsFn = detect_bridge_directions;
const _: BridgePartsFn = |bridge| {
    (
        bridge.expolygon,
        bridge.group_id,
        bridge.bridge_expansion_begin,
        bridge.angle,
    )
};
const _: ExpandExPolygonsFn = expand_expolygons;
const _: ExpansionResultPartsFn = |result| (result.anchors, result.expansions);
const _: ExpandMergeSurfacesFn = expand_merge_surfaces;
const _: GetGroupedBridgesFn = get_grouped_bridges;
const _: GroupIdFn = group_id;
const _: fn(
    Vec<crate::geometry::ExPolygon>,
    crate::geometry::RegionExpansionParameters,
) -> ExpansionZone = ExpansionZone::new;
