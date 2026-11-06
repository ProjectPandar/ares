//! ToolOrdering.cpp:1278-1316: actual layer usage precedes grouping and reorder.
use super::{OrderingScope, RaftSchedule, ToolOrdering, collect_layers, finish_layers};
use crate::{
    ProcessPrintSequence, ProjectFilamentMapMode, SliceError,
    options::{
        filament_map_recommend, refresh_filament_map, update_static_filament_maps,
        write_config_block,
    },
    project_slice::{
        island_print_order::OrderedExtrusionLayer,
        perimeters::classic::traversal::PreparedPostClassicTraversal,
        support_material::InstalledSupportLayer,
    },
};

pub(in crate::project_slice::gcode_emit) fn build_recommended(
    traversal: &mut PreparedPostClassicTraversal,
    objects: &[Vec<OrderedExtrusionLayer>],
    support: &[Vec<InstalledSupportLayer>],
    raft: Option<&RaftSchedule>,
) -> Result<ToolOrdering, SliceError> {
    let (layers, bottom_z, max_height) =
        collect_layers(traversal, objects, support, raft, OrderingScope::All)?;
    if traversal.resolved.views.full.process.print.print_sequence != ProcessPrintSequence::ByObject
        || traversal.objects.len() == 1
    {
        let mut used = layers
            .iter()
            .flat_map(|layer| layer.extruders.iter().copied())
            .collect::<Vec<_>>();
        used.sort_unstable();
        used.dedup();
        // The existing identity adapter calls is_bbl_printer(model, selected, system)
        // and decodes inherited system identity (OrcaSlicer.cpp:6043-6059).
        let is_bbl = crate::print::validate::project_is_bbl(
            &traversal.resolved.views.full,
            &traversal.project.documents().project_settings_raw,
            None,
        )?;
        let changed = filament_map_recommend::apply_used_tpu_map(
            &mut traversal.resolved.views.full,
            &used,
            is_bbl,
        );
        // ToolOrdering.cpp:3054–3084 also writes static maps in Manual mode.
        let static_views = if traversal.resolved.views.full.process.print.print_sequence
            == ProcessPrintSequence::ByLayer
        {
            update_static_filament_maps(
                &mut traversal.resolved.filament_variant_checkpoint,
                &traversal.resolved.views.full,
                &traversal.project.documents().project_settings_raw,
                &used,
            )?
        } else {
            None
        };
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
    finish_layers(traversal, layers, bottom_z, max_height, false)
}
