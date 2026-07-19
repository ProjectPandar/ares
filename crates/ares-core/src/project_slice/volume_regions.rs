use crate::{
    ProjectObject, ProjectVolume, ProjectVolumeType, RegionOptions,
    options::{FilamentRegionSourceOptions, RegionBase, RegionOverrideSources},
    project::effective_config::types::{ResolvedModelPartCandidate, ResolvedProjectObject},
};

use super::volume_bounds::{
    BoundedPrintObject, BoundingBox3f, PostBoundsVolume, VolumeOccurrenceId,
};

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

struct ModifierContext<'a> {
    bounded: &'a BoundedPrintObject,
    filament: &'a FilamentRegionSourceOptions,
    logical_filament_count: usize,
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
    let mut graph = VolumeRegionGraph {
        all_regions: Vec::new(),
        volume_regions: Vec::new(),
    };
    let modifier_context = ModifierContext {
        bounded,
        filament,
        logical_filament_count,
    };
    for (source_volume_index, volume) in source.volumes().iter().enumerate() {
        let kind = volume.volume_type();
        if matches!(
            kind,
            ProjectVolumeType::SupportEnforcer | ProjectVolumeType::SupportBlocker
        ) {
            continue;
        }
        let Some(carrier) = bounded.volume_for_source_index(source_volume_index) else {
            continue;
        };
        let bound_index = bounded
            .bound_index_for_occurrence(carrier.occurrence_id())
            .expect("bounded volume must have a bound");
        if kind == ProjectVolumeType::NegativeVolume {
            graph.volume_regions.push(VolumeRegion {
                source_volume_index,
                occurrence_id: carrier.occurrence_id(),
                kind,
                parent: None,
                region_id: None,
                bound_index,
            });
            continue;
        }
        if kind == ProjectVolumeType::ParameterModifier {
            add_modifier_regions(&mut graph, volume, carrier, bound_index, &modifier_context);
            continue;
        }
        let region = model_part_for_source_index(&candidate.model_parts, source_volume_index)
            .region
            .clone();
        let region_id = register_region(&mut graph.all_regions, region);
        graph.volume_regions.push(VolumeRegion {
            source_volume_index,
            occurrence_id: carrier.occurrence_id(),
            kind,
            parent: None,
            region_id: Some(region_id),
            bound_index,
        });
    }
    graph
}

pub(super) fn model_part_for_source_index(
    model_parts: &[ResolvedModelPartCandidate],
    source_volume_index: usize,
) -> &ResolvedModelPartCandidate {
    let index = model_parts
        .binary_search_by_key(&source_volume_index, |part| part.volume_index)
        .expect("bounded model part must have a resolved region");
    &model_parts[index]
}

fn add_modifier_regions(
    graph: &mut VolumeRegionGraph,
    volume: &ProjectVolume,
    carrier: &PostBoundsVolume,
    bound_index: usize,
    context: &ModifierContext<'_>,
) {
    let source_volume_index = carrier.source_volume_index();
    let occurrence_id = carrier.occurrence_id();
    let modifier_bbox = context.bounded.bound_at(bound_index).bbox();
    let parent_count = graph.volume_regions.len();
    let mut changed_added = false;
    let mut fallback = None;
    for parent_index in (0..parent_count).rev() {
        let parent_record = &graph.volume_regions[parent_index];
        let Some(parent_bbox) = extended_parent_bbox(graph, context.bounded, parent_index) else {
            continue;
        };
        if !parent_bbox.intersects(modifier_bbox) {
            continue;
        }
        let parent_region_id = parent_record
            .region_id
            .expect("printable parent must own a region");
        let child = RegionOptions::resolve(
            context.filament,
            RegionOverrideSources {
                base: RegionBase::Modifier {
                    parent: &graph.all_regions[parent_region_id],
                },
                volume: volume.region_overrides(),
                material: None,
            },
            context.logical_filament_count,
        );
        if child != graph.all_regions[parent_region_id] {
            let region_id = register_region(&mut graph.all_regions, child);
            graph.volume_regions.push(VolumeRegion {
                source_volume_index,
                occurrence_id,
                kind: ProjectVolumeType::ParameterModifier,
                parent: Some(parent_index),
                region_id: Some(region_id),
                bound_index,
            });
            changed_added = true;
        } else if parent_record.kind == ProjectVolumeType::ModelPart && fallback.is_none() {
            fallback = Some((parent_index, parent_region_id));
        }
    }
    if !changed_added && let Some((parent, region_id)) = fallback {
        graph.volume_regions.push(VolumeRegion {
            source_volume_index,
            occurrence_id,
            kind: ProjectVolumeType::ParameterModifier,
            parent: Some(parent),
            region_id: Some(region_id),
            bound_index,
        });
    }
}

fn register_region(regions: &mut Vec<RegionOptions>, region: RegionOptions) -> usize {
    if let Some(id) = regions.iter().position(|existing| existing == &region) {
        id
    } else {
        let id = regions.len();
        regions.push(region);
        id
    }
}

fn extended_parent_bbox(
    graph: &VolumeRegionGraph,
    bounded: &BoundedPrintObject,
    mut record_index: usize,
) -> Option<BoundingBox3f> {
    let first = &graph.volume_regions[record_index];
    let mut bbox = bounded.bound_at(first.bound_index).bbox();
    loop {
        let record = &graph.volume_regions[record_index];
        match record.kind {
            ProjectVolumeType::ModelPart => return Some(bbox),
            ProjectVolumeType::ParameterModifier => {
                record_index = record.parent?;
                bbox.extend(
                    bounded
                        .bound_at(graph.volume_regions[record_index].bound_index)
                        .bbox(),
                );
            }
            ProjectVolumeType::NegativeVolume
            | ProjectVolumeType::SupportEnforcer
            | ProjectVolumeType::SupportBlocker => return None,
        }
    }
}

const _: fn(
    &ProjectObject,
    &ResolvedProjectObject,
    &BoundedPrintObject,
    &FilamentRegionSourceOptions,
    usize,
) -> VolumeRegionGraph = build_volume_region_graph;
