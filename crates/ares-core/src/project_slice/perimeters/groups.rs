use std::collections::BTreeMap;

use super::types::RegionLayerIndex;
use crate::project_slice::region_slices::{PostRegion, RegionSurface};
use crate::{
    SliceError,
    geometry::{JoinType, SAFETY_OFFSET, offset_expolygons},
};

mod compatibility;
pub(super) mod redistribute;

pub(super) struct PerimeterGroup {
    pub(super) owner: RegionLayerIndex,
    pub(super) members: Vec<RegionLayerIndex>,
    pub(super) merged_surfaces: Option<Vec<RegionSurface>>,
}

// Layer.cpp::make_perimeters: physical layer, first compatible region, surface bucket.
pub(super) fn prepare(
    regions: &[PostRegion],
    layer_count: usize,
    filament_map: &[crate::OrcaInt],
) -> Result<Vec<PerimeterGroup>, SliceError> {
    let mut groups = Vec::new();
    for layer_index in 0..layer_count {
        prepare_layer(regions, layer_index, &mut groups, filament_map)?;
    }
    Ok(groups)
}

fn prepare_layer(
    regions: &[PostRegion],
    layer_index: usize,
    groups: &mut Vec<PerimeterGroup>,
    filament_map: &[crate::OrcaInt],
) -> Result<(), SliceError> {
    let mut done = vec![false; regions.len()];
    for (region_index, region) in regions.iter().enumerate() {
        if done[region_index] || region.layers[layer_index].surfaces.is_empty() {
            continue;
        }
        done[region_index] = true;
        let first = RegionLayerIndex {
            region_index,
            layer_index,
        };
        let mut members = vec![first];
        let mut owner = first;
        for (other_index, other) in regions.iter().enumerate().skip(region_index + 1) {
            if other.layers[layer_index].surfaces.is_empty()
                || !compatibility::is_compatible(&region.options, &other.options, filament_map)
            {
                continue;
            }
            done[other_index] = true;
            let member = RegionLayerIndex {
                region_index: other_index,
                layer_index,
            };
            members.push(member);
            if other.options.sparse_infill_density.0
                > regions[owner.region_index].options.sparse_infill_density.0
            {
                owner = member;
            }
        }
        let merged_surfaces = (members.len() > 1)
            .then(|| merge_surfaces(regions, &members))
            .transpose()?;
        groups.push(PerimeterGroup {
            owner,
            members,
            merged_surfaces,
        });
    }
    Ok(())
}

fn merge_surfaces(
    regions: &[PostRegion],
    members: &[RegionLayerIndex],
) -> Result<Vec<RegionSurface>, SliceError> {
    let mut buckets = BTreeMap::<u16, Vec<&RegionSurface>>::new();
    for member in members {
        for surface in &regions[member.region_index].layers[member.layer_index].surfaces {
            buckets
                .entry(surface.as_parts().5)
                .or_default()
                .push(surface);
        }
    }
    let mut merged = Vec::new();
    for sources in buckets.into_values() {
        let polygons = sources
            .iter()
            .map(|surface| surface.as_parts().1.clone())
            .collect::<Vec<_>>();
        let offset =
            offset_expolygons(&polygons, SAFETY_OFFSET, JoinType::Miter, 3.0).map_err(|_| {
                SliceError::InvalidInput("invalid compatible perimeter geometry".into())
            })?;
        merged.extend(
            offset
                .into_iter()
                .map(|polygon| sources[0].clone_with_expolygon(polygon)),
        );
    }
    Ok(merged)
}
