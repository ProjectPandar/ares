//! ToolOrdering.cpp:1278-1316: actual layer usage precedes grouping and reorder.
use super::{OrderingScope, RaftSchedule, ToolOrdering, collect_layers, finish_layers};
#[path = "filament_group/mod.rs"]
mod filament_group;
mod geometry;
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
    let automatic = filament_group::check_scope(
        &traversal.resolved.views.full,
        is_bbl,
        &used,
        traversal.objects.len(),
        multiple_nozzles,
    )?;
    let mut nozzle_groups = None;
    let mut assignments_published = false;
    let current = &traversal.resolved.views.full;
    let sequential = current.process.print.print_sequence == ProcessPrintSequence::ByObject;
    let single_head_auto = current.project.print.nozzle_diameter.0.len() == 1
        && current.project.gcode.filament_map_mode != ProjectFilamentMapMode::Manual;
    // Print.cpp:3700–3734 also publishes single-head sequential auto maps.
    if !sequential || traversal.objects.len() == 1 || single_head_auto {
        let changed = filament_map_recommend::apply_used_tpu_map(
            &mut traversal.resolved.views.full,
            &used,
            is_bbl,
        );
        // ToolOrdering.cpp:3054–3084 also writes static maps in Manual mode.
        let static_views = if automatic {
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
            || (traversal
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
                &traversal.project.documents().project_settings_raw,
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
    if let Some(groups) = nozzle_groups {
        ordering.nozzle_groups = groups;
    }
    traversal.resolved.process_nozzle_assignments = assignments_published.then(|| {
        super::nozzle_group::process_assignments(
            &traversal.resolved.views.full,
            &ordering.nozzle_groups,
            traversal.resolved.logical_filament_count,
        )
    });
    Ok(ordering)
}

// PrintObject.cpp:349–515; Print.cpp:3593–3604. Borrow geometry; keep area limits separate.
