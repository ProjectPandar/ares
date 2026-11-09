use crate::{
    ObjectOptions, ProcessBrimType, ProcessTimelapseType, ProjectObject, ProjectSettings,
    ProjectVolumeType, RegionOptions,
};

use super::{
    grouping::GroupedPrintObjects,
    types::{BoundedProjectUsage, ProjectUsageCoverage, ResolvedProjectObject},
};

pub(crate) struct ProjectUsageSources<'a> {
    pub(crate) settings: &'a ProjectSettings,
    pub(crate) objects: &'a [ProjectObject],
    pub(crate) grouped: &'a GroupedPrintObjects,
    pub(crate) resolved: &'a [ResolvedProjectObject],
    pub(crate) custom_tool_changes: &'a [u32],
}

pub(crate) struct ObjectUsageInput<'a> {
    pub(crate) object: &'a ObjectOptions,
    pub(crate) regions: &'a [RegionOptions],
    pub(crate) object_extruder: Option<i32>,
    pub(crate) model_part_extruders: Vec<Option<i32>>,
    pub(crate) layer_range_extruders: Vec<i32>,
}

pub(crate) fn collect_bounded_project_usage(
    sources: ProjectUsageSources<'_>,
    wipe_settings: &ProjectSettings,
) -> BoundedProjectUsage {
    let objects = sources
        .grouped
        .by_object
        .iter()
        .filter(|group| !group.transforms.is_empty())
        .zip(sources.resolved)
        .map(|(group, resolved)| {
            let object = &sources.objects[group.source_object_index];
            ObjectUsageInput {
                object: &resolved.object,
                regions: &resolved.all_regions,
                object_extruder: object.region_overrides().extruder.map(|value| value.0),
                model_part_extruders: object
                    .volumes()
                    .iter()
                    .filter(|volume| {
                        matches!(
                            volume.volume_type(),
                            ProjectVolumeType::ModelPart | ProjectVolumeType::ParameterModifier
                        )
                    })
                    .map(|volume| volume.region_overrides().extruder.map(|value| value.0))
                    .collect(),
                layer_range_extruders: object
                    .layer_config_ranges()
                    .iter()
                    .filter_map(|range| range.region_overrides().extruder.map(|value| value.0))
                    .collect(),
            }
        })
        .collect::<Vec<_>>();
    collect_usage(
        &objects,
        wipe_settings,
        sources.settings.filament.gcode.filament_diameter.0.len(),
        sources.custom_tool_changes,
    )
}

pub(crate) fn collect_usage(
    objects: &[ObjectUsageInput<'_>],
    wipe_settings: &ProjectSettings,
    logical_filament_count: usize,
    custom_tool_changes: &[u32],
) -> BoundedProjectUsage {
    let has_brim = objects.iter().any(|input| supported_brim(input.object));
    let mut object_filaments = Vec::new();
    let mut support_filaments = Vec::new();
    let mut support_uses_current = false;

    for input in objects {
        for region in input.regions {
            append_region_filaments(&mut object_filaments, region, has_brim);
        }
        for &volume_selector in &input.model_part_extruders {
            let selector = match volume_selector {
                Some(selector) if selector != 0 => selector,
                _ => input.object_extruder.unwrap_or(1),
            };
            append_positive(&mut object_filaments, selector);
        }
        for &selector in &input.layer_range_extruders {
            append_positive(&mut object_filaments, selector);
        }
        let object = input.object;
        if object.enable_support.0
            || object.enforce_support_layers.0 > 0
            || object.raft_layers.0 > 0
        {
            for selector in [
                object.support_filament.0,
                object.support_interface_filament.0,
            ] {
                support_uses_current |= append_support(&mut support_filaments, selector);
            }
        }
    }

    sort_remove_duplicates(&mut object_filaments);
    if support_uses_current {
        support_filaments.extend_from_slice(&object_filaments);
    }
    sort_remove_duplicates(&mut support_filaments);

    let mut supported_used_filaments = object_filaments;
    supported_used_filaments.extend(support_filaments);
    // Print.cpp:784-793 appends current-plate ToolChanges before the tower
    // selector and final deduplication, using unsigned 32-bit subtraction.
    let colour_count = wipe_settings.filament.gcode.filament_colour.0.len();
    supported_used_filaments.extend(
        custom_tool_changes
            .iter()
            .copied()
            .filter(|&selector| selector as usize <= colour_count)
            .map(|selector| selector.wrapping_sub(1) as usize),
    );
    let wipe_selector = wipe_settings.process.print.wipe_tower_filament.0;
    if has_wipe_tower(wipe_settings, logical_filament_count)
        && wipe_selector != 0
        && supported_used_filaments.len() > 1
    {
        append_positive(&mut supported_used_filaments, wipe_selector);
    }
    sort_remove_duplicates(&mut supported_used_filaments);

    BoundedProjectUsage {
        supported_used_filaments,
        coverage: ProjectUsageCoverage::TypedConfigSourcesOnly,
    }
}

pub(crate) fn supported_brim(object: &crate::ObjectOptions) -> bool {
    object.raft_layers.0 <= 0
        && (object.brim_type == ProcessBrimType::AutoBrim
            || (object.brim_type != ProcessBrimType::NoBrim && object.brim_width.0 > 0.0))
}

pub(crate) fn append_region_filaments(
    output: &mut Vec<usize>,
    region: &RegionOptions,
    has_brim: bool,
) {
    if region.wall_loops.0 > 0 || has_brim {
        append_positive(output, region.outer_wall_filament_id.0);
    }
    if region.wall_loops.0 > 1 {
        append_positive(output, region.inner_wall_filament_id.0);
    }
    if region.sparse_infill_density.0 > 0.0 {
        append_positive(output, region.sparse_infill_filament_id.0);
    }
    if region.sparse_infill_density.0 > 0.0
        || region.top_shell_layers.0 > 0
        || region.bottom_shell_layers.0 > 0
    {
        append_positive(output, region.internal_solid_filament_id.0);
    }
    if region.top_shell_layers.0 > 0 {
        append_positive(output, region.top_surface_filament_id.0);
    }
    if region.bottom_shell_layers.0 > 0 {
        append_positive(output, region.bottom_surface_filament_id.0);
    }
}

pub(crate) fn has_wipe_tower(settings: &ProjectSettings, logical_filament_count: usize) -> bool {
    settings.process.print.enable_prime_tower.0
        && ((settings.process.gcode.enable_wrapping_detection.0
            && settings.printer.gcode.wrapping_exclude_area.0.len() > 2)
            || settings.process.print.timelapse_type == ProcessTimelapseType::Smooth
            || (!settings.process.print.spiral_mode.0 && logical_filament_count > 1))
}

fn append_positive(output: &mut Vec<usize>, one_based: i32) {
    if one_based > 0 {
        output.push(one_based as usize - 1);
    }
}

fn append_support(output: &mut Vec<usize>, selector: i32) -> bool {
    if selector == 0 {
        true
    } else {
        append_positive(output, selector);
        false
    }
}

fn sort_remove_duplicates(values: &mut Vec<usize>) {
    values.sort_unstable();
    values.dedup();
}
