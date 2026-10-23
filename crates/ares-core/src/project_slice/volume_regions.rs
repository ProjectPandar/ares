use crate::{
    ProjectObject, ProjectVolumeType, RegionOptions,
    options::FilamentRegionSourceOptions,
    project::{
        effective_config::types::ResolvedProjectObject,
        region_graph::{PaintedRegions, object_region_graph},
    },
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
pub(super) struct VolumeRegionRange {
    pub(super) min_z: f64,
    pub(super) max_z: f64,
    pub(super) volume_regions: Vec<VolumeRegion>,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct VolumeRegionGraph {
    pub(super) all_regions: Vec<RegionOptions>,
    pub(super) ranges: Vec<VolumeRegionRange>,
}

pub(super) fn build_volume_region_graph(
    source: &ProjectObject,
    resolved: &ResolvedProjectObject,
    bounded: &BoundedPrintObject,
    filament: &FilamentRegionSourceOptions,
    logical_filament_count: usize,
) -> (VolumeRegionGraph, PaintedRegions) {
    let graph = object_region_graph(source, resolved, filament, logical_filament_count);
    let ranges = graph
        .ranges
        .into_iter()
        .zip(&resolved.layer_candidates)
        .map(|(records, candidate)| VolumeRegionRange {
            min_z: candidate.min_z,
            max_z: candidate.max_z,
            volume_regions: records
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
                .collect(),
        })
        .collect();
    (
        VolumeRegionGraph {
            all_regions: graph.all_regions,
            ranges,
        },
        PaintedRegions {
            mmu: graph.painted_regions,
            fuzzy: graph.fuzzy_regions,
        },
    )
}
