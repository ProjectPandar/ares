use crate::{
    ObjectOptions, ProjectObject, ProjectSettings, ProjectVolumeType, RegionOptions, SliceError,
    options::{RegionBase, RegionOverrideSources},
};

use super::{
    ValidatedMaterializedProject,
    grouping::GroupedPrintObjects,
    layers::{layer_candidate_ranges, layer_range_source_index},
    occupancy::model_part_occupies_range,
    types::{
        ResolvedLayerCandidate, ResolvedModelPartCandidate, ResolvedPrintObjectConfig,
        ResolvedProjectObject,
    },
};

pub(crate) fn resolve_project_objects(
    settings: &ProjectSettings,
    validated: ValidatedMaterializedProject,
    objects: &[ProjectObject],
    grouped: &GroupedPrintObjects,
) -> Result<Vec<ResolvedProjectObject>, SliceError> {
    let z_compensation = z_shrinkage_compensation(settings, validated.logical_filament_count);
    grouped
        .by_object
        .iter()
        .filter(|groups| !groups.transforms.is_empty())
        .map(|groups| {
            let object = &objects[groups.source_object_index];
            let object_options = ObjectOptions::resolve(
                &settings.process.object,
                object.object_overrides(),
                validated.logical_filament_count,
            );
            Ok(ResolvedProjectObject {
                source_object_index: groups.source_object_index,
                object: object_options,
                print_objects: groups
                    .transforms
                    .iter()
                    .copied()
                    .zip(groups.instance_indices(object))
                    .map(|(transform, instance_indices)| ResolvedPrintObjectConfig {
                        transform: transform.with_z_shrinkage_compensation(z_compensation),
                        instance_indices,
                    })
                    .collect(),
                layer_candidates: Vec::new(),
                all_regions: Vec::new(),
            })
        })
        .collect()
}

fn z_shrinkage_compensation(settings: &ProjectSettings, logical_count: usize) -> f64 {
    let xy = &settings.filament.print.filament_shrink.0;
    let z = &settings.filament.print.filament_shrinkage_compensation_z.0;
    let first = (xy[0].0, z[0].0);
    let same = (1..logical_count).all(|index| {
        let current_xy = xy.get(index).unwrap_or(&xy[0]).0;
        let current_z = z.get(index).unwrap_or(&z[0]).0;
        (current_xy, current_z) == first
    });
    if same { 100.0 / first.1 } else { 1.0 }
}

pub(crate) fn resolve_project_candidates(
    settings: &ProjectSettings,
    validated: ValidatedMaterializedProject,
    objects: &[ProjectObject],
    grouped: &GroupedPrintObjects,
    process_variant_indices: &[usize],
) -> Result<Vec<ResolvedProjectObject>, SliceError> {
    let mut resolved = resolve_project_objects(settings, validated, objects, grouped)?;

    for (groups, resolved_object) in grouped
        .by_object
        .iter()
        .filter(|groups| !groups.transforms.is_empty())
        .zip(&mut resolved)
    {
        let representative = resolved_object.print_objects[0].transform;
        let object = &objects[groups.source_object_index];

        let normalized_ranges = layer_candidate_ranges(object.layer_config_ranges());
        let layer_candidates = normalized_ranges
            .iter()
            .copied()
            .map(|range| {
                let source_range_index =
                    layer_range_source_index(&normalized_ranges, (range.min_z, range.max_z))
                        .expect("normalized candidate range must remain addressable");
                let layer_range = source_range_index
                    .map(|index| object.layer_config_ranges()[index].region_overrides());
                let model_parts = object
                    .volumes()
                    .iter()
                    .enumerate()
                    .filter(|(_, volume)| volume.volume_type() == ProjectVolumeType::ModelPart)
                    .filter(|(_, volume)| {
                        model_part_occupies_range(
                            representative,
                            volume,
                            normalized_ranges.len(),
                            range,
                        )
                    })
                    .map(|(volume_index, volume)| ResolvedModelPartCandidate {
                        volume_index,
                        region: RegionOptions::resolve(
                            &settings.filament.region,
                            RegionOverrideSources {
                                base: RegionBase::ModelPart {
                                    process: &settings.process.region,
                                    object: Some(object.region_overrides()),
                                    layer_range,
                                },
                                volume: volume.region_overrides(),
                                material: None,
                            },
                            validated.logical_filament_count,
                            process_variant_indices,
                        ),
                    })
                    .collect();
                ResolvedLayerCandidate {
                    min_z: range.min_z,
                    max_z: range.max_z,
                    source_range_index,
                    model_parts,
                }
            })
            .collect();

        resolved_object.layer_candidates = layer_candidates;
        // PrintObject.cpp:3592-3705 resolves modifier feature IDs from parent regions.
        resolved_object.all_regions = crate::project::region_graph::object_region_graph(
            object,
            resolved_object,
            &settings.filament.region,
            validated.logical_filament_count,
            process_variant_indices,
        )
        .all_regions;
    }

    Ok(resolved)
}
