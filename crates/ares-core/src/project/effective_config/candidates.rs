use crate::{
    ObjectOptions, ProjectObject, ProjectSettings, ProjectVolumeType, RegionOptions, SliceError,
    options::{RegionBase, RegionOptionOverrides, RegionOverrideSources},
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
            reject_unsupported_object_sources(object)?;
            Ok(ResolvedProjectObject {
                source_object_index: groups.source_object_index,
                object: object_options,
                print_objects: groups
                    .transforms
                    .iter()
                    .copied()
                    .map(|transform| ResolvedPrintObjectConfig {
                        transform: transform.with_z_shrinkage_compensation(z_compensation),
                    })
                    .collect(),
                layer_candidates: Vec::new(),
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
) -> Result<Vec<ResolvedProjectObject>, SliceError> {
    let mut resolved = resolve_project_objects(settings, validated, objects, grouped)?;

    for (groups, resolved_object) in grouped
        .by_object
        .iter()
        .filter(|groups| !groups.transforms.is_empty())
        .zip(&mut resolved)
    {
        let representative = groups.transforms[0];
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
    }

    Ok(resolved)
}

fn reject_unsupported_object_sources(object: &ProjectObject) -> Result<(), SliceError> {
    for volume in object
        .volumes()
        .iter()
        .filter(|volume| volume.volume_type() == ProjectVolumeType::ParameterModifier)
    {
        if let Some(key) = first_unsupported_modifier_key(volume.region_overrides()) {
            return Err(SliceError::UnsupportedProjectFeature(key.to_owned()));
        }
    }
    Ok(())
}

fn first_unsupported_modifier_key(overrides: &RegionOptionOverrides) -> Option<&'static str> {
    if overrides.wall_loops.is_some() {
        Some("wall_loops")
    } else if overrides.sparse_infill_density.is_some() {
        Some("sparse_infill_density")
    } else if overrides.top_shell_layers.is_some() {
        Some("top_shell_layers")
    } else if overrides.bottom_shell_layers.is_some() {
        Some("bottom_shell_layers")
    } else if overrides.sparse_infill_filament_id.is_some() {
        Some("sparse_infill_filament_id")
    } else if overrides.internal_solid_filament_id.is_some() {
        Some("internal_solid_filament_id")
    } else if overrides.top_surface_filament_id.is_some() {
        Some("top_surface_filament_id")
    } else if overrides.bottom_surface_filament_id.is_some() {
        Some("bottom_surface_filament_id")
    } else if overrides.outer_wall_filament_id.is_some() {
        Some("outer_wall_filament_id")
    } else if overrides.inner_wall_filament_id.is_some() {
        Some("inner_wall_filament_id")
    } else {
        None
    }
}
