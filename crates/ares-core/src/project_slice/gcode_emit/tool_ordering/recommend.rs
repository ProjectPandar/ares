//! ToolOrdering.cpp:1278-1316: actual layer usage precedes grouping and reorder.
use super::{OrderingScope, RaftSchedule, ToolOrdering, collect_layers, finish_layers};
#[path = "filament_group/mod.rs"]
mod filament_group;
mod geometry;
mod manual_nozzle;
use crate::project_slice::gcode_emit::wipe_tower::estimate::config::DynamicPrintConfig;
use crate::{
    ProcessPrintSequence, ProjectFilamentMapMode, SliceError,
    options::{
        filament_map_recommend, refresh_filament_map, update_static_filament_maps,
        write_config_block,
    },
    project_slice::{
        island_print_order::OrderedExtrusionLayer,
        prepare_infill::horizontal_shell_propagation::PreparedPostHorizontalShellPropagation,
        support_material::InstalledSupportLayer,
    },
};
use geometry::geometric_unprintables;

pub(in crate::project_slice::gcode_emit) fn build_recommended(
    horizontal: &mut PreparedPostHorizontalShellPropagation,
    objects: &[Vec<OrderedExtrusionLayer>],
    support: &[Vec<InstalledSupportLayer>],
    raft: Option<&RaftSchedule>,
) -> Result<ToolOrdering, SliceError> {
    let traversal = &mut horizontal.predecessor;
    let (layers, bottom_z, max_height) =
        collect_layers(traversal, objects, support, raft, OrderingScope::All)?;
    let mut used = layers
        .iter()
        .flat_map(|layer| layer.extruders.iter().copied())
        .collect::<Vec<_>>();
    used.sort_unstable();
    used.dedup();
    let is_bbl = crate::print::validate::project_is_bbl(
        &traversal.resolved.views.full,
        &traversal.project.documents().project_settings_raw,
        None,
    )?;
    let multiple_nozzles = traversal
        .project
        .documents()
        .project_settings_raw
        .get("extruder_max_nozzle_count")
        .and_then(|value| value.token())
        .is_some_and(|wire| {
            wire.split([';', ','])
                .any(|count| count.parse::<i32>().is_ok_and(|value| value > 1))
        });
    let current = &traversal.resolved.views.full;
    let manual_group = (current.project.gcode.filament_map_mode
        == ProjectFilamentMapMode::NozzleManual)
        .then(|| {
            manual_nozzle::Grouping::resolve(
                current,
                &traversal.project.documents().project_settings_raw,
                &used,
            )
        });
    let automatic = manual_group.is_none()
        && filament_group::check_scope(
            current,
            is_bbl,
            &used,
            traversal.objects.len(),
            multiple_nozzles,
        )?;
    let mut nozzle_groups = None;
    let mut assignments_published = false;
    let sequential = current.process.print.print_sequence == ProcessPrintSequence::ByObject;
    let single_head_auto = manual_group.is_none()
        && current.project.print.nozzle_diameter.0.len() == 1
        && current.project.gcode.filament_map_mode != ProjectFilamentMapMode::Manual;
    // Print.cpp:3704–3739 publishes static automatic maps in by-object mode.
    let non_bbl_multi_head_auto_for_flush = !is_bbl
        && !multiple_nozzles
        && current.project.print.nozzle_diameter.0.len() > 1
        && current.project.gcode.filament_map_mode == ProjectFilamentMapMode::AutoForFlush;
    if !sequential
        || traversal.objects.len() == 1
        || single_head_auto
        || non_bbl_multi_head_auto_for_flush
    {
        let changed = manual_group.is_none()
            && filament_map_recommend::apply_used_tpu_map(
                &mut traversal.resolved.views.full,
                &used,
                is_bbl,
            );
        // ToolOrdering.cpp:3054–3084 also writes static maps in Manual mode.
        let static_views = if let Some(group) = &manual_group {
            group.write_back(
                &mut traversal.resolved.filament_variant_checkpoint,
                &traversal.resolved.views.full,
                &used,
            )?
        } else if automatic {
            let config = DynamicPrintConfig::new(&traversal.resolved.views.full)?;
            let geometry =
                geometric_unprintables(&horizontal.objects, traversal, objects, &config)?;
            let (views, groups) = filament_group::apply(
                &mut traversal.resolved.filament_variant_checkpoint,
                (
                    &traversal.resolved.views.full,
                    &traversal.project.documents().project_settings_raw,
                ),
                &layers,
                geometry,
                config,
            )?;
            nozzle_groups = Some(groups);
            Some(views)
        } else {
            update_static_filament_maps(
                &mut traversal.resolved.filament_variant_checkpoint,
                &traversal.resolved.views.full,
                &traversal.project.documents().project_settings_raw,
                &used,
                is_bbl,
            )?
        };
        assignments_published = static_views.is_some();
        if static_views.is_some()
            || changed
            || (manual_group.is_none()
                && traversal
                    .resolved
                    .views
                    .full
                    .project
                    .gcode
                    .filament_map_mode
                    != ProjectFilamentMapMode::Manual
                && traversal.resolved.applied_filament_map
                    != traversal.resolved.views.full.project.gcode.filament_map)
        {
            // Print.cpp:3166-3200 refreshes from the PrintApply.cpp:1165-1172 checkpoint.
            traversal.resolved.views = if let Some(views) = static_views {
                views
            } else {
                refresh_filament_map(
                    &traversal.resolved.filament_variant_checkpoint,
                    &traversal.resolved.views.full.project.gcode.filament_map,
                    &traversal.resolved.views.full,
                )?
            };
            let mut block = Vec::new();
            // The same `GCode.cpp:8342-8346` block, rendered again from the refreshed config:
            // the sliced plate's tower entry.
            write_config_block(
                &traversal.resolved.views,
                (
                    &traversal.project.documents().project_settings_raw,
                    traversal.project.documents().config_composer,
                ),
                traversal.project.current_plate_index(),
                None,
                &mut block,
            )?;
            traversal.config_block = (!block.is_empty()).then_some(block);
        }
        if let Err(message) = filament_map_recommend::check_filament_printable_after_group(
            &traversal.resolved.views.full,
            &used,
            is_bbl,
        ) {
            // OrcaSlicer.cpp:6286-6291 catches slicing exceptions, not Print::validate.
            let settings = &traversal.resolved.views.full;
            return Err(SliceError::Slicing {
                message,
                plate_index: traversal.project.first_plate_id(),
                layer_height: f64::from(settings.process.object.layer_height.0 as f32),
                wall_loops: settings.process.region.wall_loops.0,
                sparse_infill_density: f64::from(
                    settings.process.region.sparse_infill_density.0 as f32,
                ),
            });
        }
    }
    let mut ordering = finish_layers(traversal, layers, bottom_z, max_height, false)?;
    if let Some(group) = manual_group {
        // Print.cpp:3705–3743 also publishes the print-wide result when
        // sequential manual printing skips write-back.
        nozzle_groups = Some(group.groups);
        traversal.resolved.manual_nozzle_maps = Some(group.maps);
        assignments_published = true;
    }
    if let Some(groups) = nozzle_groups {
        ordering.nozzle_groups = groups;
    }
    traversal.resolved.process_nozzle_assignments = assignments_published.then(|| {
        let nozzle_map = traversal.resolved.manual_nozzle_maps.as_ref().map_or(
            &traversal
                .resolved
                .views
                .full
                .project
                .gcode
                .filament_nozzle_map,
            |maps| &maps.nozzles,
        );
        super::nozzle_group::process_assignments(
            nozzle_map,
            &ordering.nozzle_groups,
            traversal.resolved.logical_filament_count,
        )
    });
    Ok(ordering)
}

// PrintObject.cpp:349–515; Print.cpp:3593–3604. Borrow geometry; keep area limits separate.
