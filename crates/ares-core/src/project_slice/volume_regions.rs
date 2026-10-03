use crate::{
    ProjectObject, ProjectVolumeType, RegionOptions,
    options::FilamentRegionSourceOptions,
    project::{effective_config::types::ResolvedProjectObject, region_graph::build_region_graph},
};

use super::volume_bounds::{BoundedPrintObject, VolumeOccurrenceId};

#[derive(Clone, Debug, PartialEq)]
pub(super) struct VolumeRegion {
    pub(super) source_volume_index: usize,
    pub(super) occurrence_id: VolumeOccurrenceId,
    pub(super) kind: ProjectVolumeType,
    pub(super) parent: Option<usize>,
    pub(super) region_id: Option<usize>,
    pub(super) bound_index: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct VolumeRegionGraph {
    pub(super) all_regions: Vec<RegionOptions>,
    pub(super) volume_regions: Vec<VolumeRegion>,
}

pub(super) fn build_volume_region_graph(
    source: &ProjectObject,
    resolved: &ResolvedProjectObject,
    bounded: &BoundedPrintObject,
    filament: &FilamentRegionSourceOptions,
    logical_filament_count: usize,
) -> VolumeRegionGraph {
    let [candidate] = resolved.layer_candidates.as_slice() else {
        panic!("bounded object must have exactly one resolved layer candidate");
    };
    let boxes = source
        .volumes()
        .iter()
        .enumerate()
        .map(|(index, _)| {
            bounded
                .bound_for_source_index(index)
                .map(|bound| bound.bbox())
        })
        .collect();
    let mut graph = build_region_graph(
        source,
        std::slice::from_ref(candidate),
        &[boxes],
        filament,
        logical_filament_count,
    );
    let volume_regions = graph
        .ranges
        .pop()
        .expect("single range graph")
        .into_iter()
        .map(|record| {
            let bound_index = bounded
                .bound_index_for_source_index(record.source_volume_index)
                .expect("region volume owns a bound");
            VolumeRegion {
                source_volume_index: record.source_volume_index,
                occurrence_id: bounded.bound_at(bound_index).occurrence_id(),
                kind: record.kind,
                parent: record.parent,
                region_id: record.region_id,
                bound_index,
            }
        })
        .collect();
    VolumeRegionGraph {
        all_regions: graph.all_regions,
        volume_regions,
    }
}
