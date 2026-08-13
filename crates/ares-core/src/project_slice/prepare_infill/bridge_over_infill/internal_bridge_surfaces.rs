#![cfg_attr(
    not(test),
    expect(dead_code, reason = "unwired bridge transaction dependency")
)]

#[cfg(test)]
mod tests;

use super::types::CandidateSurface;
use crate::{
    geometry::{ClipperError, ExPolygon, FillRule, union_ex},
    project_slice::region_slices::{RegionSurface, RegionSurfaceKind},
};

pub(in crate::project_slice) fn build_internal_bridge_surfaces(
    region_index: usize,
    fill_surfaces: &[RegionSurface],
    candidates: &[CandidateSurface],
) -> Result<Vec<RegionSurface>, ClipperError> {
    build_internal_bridge_surfaces_using(region_index, fill_surfaces, candidates, |polygons| {
        union_ex(polygons, FillRule::NonZero)
    })
}

fn build_internal_bridge_surfaces_using<Union>(
    region_index: usize,
    fill_surfaces: &[RegionSurface],
    candidates: &[CandidateSurface],
    mut union: Union,
) -> Result<Vec<RegionSurface>, ClipperError>
where
    Union: FnMut(&[crate::geometry::Polygon]) -> Result<Vec<ExPolygon>, ClipperError>,
{
    let mut output = Vec::new();
    for candidate in candidates {
        if candidate.source.region_index != region_index {
            continue;
        }
        let Some(source) = fill_surfaces.get(candidate.source.surface_index) else {
            continue;
        };
        if source.as_parts().0 != RegionSurfaceKind::InternalSolid {
            continue;
        }
        for expolygon in union(&candidate.new_polygons)? {
            let mut bridge = source.clone_with_expolygon(expolygon);
            bridge.retag(RegionSurfaceKind::InternalBridge);
            bridge.set_bridge_angle(candidate.bridge_angle);
            output.push(bridge);
        }
    }
    Ok(output)
}
