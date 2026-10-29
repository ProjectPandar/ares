//! PrintApply.cpp1059–1080,1676–1693: common PrintRegion configuration ownership.
use super::{RegionGraph, register_region};
use crate::geometry::fixed_gcc_sort_by;
use crate::{OrcaInt, ProjectObject, ProjectVolumeType, RegionOptions};

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PaintedRegion {
    pub(crate) extruder: usize,
    pub(crate) parent: usize,
    pub(crate) region_id: usize,
}

fn used_states(source: &ProjectObject) -> [bool; 17] {
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
fn configuration(parent: &RegionOptions, extruder: usize) -> RegionOptions {
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
    for (records, painted) in graph.ranges.iter().zip(&mut graph.painted_regions) {
        for extruder in (1..used.len()).filter(|&state| used[state]) {
            for (parent, record) in records.iter().enumerate().filter(|(_, r)| {
                matches!(
                    r.kind,
                    ProjectVolumeType::ModelPart | ProjectVolumeType::ParameterModifier
                )
            }) {
                let cfg = configuration(&graph.all_regions[record.region_id.unwrap()], extruder);
                let region_id = register_region(&mut graph.all_regions, cfg);
                painted.push(PaintedRegion {
                    extruder,
                    parent,
                    region_id,
                });
            }
        }
        fixed_gcc_sort_by(painted, |a, b| {
            let a_parent = records[a.parent].region_id.unwrap();
            let b_parent = records[b.parent].region_id.unwrap();
            a_parent < b_parent || (a_parent == b_parent && a.extruder < b.extruder)
        });
    }
}
