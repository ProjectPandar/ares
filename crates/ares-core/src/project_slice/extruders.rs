use crate::{
    ProcessBrimType, ProjectObject, ProjectVolumeType, RegionOptions,
    project::effective_config::types::ResolvedProjectObject,
};

pub(super) fn collect_project_object_extruders(
    source_objects: &[ProjectObject],
    resolved_objects: &[ResolvedProjectObject],
    logical_filament_count: usize,
) -> Vec<Vec<usize>> {
    let has_brim = resolved_objects
        .iter()
        .any(|resolved| supported_brim(&resolved.object));

    resolved_objects
        .iter()
        .map(|resolved| {
            let mut extruders = Vec::new();
            for region in resolved
                .layer_candidates
                .iter()
                .flat_map(|candidate| &candidate.model_parts)
                .map(|part| &part.region)
            {
                append_region_extruders(&mut extruders, region, has_brim, logical_filament_count);
            }

            let source = &source_objects[resolved.source_object_index];
            let object_extruder = source.region_overrides().extruder;
            for volume in source.volumes().iter().filter(|volume| {
                matches!(
                    volume.volume_type(),
                    ProjectVolumeType::ModelPart | ProjectVolumeType::ParameterModifier
                )
            }) {
                let volume_extruder = volume.region_overrides().extruder;
                let selected = match volume_extruder {
                    Some(value) if value.0 > 0 => value.0,
                    _ => object_extruder.map_or(1, |value| value.0),
                };
                if selected > 0 {
                    extruders.push(selected as usize - 1);
                }
            }

            extruders.sort_unstable();
            extruders.dedup();
            extruders
        })
        .collect()
}

fn supported_brim(object: &crate::ObjectOptions) -> bool {
    object.raft_layers.0 <= 0
        && (object.brim_type == ProcessBrimType::AutoBrim
            || (object.brim_type != ProcessBrimType::NoBrim && object.brim_width.0 > 0.0))
}

fn append_region_extruders(
    output: &mut Vec<usize>,
    region: &RegionOptions,
    has_brim: bool,
    logical_filament_count: usize,
) {
    if region.wall_loops.0 > 0 || has_brim {
        append_region_selector(
            output,
            region.outer_wall_filament_id.0,
            logical_filament_count,
        );
    }
    if region.wall_loops.0 > 1 {
        append_region_selector(
            output,
            region.inner_wall_filament_id.0,
            logical_filament_count,
        );
    }
    if region.sparse_infill_density.0 > 0.0 {
        append_region_selector(
            output,
            region.sparse_infill_filament_id.0,
            logical_filament_count,
        );
    }
    if region.sparse_infill_density.0 > 0.0
        || region.top_shell_layers.0 > 0
        || region.bottom_shell_layers.0 > 0
    {
        append_region_selector(
            output,
            region.internal_solid_filament_id.0,
            logical_filament_count,
        );
    }
    if region.top_shell_layers.0 > 0 {
        append_region_selector(
            output,
            region.top_surface_filament_id.0,
            logical_filament_count,
        );
    }
    if region.bottom_shell_layers.0 > 0 {
        append_region_selector(
            output,
            region.bottom_surface_filament_id.0,
            logical_filament_count,
        );
    }
}

fn append_region_selector(output: &mut Vec<usize>, selector: i32, logical_count: usize) {
    let normalized = selector.saturating_sub(1).max(0) as usize;
    output.push(if normalized >= logical_count {
        0
    } else {
        normalized
    });
}
