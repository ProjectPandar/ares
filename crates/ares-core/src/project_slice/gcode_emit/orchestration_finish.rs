use super::*;

pub(super) fn finish(
    mut output: Vec<u8>,
    state: &motion::EmitState,
    traversal: &PreparedPostClassicTraversal,
    nozzle_groups: &[tool_ordering::NozzleGroup],
    (record_moves, tower_statistics): (bool, (f64, f64)),
) -> Result<crate::ProcessedGCode, SliceError> {
    // Upstream throws after emit when any custom template failed
    // (`GCode.cpp:3637-3645` — `failed_templates` collected via
    // `placeholder_parser_process` `GCode.cpp:3920-3933`).
    if !state.failed_templates.is_empty() {
        return Err(template_failure::from_failed_templates(
            &state.failed_templates,
            traversal,
        ));
    }
    orchestration_header::finalize_layer_count(&mut output, traversal, state);
    let nozzle_counts = super::tool_ordering::nozzle_count(traversal);
    let extruder_colors = processor::extruder_colors(&traversal.resolved.views.full.filament.gcode);
    let (processed, printable_area) = processor::process(
        output,
        !traversal.resolved.views.full.printer.gcode.disable_m73.0,
        traversal
            .resolved
            .views
            .full
            .printer
            .gcode
            .machine_load_filament_time
            .0,
        processor::ProcessorLimits::from_traversal(traversal, state.tags.is_bbl())
            .with_settings(&traversal.resolved.views.full)
            .extruder_colors(extruder_colors)
            .writer_extruders(state.extruders.ids().to_vec())
            .tower_statistics(tower_statistics)
            .recording_moves(record_moves)
            .mass_locator(record_moves.then(|| super::object_mass::Locator::new(traversal)))
            .pre_heating(traversal.resolved.views.runtime_gcode.enable_pre_heating.0)
            .nozzle_groups(nozzle_groups)
            .nozzle_counts(&nozzle_counts),
    );
    retained_output::check(processed, printable_area, traversal)
}
