pub(crate) mod bounds;
mod fuzzy;
mod painted;
pub(crate) use fuzzy::{FuzzyRegion, PaintedRegions};
pub(crate) use painted::PaintedRegion;

use super::effective_config::types::{
    ResolvedLayerCandidate, ResolvedModelPartCandidate, ResolvedProjectObject,
};
use crate::{
    ProjectObject, ProjectVolume, ProjectVolumeType, RegionOptions,
    options::{FilamentRegionSourceOptions, RegionBase, RegionOverrideSources},
};
use bounds::{BoundingBox3f, transformed_mesh_bbox, transformed_mesh_range_bboxes};

pub(crate) struct VolumeRegion {
    pub(crate) source_volume_index: usize,
    pub(crate) kind: ProjectVolumeType,
    pub(crate) parent: Option<usize>,
    pub(crate) region_id: Option<usize>,
    bbox: BoundingBox3f,
}

pub(crate) struct RegionGraph {
    pub(crate) all_regions: Vec<RegionOptions>,
    pub(crate) ranges: Vec<Vec<VolumeRegion>>,
    pub(crate) painted_regions: Vec<Vec<PaintedRegion>>,
    pub(crate) mmu_painted: bool,
    pub(crate) fuzzy_regions: Vec<Vec<FuzzyRegion>>,
}

pub(crate) fn object_region_graph(
    source: &ProjectObject,
    resolved: &ResolvedProjectObject,
    filament: &FilamentRegionSourceOptions,
    logical_count: usize,
    process_variant_indices: &[usize],
) -> RegionGraph {
    let candidates = &resolved.layer_candidates;
    let mut boxes = vec![vec![None; source.volumes().len()]; candidates.len()];
    let ranges = candidates
        .iter()
        .map(|range| (range.min_z - 1e-4, range.max_z + 1e-4))
        .collect::<Vec<_>>();
    let offset = if source.is_mm_painted(logical_count) {
        0.0
    } else {
        (resolved.object.xy_contour_compensation.0 as f32).max(0.0)
    };
    for (index, volume) in source.volumes().iter().enumerate() {
        if matches!(
            volume.volume_type(),
            ProjectVolumeType::SupportEnforcer | ProjectVolumeType::SupportBlocker
        ) || volume.mesh().triangles().is_empty()
        {
            continue;
        }
        // `trafo_for_bbox` (`PrintApply.cpp:622-630`): drop only the object's bed placement and
        // keep the volume's own XY offset.
        let transform = resolved.print_objects[0]
            .transform
            .without_xy_translation()
            .then(volume.transform());
        if candidates.len() == 1 {
            boxes[0][index] = Some(transformed_mesh_bbox(volume.mesh(), transform, offset));
        } else {
            for (range, bbox) in boxes.iter_mut().zip(transformed_mesh_range_bboxes(
                volume.mesh(),
                transform,
                &ranges,
                offset,
            )) {
                range[index] = bbox;
            }
        }
    }
    build_region_graph(
        source,
        candidates,
        &boxes,
        (filament, logical_count, process_variant_indices),
    )
}

pub(crate) fn build_region_graph(
    source: &ProjectObject,
    candidates: &[ResolvedLayerCandidate],
    boxes: &[Vec<Option<BoundingBox3f>>],
    (filament, logical_count, process_variant_indices): (
        &FilamentRegionSourceOptions,
        usize,
        &[usize],
    ),
) -> RegionGraph {
    let mut graph = RegionGraph {
        all_regions: Vec::new(),
        ranges: candidates.iter().map(|_| Vec::new()).collect(),
        painted_regions: candidates.iter().map(|_| Vec::new()).collect(),
        mmu_painted: source.is_mm_painted(logical_count),
        fuzzy_regions: candidates.iter().map(|_| Vec::new()).collect(),
    };
    for (source_volume_index, volume) in source.volumes().iter().enumerate() {
        let kind = volume.volume_type();
        for ((candidate, bounds), records) in candidates.iter().zip(boxes).zip(&mut graph.ranges) {
            let Some(bbox) = bounds[source_volume_index] else {
                continue;
            };
            match kind {
                ProjectVolumeType::ModelPart => {
                    let region =
                        model_part_for_source_index(&candidate.model_parts, source_volume_index);
                    let region_id = register_region(&mut graph.all_regions, region.region.clone());
                    records.push(VolumeRegion {
                        source_volume_index,
                        kind,
                        parent: None,
                        region_id: Some(region_id),
                        bbox,
                    });
                }
                ProjectVolumeType::NegativeVolume => records.push(VolumeRegion {
                    source_volume_index,
                    kind,
                    parent: None,
                    region_id: None,
                    bbox,
                }),
                ProjectVolumeType::ParameterModifier => add_modifier(
                    records,
                    &mut graph.all_regions,
                    (source_volume_index, volume, bbox),
                    (filament, logical_count, process_variant_indices),
                ),
                ProjectVolumeType::SupportEnforcer | ProjectVolumeType::SupportBlocker => {}
            }
        }
    }
    painted::register(source, &mut graph);
    fuzzy::register(source, &mut graph);
    graph
}

fn register_region(regions: &mut Vec<RegionOptions>, region: RegionOptions) -> usize {
    if let Some(index) = regions.iter().position(|existing| existing == &region) {
        index
    } else {
        let index = regions.len();
        regions.push(region);
        index
    }
}

pub(crate) fn model_part_for_source_index(
    model_parts: &[ResolvedModelPartCandidate],
    source_volume_index: usize,
) -> &ResolvedModelPartCandidate {
    let index = model_parts
        .binary_search_by_key(&source_volume_index, |part| part.volume_index)
        .expect("bounded model part must have a resolved region");
    &model_parts[index]
}

fn add_modifier(
    records: &mut Vec<VolumeRegion>,
    regions: &mut Vec<RegionOptions>,
    (source_volume_index, volume, bbox): (usize, &ProjectVolume, BoundingBox3f),
    (filament, logical_count, process_variant_indices): (
        &FilamentRegionSourceOptions,
        usize,
        &[usize],
    ),
) {
    let mut added = false;
    let mut fallback = None;
    for parent in (0..records.len()).rev() {
        let Some(parent_bbox) = extended_parent_bbox(records, parent) else {
            continue;
        };
        if !parent_bbox.intersects(bbox) {
            continue;
        }
        let parent_id = records[parent]
            .region_id
            .expect("printable parent owns a region");
        let child = RegionOptions::resolve(
            filament,
            RegionOverrideSources {
                base: RegionBase::Modifier {
                    parent: &regions[parent_id],
                },
                volume: volume.region_overrides(),
                material: None,
            },
            logical_count,
            process_variant_indices,
        );
        if child != regions[parent_id] {
            let region_id = register_region(regions, child);
            records.push(VolumeRegion {
                source_volume_index,
                kind: ProjectVolumeType::ParameterModifier,
                parent: Some(parent),
                region_id: Some(region_id),
                bbox,
            });
            added = true;
        } else if records[parent].kind == ProjectVolumeType::ModelPart && fallback.is_none() {
            fallback = Some((parent, parent_id));
        }
    }
    if !added && let Some((parent, region_id)) = fallback {
        records.push(VolumeRegion {
            source_volume_index,
            kind: ProjectVolumeType::ParameterModifier,
            parent: Some(parent),
            region_id: Some(region_id),
            bbox,
        });
    }
}

fn extended_parent_bbox(records: &[VolumeRegion], mut index: usize) -> Option<BoundingBox3f> {
    let mut bbox = records[index].bbox;
    loop {
        match records[index].kind {
            ProjectVolumeType::ModelPart => return Some(bbox),
            ProjectVolumeType::ParameterModifier => {
                index = records[index].parent?;
                bbox.extend(records[index].bbox);
            }
            ProjectVolumeType::NegativeVolume
            | ProjectVolumeType::SupportEnforcer
            | ProjectVolumeType::SupportBlocker => return None,
        }
    }
}
