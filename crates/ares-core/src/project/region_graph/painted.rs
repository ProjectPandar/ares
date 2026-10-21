//! PrintApply.cpp1059–1080,1676–1693: common PrintRegion configuration ownership.
use super::{RegionGraph, register_region};
use crate::{OrcaInt, ProjectObject, ProjectVolumeType, RegionOptions};

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PaintedRegion {
    pub(crate) extruder: usize,
    pub(crate) parent: usize,
    pub(crate) region_id: usize,
}

pub(crate) fn used_states(source: &ProjectObject) -> [bool; 17] {
    let mut used = [false; 17];
    for volume in source.volumes() {
        for (dst, state) in used
            .iter_mut()
            .zip(volume.mesh().mmu_segmentation_facets.used_states())
        {
            *dst |= state;
        }
    }
    used
}
pub(crate) fn configuration(parent: &RegionOptions, extruder: usize) -> RegionOptions {
    let mut cfg = parent.clone();
    cfg.outer_wall_filament_id = OrcaInt(extruder as i32);
    cfg.inner_wall_filament_id = OrcaInt(extruder as i32);
    cfg.internal_solid_filament_id = OrcaInt(extruder as i32);
    cfg.top_surface_filament_id = OrcaInt(extruder as i32);
    cfg.bottom_surface_filament_id = OrcaInt(extruder as i32);
    cfg.sparse_infill_filament_id = OrcaInt(extruder as i32);
    cfg
}
pub(super) fn register(source: &ProjectObject, graph: &mut RegionGraph) {
    if !graph.mmu_painted {
        return;
    }
    let used = used_states(source);
    for records in &graph.ranges {
        for extruder in (1..used.len()).filter(|&state| used[state]) {
            for record in records.iter().filter(|r| {
                matches!(
                    r.kind,
                    ProjectVolumeType::ModelPart | ProjectVolumeType::ParameterModifier
                )
            }) {
                let cfg = configuration(&graph.all_regions[record.region_id.unwrap()], extruder);
                register_region(&mut graph.all_regions, cfg);
            }
        }
    }
}
